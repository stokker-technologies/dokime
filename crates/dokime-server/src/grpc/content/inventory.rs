/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, Response};
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use crate::grpc::transcode::Transcode;
use crate::grpc::value::projection;
use dokime::ExecutionFileEntry;
use dokime::RuntimeHandle;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::content as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

struct List {
    resources: Arc<Resources>,
}

struct Count {
    after: usize,
    returned: usize,
    more: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for List {
    type Declaration = wire::ListExecutionFiles;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ListExecutionFiles, _: &mut (),
    ) -> Result<Response<dokime::domain::model::content::wire::ListExecutionFilesReply>, Status> {
        let (id, epoch) = super::files::execution(&self.resources.runtime, &request.execution_id)?;
        let limit = dokime::domain::model::execution::wire_cursor::page_size(request.limit, &request.after_path)
            .map_err(crate::grpc::value::canonical::cursor_failure)?;
        if !request.after_path.is_empty() && request.cursor.is_none() {
            return Err(Status::invalid_argument("subsequent page requires cursor"));
        }
        if let Some(cursor) = &request.cursor
            && (cursor.epoch != epoch || cursor.execution_id != id.as_str())
        {
            return Err(Status::failed_precondition("file cursor epoch or execution mismatch"));
        }
        let entries = self.resources.runtime.execution_files(&id).await.map_err(super::files::read_failure)?;
        project(&entries, &id, &epoch, request, limit, self.resources.maximum)
            .map(Into::into)
            .map_err(|source| super::files::read_failure(source.into()))
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<List>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

fn project(
    entries: &[ExecutionFileEntry], id: &TestExecutionId, epoch: &str, request: &wire::ListExecutionFiles, limit: usize, maximum: usize,
) -> dokime::harmos::Result<dokime::domain::model::content::wire::ListExecutionFilesReply> {
    let counted = count(entries, request, limit)?;
    let mut selected = projection::slots(counted.returned)?;
    let mut hash = Sha256::new();
    for (index, entry) in entries.iter().enumerate() {
        let file = metadata(entry)?;
        let encoded = file.wire_bytes()?;
        if encoded.len() > maximum {
            return Err(exhausted());
        }
        let mut prefix = Vec::with_capacity(10);
        prost::encoding::encode_varint(encoded.len() as u64, &mut prefix);
        hash.update(&prefix);
        hash.update(&encoded);
        if (counted.after..counted.after + counted.returned).contains(&index) {
            selected.push(file);
        }
    }
    let digest = hash.finalize();
    if request.cursor.as_ref().is_some_and(|cursor| cursor.metadata_sha256.as_slice() != digest.as_slice()) {
        return Err(dokime::harmos::Error::InvalidInput { field: "execution files cursor", reason: "metadata fingerprint differs" });
    }
    let next_after_path =
        if counted.more { Some(projection::text(&selected.last().ok_or_else(invalid_wire)?.relative_path)?) } else { None };
    let mut metadata_sha256 = projection::slots(digest.len())?;
    metadata_sha256.extend_from_slice(&digest);
    let page = dokime::domain::model::content::wire::ListExecutionFilesReply {
        files: selected,
        next_after_path,
        cursor: Some(dokime::domain::model::content::wire::ExecutionFilesCursor {
            epoch: projection::text(epoch)?,
            execution_id: projection::text(id.as_str())?,
            metadata_sha256,
        }),
    };
    if page.wire_len()? > maximum {
        return Err(exhausted());
    }
    Ok(page)
}

fn count(entries: &[ExecutionFileEntry], request: &wire::ListExecutionFiles, limit: usize) -> dokime::harmos::Result<Count> {
    if entries.windows(2).any(|pair| pair[0].relative_path >= pair[1].relative_path) {
        return Err(invalid_wire());
    }
    let after = if request.after_path.is_empty() {
        0
    } else {
        entries
            .iter()
            .position(|entry| entry.relative_path == request.after_path)
            .map(|index| index + 1)
            .ok_or(dokime::harmos::Error::InvalidInput { field: "execution files after path", reason: "after path not found" })?
    };
    let returned = entries.len().saturating_sub(after).min(limit);
    let more = entries.len().saturating_sub(after) > returned;
    Ok(Count { after, returned, more })
}

fn metadata(entry: &ExecutionFileEntry) -> dokime::harmos::Result<dokime::domain::model::content::wire::ExecutionFileMetadata> {
    Ok(dokime::domain::model::content::wire::ExecutionFileMetadata {
        relative_path: projection::text(&entry.relative_path)?,
        content_type: entry.content_type.as_deref().map(projection::text).transpose()?,
        kind: super::files::kind(&entry.content_kind),
        state: super::files::state(&entry.stream_state),
        size_bytes: entry.size_bytes,
        created_at: Some(timestamp(entry)?),
    })
}

fn timestamp(entry: &ExecutionFileEntry) -> dokime::harmos::Result<dokime::domain::model::identity::wire::Timestamp> {
    let at = time::OffsetDateTime::parse(entry.updated_at.as_str(), &time::format_description::well_known::Rfc3339).map_err(|source| {
        dokime::harmos::Error::backend("parse original output creation time", dokime::harmos::BackendFailure::Corrupt, source)
    })?;
    Ok(dokime::domain::model::identity::wire::Timestamp { seconds: at.unix_timestamp(), nanos: at.nanosecond() as i32 })
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "canonical execution inventory" }
}

fn invalid_wire() -> dokime::harmos::Error {
    dokime::harmos::Error::InvalidInput { field: "canonical execution inventory", reason: "original order or encoded length differs" }
}

/* === Tests === */
