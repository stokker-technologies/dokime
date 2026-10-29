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
use dokime::harmos::Context;
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use crate::grpc::transcode::Transcode;
use crate::grpc::value::projection;
use dokime::ExecutionFileEntry;
use dokime::app::LiveRuntime;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::content as wire;
use dokime::operations::hosting::{Host, Serve, Served};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

struct Count {
    after: usize,
    returned: usize,
    more: bool,
}

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::ListExecutionFiles),
}

/* === Traits === */

/* === Implementations === */

impl Serve<wire::ListExecutionFiles> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ListExecutionFiles, _: Context<'a, wire::ListExecutionFiles>,
    ) -> Served<'a, dokime::domain::model::content::wire::ListExecutionFilesReply> {
        Box::pin(async move {
            let (id, epoch) = super::files::execution(&dokime::app::live_runtime(&self.runtime).await?, &request.execution_id)?;
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
            let entries = dokime::app::live_runtime(&self.runtime).await?.execution_files(&id).await.map_err(super::files::read_failure)?;
            project(&entries, &id, &epoch, &request, limit, self.maximum)
                .map(Into::into)
                .map_err(|source| super::files::read_failure(source.into()))
        })
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<wire::ListExecutionFiles>>(Host::<wire::ListExecutionFiles>::new(served));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
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
        return Err(dokime::harmos::Error::InvalidInput {
            field: "execution files cursor",
            reason: "metadata fingerprint differs",
        });
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
        entries.iter().position(|entry| entry.relative_path == request.after_path).map(|index| index + 1).ok_or(
            dokime::harmos::Error::InvalidInput {
                field: "execution files after path",
                reason: "after path not found",
            },
        )?
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
    Ok(dokime::domain::model::identity::wire::Timestamp {
        seconds: at.unix_timestamp(),
        nanos: at.nanosecond() as i32,
    })
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted {
        resource: "canonical execution inventory",
    }
}

fn invalid_wire() -> dokime::harmos::Error {
    dokime::harmos::Error::InvalidInput {
        field: "canonical execution inventory",
        reason: "original order or encoded length differs",
    }
}

/* === Tests === */
