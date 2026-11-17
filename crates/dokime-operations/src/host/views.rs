/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Retained Resource Views and Mutation Receipts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;

// Third Party Imports
use harmos_operations::storage::{
    CreateFileProduction, CreateFileProductionRequest, DeleteFile, DeleteFileRequest, GetFileBytes, GetFileBytesRequest,
};
use harmos_protocol::model::{FileReference, PageRequest, ReadCut, Receipt, ResourceRevision};
use harmos_runtime::storage::{self, Identifier, StagedFile};
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::host::files::transfer;
use crate::host::files::uploads::{append, delete, publish};
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::files::{UploadsAppendRequest, UploadsDeleteRequest, UploadsPublishRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn read<T: Message + Default>(
    context: &Context, scope: Identifier, resource: &str, at: Option<&ReadCut>, current: impl Future<Output = Result<T>>,
) -> Result<(T, ReadCut)> {
    let store = context.host.store();
    let name = format!("{}/{}:{scope}/{resource}", store.identity(), scope.as_str().len());
    if let Some(at) = at {
        if at.revisions.is_empty() && at.positions.len() == 1 {
            let position = store.evidence_position(at.positions.first())?;
            let key = format!("view-at:{name}:{}:{}", position.history.as_ref().unwrap().epoch, position.sequence);
            let Some(saved) = retained(context, &key).await? else {
                return Err(Error::Precondition(format!("Read {resource}: The Selected View Is Unavailable; Resynchronize")));
            };
            return Ok((T::decode(saved.as_slice())?, at.clone()));
        }
        if at.revisions.len() != 1 || at.revisions[0].name != name || at.positions.len() != 1 {
            return Err(Error::Invalid(format!("Read {resource}: Supply Its Returned Cut")));
        }
        store.evidence_position(at.positions.first())?;
        let revision = hex(&at.revisions[0].revision);
        let Some(saved) = retained(context, &format!("view:{name}:{revision}")).await? else {
            return Err(Error::Precondition(format!("Read {resource}: Retained View Is Unavailable; Resynchronize")));
        };
        return Ok((T::decode(saved.as_slice())?, at.clone()));
    }
    let position = store.evidence_position(None)?;
    let value = current.await?;
    let payload = {
        let span = if resource.starts_with("plots/") { tracing::info_span!("plot.encode") } else { tracing::Span::none() };
        let _entered = span.enter();
        value.encode_to_vec()
    };
    let revision = Sha256::digest(&payload).to_vec();
    let file = retain(context, &scope, &format!("view:{name}:{}", hex(&revision)), &payload).await?;
    let key = format!("view-at:{name}:{}:{}", position.history.as_ref().unwrap().epoch, position.sequence);
    administration::retain(context, &key, scope, &file).await?;

    Ok((
        value,
        ReadCut {
            positions: vec![position],
            revisions: vec![ResourceRevision { name, revision }],
        },
    ))
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn receipt(context: &Context, request_id: &str) -> Result<Receipt> {
    if request_id.is_empty() {
        return Err(Error::Invalid("Mutate Resource: Supply a Nonempty Request ID".into()));
    }
    let position = context.host.store().evidence_position(None)?;
    Ok(Receipt {
        request_id: request_id.into(),
        accepted_at: Some(position.clone()),
        durable_at: Some(position),
        completed: true,
    })
}

pub(crate) fn continuation(page: Option<&PageRequest>, at: Option<&ReadCut>) -> Result<()> {
    if page.is_some_and(|page| !page.page_token.is_empty()) && at.is_none() {
        return Err(Error::Invalid("List Resource: Continue with the Returned Cut".into()));
    }
    Ok(())
}

pub(crate) fn offset(token: &str, cut: &ReadCut) -> Result<usize> {
    if token.is_empty() {
        return Ok(0);
    }
    let (identity, offset) = token.split_once(':').ok_or_else(|| Error::Invalid("Read Resource: Use the Returned Page Token".into()))?;
    if identity != hex(&Sha256::digest(cut.encode_to_vec())) {
        return Err(Error::Invalid("Read Resource: Continue the Same Selection and Cut".into()));
    }
    offset.parse().map_err(|_| Error::Invalid("Read Resource: Use the Returned Page Token".into()))
}

pub(crate) fn token(offset: usize, cut: &ReadCut) -> String {
    format!("{}:{offset}", hex(&Sha256::digest(cut.encode_to_vec())))
}

pub(crate) fn page<T: Message>(
    rows: Vec<T>, cut: &ReadCut, page: Option<&PageRequest>, maximum: usize, maximum_bytes: usize,
) -> Result<(Vec<T>, String)> {
    let start = offset(page.map_or("", |page| page.page_token.as_str()), cut)?;
    if start > rows.len() {
        return Err(Error::Invalid("List Resource: Continue Within the Returned View".into()));
    }
    let limit = crate::host::projection::limit(page.map_or(0, |page| page.page_size), maximum);
    let bytes = page.filter(|page| page.max_bytes != 0).map_or(maximum_bytes, |page| (page.max_bytes as usize).min(maximum_bytes));
    // Reserve the Cut, Continuation and Collection Summary Envelope Before Rows.
    let mut retained = cut.encoded_len() + 512;
    let total = rows.len();
    let mut selected = Vec::new();
    for row in rows.into_iter().skip(start).take(limit) {
        let size = row.encoded_len() + 12;
        if retained.saturating_add(size) > bytes {
            if selected.is_empty() {
                return Err(Error::Invalid("List Resource: One Row Exceeds the Requested Byte Bound".into()));
            }
            break;
        }
        retained += size;
        selected.push(row);
    }
    let next = start + selected.len();
    Ok((selected, if next < total { token(next, cut) } else { String::new() }))
}

/* === Private Functions === */

async fn retained(context: &Context, key: &str) -> Result<Option<Vec<u8>>> {
    let (file, revision) = administration::message::<FileReference>(context, key).await?;
    if revision.is_empty() {
        return Ok(None);
    }
    let file = storage::FileReference::try_from(file)
        .map_err(|source| Error::Invalid(format!("Read Retained View: Restore a Valid Backup of Its Storage Root: {source}")))?;
    let maximum = transfer::chunk_bytes(context).await? as u64;
    let mut payload = Vec::new();
    while (payload.len() as u64) < file.length {
        let response = context
            .storage::<GetFileBytes>(GetFileBytesRequest {
                file: Some(file.clone()),
                offset: payload.len() as u64,
                max_bytes: maximum,
                at: None,
            })
            .await?;
        if response.data.is_empty() {
            return Err(Error::Precondition("Read Retained View: Restore Its Complete Native File".into()));
        }
        payload.extend(response.data);
    }
    Ok(Some(payload))
}

async fn retain(context: &Context, scope: &Identifier, key: &str, payload: &[u8]) -> Result<FileReference> {
    let (saved, revision) = administration::message::<FileReference>(context, key).await?;
    if !revision.is_empty() {
        return Ok(saved);
    }

    // Produce the Complete View in Bounded Native Chunks Before Publishing Its Index.
    let digest: [u8; 32] = Sha256::digest(payload).into();
    let request_id = uuid::Uuid::now_v7().to_string();
    let maximum = transfer::chunk_bytes(context).await?;
    let mutation = transfer::mutation(context, "Retain View", &request_id).await?;
    let created = context
        .storage::<CreateFileProduction>(CreateFileProductionRequest {
            metadata: Some(storage::FileMetadata {
                id: Identifier::new(format!("view/{request_id}")).map_err(|source| Error::Invalid(format!("Retain View: {source}")))?,
                scope: Some(scope.clone()),
                media_type: "application/x-protobuf".into(),
                expected_length: Some(payload.len() as u64),
                expected_sha256: Some(digest),
            }),
            mutation: Some(mutation),
        })
        .await?;
    let upload =
        created.production.map(transfer::upload).ok_or_else(|| Error::Precondition("Retain View: Create Its Native Upload".into()))?;
    let produced = async {
        let mut offset = 0;
        for data in payload.chunks(maximum) {
            append::handle(
                context,
                UploadsAppendRequest {
                    upload: Some(upload.clone()),
                    offset,
                    data: data.to_vec(),
                    request_id: format!("{request_id}/{offset}"),
                },
            )
            .await?;
            offset += data.len() as u64;
        }
        let published = publish::handle(
            context,
            UploadsPublishRequest {
                upload: Some(upload.clone()),
                length: payload.len() as u64,
                sha256: digest.to_vec(),
                request_id: request_id.clone(),
            },
        )
        .await?;
        published.upload.ok_or_else(|| Error::Precondition("Retain View: Publish Its Native File".into()))
    }
    .await;
    let file = match produced {
        Ok(file) => file,
        Err(source) => {
            if let Err(cleanup) = delete::handle(context, UploadsDeleteRequest { upload: Some(upload), request_id }).await {
                return Err(Error::Invalid(format!("Retain View: {source}; Remove Its Failed Upload: {cleanup}")));
            }
            return Err(source);
        }
    };
    let saved = administration::retain(context, key, scope.clone(), &file).await?;
    if saved != file {
        let file = storage::FileReference::try_from(file)
            .map_err(|source| Error::Invalid(format!("Retain View: Remove Its Unused Native File: {source}")))?;
        let mutation = transfer::mutation(context, "Retain View", &request_id).await?;
        context
            .storage::<DeleteFile>(DeleteFileRequest {
                file: Some(StagedFile {
                    history: file.position.history,
                    id: file.metadata.id,
                }),
                mutation: Some(mutation),
            })
            .await?;
    }
    Ok(saved)
}

/* === Tests === */
