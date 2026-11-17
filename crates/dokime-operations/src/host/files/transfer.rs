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

// Third Party Imports
use harmos_protocol::model::{FileReference, History, Upload, UploadState};
use harmos_runtime::Identifier;
use harmos_runtime::builtins::storage::{Info, InfoRequest};
use harmos_runtime::storage::{FilePhase, FileProducer, FileProgress, FileSession, MutationId, StagedFile, StoreHistory};
use sha2::{Digest, Sha256};

// Local Imports
use crate::host::Context;
use crate::{Error, Result};
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
    let available = context.settings.message_bytes.saturating_sub(1024);
    let maximum = usize::try_from(info.file_batch_bytes).unwrap_or(usize::MAX).min(available);
    if maximum == 0 {
        return Err(Error::Bound("Transfer File: Configure a Message Bound That Can Carry File Bytes".into()));
    }
    Ok(maximum)
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
    if request_id.is_empty() {
        return Err(Error::Invalid(format!("{action}: Supply a Request ID")));
    }
    let info = context.storage::<Info>(InfoRequest {}).await?;
    let window = info.mutations.ok_or_else(|| Error::Unavailable(format!("{action}: Open a Storage Mutation Window")))?;
    let scope = context.caller.as_ref().map_or("", |caller| caller.execution.as_str());
    let digest = Sha256::digest(format!("{scope}/{action}/{request_id}"));
    let key = Identifier::new(crate::host::views::hex(&digest)).map_err(|cause| Error::Invalid(format!("{action}: {cause}")))?;
    Ok(MutationId { window: window.id, key })
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

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{progress, session, staged, upload};
    use harmos_protocol::model::{History, Upload, UploadState};
    use harmos_runtime::Identifier;
    use harmos_runtime::storage::{FilePhase, FileProducer, FileProgress, FileSession, StoreHistory};

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
    fn upload_wire_mapping_preserves_the_writer_fence_and_store_history() {
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
    fn progress_keeps_accepted_and_durable_prefixes_distinct_and_fences_terminal_writers() {
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
    fn uploads_refuse_missing_or_malformed_histories_before_storage_admission() {
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
