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
use std::pin::Pin;

// Third Party Imports
use harmos_operations::storage::{
    CreateFileProduction, CreateFileProductionRequest, DeleteFile, DeleteFileRequest, GetFileBytes, GetFileBytesRequest,
};
use harmos_protocol::model::{FileReference, PageRequest, ReadCut, Receipt, ResourceRevision};
use harmos_runtime::storage::{self, Identifier, StagedFile};
use prost::Message;
use sha2::{Digest, Sha256};
use tokio::sync::oneshot;
use uuid::Uuid;

// Local Imports
use crate::host::files::transfer;
use crate::host::files::uploads::{append, delete, publish};
use crate::host::projection;
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
        let position_matches = at.positions.len() == 1;
        let revision_matches = at.revisions.len() == 1 && at.revisions[0].name == name;
        if !position_matches || !revision_matches {
            return Err(Error::Invalid(format!("Read {resource}: Supply Its Returned Cut")));
        }
        {
            let (reader, _cancellation) = store.read_session()?;
            reader.evidence_position(at.positions.first())?;
        }
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
    retain(context, &scope, format!("view:{name}:{}", hex(&revision)), payload).await?;

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
    rows: impl IntoIterator<Item = T, IntoIter: ExactSizeIterator>, cut: &ReadCut, page: Option<&PageRequest>, maximum: usize,
    maximum_bytes: usize,
) -> Result<(Vec<T>, String)> {
    let rows = rows.into_iter();
    let start = offset(page.map_or("", |page| page.page_token.as_str()), cut)?;
    if start > rows.len() {
        return Err(Error::Invalid("List Resource: Continue Within the Returned View".into()));
    }
    let limit = projection::limit(page.map_or(0, |page| page.page_size), maximum);
    let bytes = page.filter(|page| page.max_bytes != 0).map_or(maximum_bytes, |page| (page.max_bytes as usize).min(maximum_bytes));
    // Reserve the Cut, Continuation and Collection Summary Envelope Before Rows.
    let mut retained = cut.encoded_len() + 512;
    let total = rows.len();
    let mut selected = Vec::new();
    for row in rows.skip(start).take(limit) {
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

async fn retain(context: &Context, scope: &Identifier, key: String, payload: Vec<u8>) -> Result<FileReference> {
    // The Host Settles Admitted File Work After the Read Deadline Drops Its Receiver.
    let owned = context.clone();
    let scope = scope.clone();
    settle(
        |work| context.host.spawn_work(work),
        move |sender| async move {
            let result = retain_owned(&owned, &scope, &key, &payload, &sender).await;
            if let Err(Err(source)) = sender.send(result)
                && !matches!(source, Error::Cancelled(_))
            {
                tracing::error!(%source, "Settle Retained View: Inspect Its Storage Before Reusing the Root");
            }
        },
    )
    .await
}

async fn settle<F: Future<Output = ()> + Send + 'static>(
    spawn: impl FnOnce(Pin<Box<dyn Future<Output = ()> + Send>>) -> Result<()>,
    work: impl FnOnce(oneshot::Sender<Result<FileReference>>) -> F,
) -> Result<FileReference> {
    let (sender, receiver) = oneshot::channel();
    spawn(Box::pin(work(sender)))?;
    receiver.await.map_err(|_| Error::Cancelled("Retain View: Reconnect After the Host Stops".into()))?
}

async fn retain_owned(
    context: &Context, scope: &Identifier, key: &str, payload: &[u8], sender: &oneshot::Sender<Result<FileReference>>,
) -> Result<FileReference> {
    if sender.is_closed() {
        return Err(Error::Cancelled("Retain View: Read Ended Before Publication".into()));
    }
    let (saved, revision) = administration::message::<FileReference>(context, key).await?;
    if !revision.is_empty() {
        return Ok(saved);
    }

    // Produce the Complete View in Bounded Native Chunks Before Publishing Its Index.
    let digest: [u8; 32] = Sha256::digest(payload).into();
    let request_id = Uuid::now_v7().to_string();
    let (maximum, mutation) = transfer::admission(context, "Retain View", &request_id).await?;
    let created = transfer::write(context, "Retain View", &request_id, mutation, |mutation| {
        context.storage::<CreateFileProduction>(CreateFileProductionRequest {
            metadata: Some(storage::FileMetadata {
                id: Identifier::new(format!("view/{request_id}")).map_err(|source| Error::Invalid(format!("Retain View: {source}")))?,
                scope: Some(scope.clone()),
                media_type: "application/x-protobuf".into(),
                expected_length: Some(payload.len() as u64),
                expected_sha256: Some(digest),
            }),
            mutation: Some(mutation),
        })
    })
    .await?;
    let upload =
        created.production.map(transfer::upload).ok_or_else(|| Error::Precondition("Retain View: Create Its Native Upload".into()))?;
    let produced = async {
        let mut offset = 0;
        for data in payload.chunks(maximum) {
            if sender.is_closed() {
                return Err(Error::Cancelled("Retain View: Read Ended Before Publication".into()));
            }
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
        if sender.is_closed() {
            return Err(Error::Cancelled("Retain View: Read Ended Before Publication".into()));
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
    if sender.is_closed() {
        discard(context, file, &request_id).await?;
        return Err(Error::Cancelled("Retain View: Read Ended Before Indexing".into()));
    }
    finish_file(file.clone(), administration::retain(context, key, scope.clone(), &file), |file| discard(context, file, &request_id)).await
}

async fn finish_file<F: Future<Output = Result<()>>>(
    file: FileReference, indexed: impl Future<Output = Result<FileReference>>, discard: impl FnOnce(FileReference) -> F,
) -> Result<FileReference> {
    let saved = match indexed.await {
        Ok(saved) => saved,
        Err(source) => {
            if let Err(cleanup) = discard(file).await {
                return Err(Error::Invalid(format!("Retain View: {source}; Remove Its Unindexed File: {cleanup}")));
            }
            return Err(source);
        }
    };
    if saved != file {
        discard(file).await?;
    }
    Ok(saved)
}

async fn discard(context: &Context, file: FileReference, request_id: &str) -> Result<()> {
    let file = storage::FileReference::try_from(file)
        .map_err(|source| Error::Invalid(format!("Retain View: Remove Its Unused Native File: {source}")))?;
    let mutation = transfer::mutation(context, "Discard Unused View", request_id).await?;
    let file = StagedFile { history: file.position.history, id: file.metadata.id };
    transfer::write(context, "Discard Unused View", request_id, mutation, |mutation| {
        context.storage::<DeleteFile>(DeleteFileRequest {
            file: Some(file.clone()),
            mutation: Some(mutation),
        })
    })
    .await?;
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::{fs, time::Duration};

    use harmos_protocol::model::{FileMetadata, FileReference, History, Position};
    use harmos_runtime::storage;
    use sha2::{Digest, Sha256};
    use tokio::sync::oneshot;

    use super::{finish_file, settle};
    use crate::{Error, host};

    fn reference(id: &str, data: &[u8]) -> FileReference {
        let file = FileReference {
            metadata: Some(FileMetadata {
                id: id.into(),
                media_type: "application/x-protobuf".into(),
                ..Default::default()
            }),
            position: Some(Position {
                history: Some(History {
                    store: "backend".into(),
                    epoch: "epoch".into(),
                }),
                sequence: 1,
            }),
            length: data.len() as u64,
            sha256: Sha256::digest(data).to_vec(),
        };
        storage::FileReference::try_from(file.clone()).unwrap();
        file
    }

    #[test]
    fn deadline_cleanup() {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let root = tempfile::tempdir().unwrap();
            let path = root.path().join("view.pb");
            fs::write(&path, [1, 2, 3]).unwrap();
            let open = fs::File::open(&path).unwrap();
            let (release, blocked) = oneshot::channel::<()>();
            let removed = path.clone();
            let mut work = None;
            let read = settle(
                |task| {
                    work = Some(tokio::spawn(task));
                    Ok(())
                },
                move |sender| async move {
                    let result = finish_file(
                        reference("view/original", &[1, 2, 3]),
                        async {
                            blocked.await.unwrap();
                            Err(Error::Conflict("index rejected".into()))
                        },
                        |_| async move {
                            drop(open);
                            fs::remove_file(removed).unwrap();
                            Ok(())
                        },
                    )
                    .await;
                    let _ = sender.send(result);
                },
            );
            let reply = host::complete("Read Document", Duration::ZERO, read).await;
            assert!(matches!(reply, Err(Error::Deadline(_))));
            assert!(path.exists());
            release.send(()).unwrap();
            work.unwrap().await.unwrap();
            assert!(!path.exists());
        });
    }

    #[test]
    fn failed_index_cleanup() {
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        runtime.block_on(async {
            let root = tempfile::tempdir().unwrap();
            let path = root.path().join("view.pb");
            fs::write(&path, [1, 2, 3]).unwrap();
            let result =
                finish_file(reference("view/original", &[1, 2, 3]), async { Err(Error::Conflict("index rejected".into())) }, |_| async {
                    fs::remove_file(&path).unwrap();
                    Ok(())
                })
                .await;
            assert!(matches!(result, Err(Error::Conflict(_))));
            assert!(!path.exists());
        });
    }

    #[test]
    fn publication_races() {
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        runtime.block_on(async {
            for same in [false, true] {
                let original = reference("view/original", &[1, 2, 3]);
                let saved = if same { original.clone() } else { reference("view/winner", &[1, 2, 3]) };
                let mut discarded = None;
                let result = finish_file(original.clone(), async { Ok(saved.clone()) }, |file| {
                    discarded = Some(file);
                    async { Ok(()) }
                })
                .await
                .unwrap();
                assert_eq!(result, saved);
                assert_eq!(discarded, (!same).then_some(original));
            }
        });
    }
}
