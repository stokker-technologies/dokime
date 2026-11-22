/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Transfer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error;
use std::future::Future;

// Third Party Imports
use harmos_operations::storage::{Info, InfoRequest};
use harmos_protocol::model::{FileMetadata, FileReference, History, Upload, UploadState};
use harmos_runtime::BackendFailure;
use harmos_runtime::Identifier;
use harmos_runtime::storage::{FilePhase, FileProducer, FileProgress, FileSession, MutationId, StagedFile, StoreHistory};
use sha2::{Digest, Sha256};

// Local Imports
use crate::host::Context;
use crate::host::files::uploads::{append, create, delete, publish};
use crate::{Error, Result};
use dokime_protocol::host::files::{UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsPublishRequest};
use dokime_storage::persisted::Scope;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn chunk_bytes(context: &Context) -> Result<usize> {
    let info = context.storage::<Info>(InfoRequest {}).await?;
    chunk_limit(context, info.file_batch_bytes)
}
pub(crate) async fn admission(context: &Context, action: &str, request_id: &str) -> Result<(usize, MutationId)> {
    let key = mutation_key(context, action, request_id)?;
    let info = context.storage::<Info>(InfoRequest {}).await?;
    let maximum = chunk_limit(context, info.file_batch_bytes)?;
    let window = info.mutations.ok_or_else(|| Error::Unavailable(format!("{action}: Open a Storage Mutation Window")))?;
    let mutation = MutationId { window: window.id, key };
    tracing::debug!(action, request_id, window = %mutation.window, mutation = %mutation.key, "Admit Storage Mutation");
    Ok((maximum, mutation))
}
pub(crate) fn authorize(context: &Context, file: &FileReference) -> Result<()> {
    if let Some(caller) = &context.caller {
        let metadata = file.metadata.as_ref().ok_or_else(|| Error::Permission("Access File: Supply Its Scoped Reference".into()))?;
        let scope = Scope::Execution(&caller.execution).identifier()?;
        if metadata.scope.as_deref() != Some(scope.as_str()) {
            return Err(Error::Permission("Access File: Select Content in the Calling Execution".into()));
        }
    }
    Ok(())
}
pub(crate) async fn mutation(context: &Context, action: &str, request_id: &str) -> Result<MutationId> {
    let key = mutation_key(context, action, request_id)?;
    let info = context.storage::<Info>(InfoRequest {}).await?;
    let window = info.mutations.ok_or_else(|| Error::Unavailable(format!("{action}: Open a Storage Mutation Window")))?;
    let mutation = MutationId { window: window.id, key };
    tracing::debug!(action, request_id, window = %mutation.window, mutation = %mutation.key, "Admit Storage Mutation");
    Ok(mutation)
}

pub(crate) async fn write<T, F: Future<Output = Result<T>>>(
    context: &Context, action: &'static str, request_id: &str, identity: MutationId, call: impl FnMut(MutationId) -> F,
) -> Result<T> {
    retry(action, identity, mutation(context, action, request_id), call).await
}

pub(crate) async fn publication(
    context: &Context, action: &str, request_id: &str, metadata: FileMetadata, mut read: impl FnMut(u64, &mut [u8]) -> Result<()>,
    cancelled: impl Fn() -> bool,
) -> Result<FileReference> {
    let length = metadata.expected_length.ok_or_else(|| Error::Invalid(format!("{action}: Supply the File Length")))?;
    let digest = metadata.expected_sha256.clone().ok_or_else(|| Error::Invalid(format!("{action}: Supply the File Digest")))?;
    let created = create::derived(
        context,
        UploadsCreateRequest {
            metadata: Some(metadata),
            request_id: format!("{request_id}/create"),
        },
    )
    .await?;
    let upload = created.upload.ok_or_else(|| Error::Precondition(format!("{action}: Create Its Native Upload")))?;
    let result = async {
        let mut offset = 0;
        while offset < length {
            if cancelled() {
                return Err(Error::Cancelled(format!("{action}: Read Ended Before Publication")));
            }
            let count = (length - offset).min(created.chunk_bytes) as usize;
            let mut data = vec![0; count];
            read(offset, &mut data)?;
            append::handle(
                context,
                UploadsAppendRequest {
                    upload: Some(upload.clone()),
                    offset,
                    data,
                    request_id: format!("{request_id}/{offset}"),
                },
            )
            .await?;
            offset += count as u64;
        }
        if cancelled() {
            return Err(Error::Cancelled(format!("{action}: Read Ended Before Publication")));
        }
        let published = publish::handle(
            context,
            UploadsPublishRequest {
                upload: Some(upload.clone()),
                length,
                sha256: digest,
                request_id: format!("{request_id}/publish"),
            },
        )
        .await?;
        published.upload.ok_or_else(|| Error::Precondition(format!("{action}: Publish Its Native File")))
    }
    .await;
    match result {
        Ok(file) => Ok(file),
        Err(source) => {
            if let Err(cleanup) = delete::handle(
                context,
                UploadsDeleteRequest {
                    upload: Some(upload),
                    request_id: format!("{request_id}/delete"),
                },
            )
            .await
            {
                return Err(Error::Invalid(format!("{action}: {source}; Remove Its Failed Upload: {cleanup}")));
            }
            Err(source)
        }
    }
}

pub(crate) fn expired(source: &Error) -> bool {
    let mut cause: Option<&(dyn error::Error + 'static)> = Some(source);
    while let Some(current) = cause {
        if let Some(native) = current.downcast_ref::<harmos_runtime::Error>() {
            let carried =
                native.failure().is_some_and(|failure| failure.detail("storage.kind") == Some("STORAGE_FAILURE_MUTATION_EXPIRED"));
            let backend = matches!(
                native,
                harmos_runtime::Error::Backend {
                    kind: BackendFailure::MutationExpired,
                    ..
                }
            );
            if carried || backend {
                return true;
            }
        }
        cause = current.source();
    }
    false
}
pub(crate) fn staged(upload: &Upload) -> Result<StagedFile> {
    let history = upload.history.clone().ok_or_else(|| Error::Invalid("Access Upload: Supply Its Store History".into()))?;
    let history = StoreHistory::try_from(history).map_err(|cause| Error::Invalid(format!("Access Upload: {cause}")))?;
    let id = Identifier::new(upload.id.clone()).map_err(|cause| Error::Invalid(format!("Access Upload: {cause}")))?;
    Ok(StagedFile { history, id })
}
pub(crate) fn session(upload: Option<Upload>) -> Result<FileSession> {
    let upload = upload.ok_or_else(|| Error::Invalid("Access Upload: Supply Its Writer".into()))?;
    let file = staged(&upload)?;
    if upload.writer_generation == 0 {
        return Err(Error::Precondition("Access Upload: Resume an Active Writer".into()));
    }
    Ok(FileSession {
        history: file.history,
        id: file.id,
        generation: upload.writer_generation,
    })
}
pub(crate) fn upload(production: FileProducer) -> Upload {
    Upload {
        id: production.session.id.to_string(),
        history: Some(History::from(production.session.history)),
        writer_generation: production.session.generation,
        accepted_length: production.length,
        durable_length: 0,
        state: UploadState::Producing as i32,
    }
}
pub(crate) fn progress(file: StagedFile, progress: FileProgress, generation: u64, accepted: u64, durable: u64) -> Upload {
    let state = match progress.phase {
        FilePhase::Producing => UploadState::Producing,
        FilePhase::Finished => UploadState::Finished,
        FilePhase::Abandoned => UploadState::Abandoned,
    };
    Upload {
        id: file.id.to_string(),
        history: Some(file.history.into()),
        writer_generation: if progress.phase == FilePhase::Producing { generation } else { 0 },
        accepted_length: accepted,
        durable_length: durable,
        state: state as i32,
    }
}

/* === Private Functions === */

async fn retry<T, F: Future<Output = Result<T>>>(
    action: &'static str, identity: MutationId, fresh: impl Future<Output = Result<MutationId>>, mut call: impl FnMut(MutationId) -> F,
) -> Result<T> {
    match call(identity).await {
        Err(source) if expired(&source) => {
            let identity = fresh.await?;
            call(identity).await.map_err(|source| match source {
                Error::Native { source, .. } => Error::Native { action, source },
                source => source,
            })
        }
        result => result,
    }
}

fn chunk_limit(context: &Context, file_batch_bytes: u64) -> Result<usize> {
    let available = context.settings.message_bytes.saturating_sub(1024);
    let maximum = usize::try_from(file_batch_bytes).unwrap_or(usize::MAX).min(available);
    if maximum == 0 {
        return Err(Error::Bound("Transfer File: Configure a Message Bound That Can Carry File Bytes".into()));
    }
    Ok(maximum)
}

fn mutation_key(context: &Context, action: &str, request_id: &str) -> Result<Identifier> {
    if request_id.is_empty() {
        return Err(Error::Invalid(format!("{action}: Supply a Request ID")));
    }
    let scope = context.caller.as_ref().map_or("", |caller| caller.execution.as_str());
    let digest = Sha256::digest(format!("{scope}/{action}/{request_id}"));
    let key = Identifier::new(crate::host::views::hex(&digest)).map_err(|cause| Error::Invalid(format!("{action}: {cause}")))?;
    Ok(key)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{expired, progress, retry, session, staged, upload};
    use harmos_protocol::model::{History, Upload, UploadState};
    use harmos_runtime::operations::FailureKind;
    use harmos_runtime::storage::{FilePhase, FileProducer, FileProgress, FileSession, MutationId, StoreHistory};
    use harmos_runtime::{BackendFailure, Failure, Identifier};

    #[test]
    fn only_expired_mutations_request_a_fresh_identity() {
        for kind in [BackendFailure::MutationExpired, BackendFailure::Transient, BackendFailure::Permanent, BackendFailure::Conflict] {
            let source = harmos_runtime::Error::backend("Write File", kind, std::io::Error::other("storage failure"));
            let source = crate::Error::Native { action: "Write File", source };
            assert_eq!(expired(&source), kind == BackendFailure::MutationExpired);
        }
        assert!(!expired(&crate::Error::Invalid("Select a File".into())));
    }

    #[test]
    fn mutation_expiry_survives_native_operation_failure_frames() {
        let failure =
            Failure::new(FailureKind::Transient, "Mutation Expired").with_detail("storage.kind", "STORAGE_FAILURE_MUTATION_EXPIRED");
        let source = harmos_runtime::Error::OperationFailed {
            operation: "Write File".into(),
            source: Arc::new(failure),
        };
        assert!(expired(&crate::Error::Native { action: "Write File", source }));
    }

    #[test]
    fn expired_write_refreshes_once() {
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        runtime.block_on(async {
            for expires_again in [false, true] {
                let initial = MutationId {
                    window: Identifier::new("old-window").unwrap(),
                    key: Identifier::new("request").unwrap(),
                };
                let fresh = MutationId {
                    window: Identifier::new("current-window").unwrap(),
                    ..initial.clone()
                };
                let mut calls = Vec::new();
                let mut refreshed = false;
                let result = retry(
                    "Retain View",
                    initial.clone(),
                    async {
                        refreshed = true;
                        Ok(fresh.clone())
                    },
                    |identity| {
                        calls.push(identity);
                        let result = if calls.len() == 1 || expires_again {
                            let source = harmos_runtime::Error::backend(
                                "Create File",
                                BackendFailure::MutationExpired,
                                std::io::Error::other("expired"),
                            );
                            Err(crate::Error::Native { action: "Create File", source })
                        } else {
                            Ok("published")
                        };
                        std::future::ready(result)
                    },
                )
                .await;
                assert!(refreshed);
                assert_eq!(calls, vec![initial, fresh]);
                if expires_again {
                    assert!(matches!(result, Err(crate::Error::Native { action: "Retain View", .. })));
                } else {
                    assert_eq!(result.unwrap(), "published");
                }
            }
        });
    }

    #[test]
    fn writes_only_refresh_expired_identities() {
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        runtime.block_on(async {
            for kind in [None, Some(BackendFailure::Transient), Some(BackendFailure::Conflict)] {
                let initial = MutationId {
                    window: Identifier::new("current-window").unwrap(),
                    key: Identifier::new("request").unwrap(),
                };
                let mut calls = 0;
                let result = retry("Retain View", initial, async { panic!("unexpected refresh") }, |_| {
                    calls += 1;
                    let result = match kind {
                        None => Ok(()),
                        Some(kind) => Err(crate::Error::Native {
                            action: "Create File",
                            source: harmos_runtime::Error::backend("Create File", kind, std::io::Error::other("failed")),
                        }),
                    };
                    std::future::ready(result)
                })
                .await;
                assert_eq!(calls, 1);
                assert_eq!(result.is_ok(), kind.is_none());
            }
        });
    }

    fn producer() -> FileProducer {
        FileProducer {
            session: FileSession {
                history: StoreHistory {
                    store: Identifier::new("root").unwrap(),
                    epoch: Identifier::new("incarnation").unwrap(),
                },
                id: Identifier::new("payload").unwrap(),
                generation: 7,
            },
            length: 123,
        }
    }
    #[test]
    fn writer_fencing() {
        let producer = producer();
        let wire = upload(producer.clone());
        assert_eq!(session(Some(wire.clone())).unwrap(), producer.session);
        assert_eq!(wire.accepted_length, producer.length);
        assert_eq!(wire.state, UploadState::Producing as i32);
        let mut terminal = wire;
        terminal.writer_generation = 0;
        assert!(session(Some(terminal)).is_err());
    }
    #[test]
    fn durable_progress() {
        let file = staged(&upload(producer())).unwrap();
        let open = progress(file.clone(), FileProgress { length: 123, phase: FilePhase::Producing }, 7, 123, 80);
        assert_eq!(open.accepted_length, 123);
        assert_eq!(open.durable_length, 80);
        assert_eq!(open.writer_generation, 7);
        let closed = progress(file, FileProgress { length: 123, phase: FilePhase::Finished }, 7, 123, 123);
        assert_eq!(closed.state, UploadState::Finished as i32);
        assert_eq!(closed.writer_generation, 0);
    }
    #[test]
    fn malformed_uploads() {
        assert!(staged(&Upload::default()).is_err());
        let wire = Upload {
            id: "payload".into(),
            history: Some(History {
                store: String::new(),
                epoch: "epoch".into(),
            }),
            ..Default::default()
        };
        assert!(staged(&wire).is_err());
    }
}
