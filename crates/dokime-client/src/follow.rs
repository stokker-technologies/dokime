/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod receipt;

/* === Re-Exports === */

pub use receipt::FileResume;

/* === Imports === */

// Standard Library Imports
use std::future::Future;

// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

// Local Imports
use super::{Client, DownloadError as Error, contracts::content};
use receipt::{FOLLOW_BUFFER_BYTES, hash, open};

/* === Constants === */

const FOLLOW_COMMIT_BYTES: u64 = 1_048_576;
const IDENTITY_BYTES: u64 = 4_096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    // Records An Original Execution File, Preserving A Verified Resume Receipt.
    //
    // Commits bounded batches to disk before advancing the receipt. Cancellation
    // may leave an uncommitted suffix; the next call removes it only after checking
    // the previously committed prefix. Abandoned output remains partial and has no
    // seal. Finished output is accepted only after its full hash and final status.
    pub async fn follow_execution_file(
        &self, request: content::FollowExecutionFile, resume: &mut FileResume,
    ) -> Result<dokime_domain::model::content::wire::ExecutionFileFollowTerminal, Error> {
        self.follow_execution_file_with_commit(request, resume, |_| async { Ok::<_, std::io::Error>(()) }).await
    }

    // Awaits Caller Receipt Persistence After Each Durable Local Commit.
    //
    // The snapshot contains no secrets. Persistence failure stops the reader
    // before the next frame; the caller's previous external receipt remains the
    // resume authority. The initial content identity is committed before data.
    pub async fn follow_execution_file_with_commit<F, Fut, E>(
        &self, mut request: content::FollowExecutionFile, resume: &mut FileResume, mut committed: F,
    ) -> Result<dokime_domain::model::content::wire::ExecutionFileFollowTerminal, Error>
    where
        F: FnMut(FileResume) -> Fut + Send,
        Fut: Future<Output = Result<(), E>> + Send,
        E: std::error::Error + Send + Sync + 'static,
    {
        let buffer_bytes = FOLLOW_BUFFER_BYTES as u32;
        if request.max_bytes == 0 {
            request.max_bytes = buffer_bytes;
        }
        if request.offset != resume.offset
            || request.identity != resume.identity
            || request.max_bytes > buffer_bytes
            || resume.identity.is_none() && (resume.offset != 0 || resume.sha256 != hash(&Sha256::new()))
        {
            return Err(Error::Invalid("follow request differs from its local committed receipt"));
        }
        let stream = self.stream(request.clone()).await?;
        let read = async {
            let frame = stream.message().await?.ok_or(Error::Invalid("follow has no metadata"))?;
            let Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Metadata(metadata)) = frame.payload else {
                return Err(Error::Invalid("follow must begin with metadata"));
            };
            let identity = metadata.identity.as_ref().ok_or(Error::Invalid("follow metadata has no identity"))?;
            validate_identity(identity, &request, IDENTITY_BYTES)?;
            if metadata.offset != resume.offset
                || metadata.available_length < metadata.offset
                || metadata.kind == dokime_domain::model::content::wire::ExecutionFileKind::Unspecified
                || metadata.phase == dokime_domain::model::content::wire::ExecutionFilePhase::Unspecified
            {
                return Err(Error::Invalid("follow metadata has an invalid prefix or phase"));
            }
            let (mut output, mut digest) = open(resume).await?;
            resume.identity = Some(identity.clone());
            let mut position = resume.offset;
            commit(&mut output, &digest, position, resume).await?;
            committed(resume.clone()).await.map_err(|source| Error::Receipt(Box::new(source)))?;
            let mut terminal = None;
            while let Some(frame) = stream.message().await? {
                if terminal.is_some() {
                    return Err(Error::Invalid("follow continued after its terminal frame"));
                }
                match frame.payload {
                    // An Empty Chunk Is The Server's Quiet-Period Heartbeat.
                    Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Chunk(chunk)) if chunk.data.is_empty() => {}
                    Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Chunk(chunk)) => {
                        let limit = usize::try_from(request.max_bytes).map_err(|_| Error::Invalid("follow buffer exceeds native size"))?;
                        if chunk.offset != position || chunk.data.len() > limit {
                            return Err(Error::Invalid("follow chunks are not contiguous or exceed their bound"));
                        }
                        let next = position.checked_add(chunk.data.len() as u64).ok_or(Error::Invalid("follow length overflow"))?;
                        output.write_all(&chunk.data).await?;
                        digest.update(&chunk.data);
                        position = next;
                        if position - resume.offset >= FOLLOW_COMMIT_BYTES {
                            commit(&mut output, &digest, position, resume).await?;
                            committed(resume.clone()).await.map_err(|source| Error::Receipt(Box::new(source)))?;
                        }
                    }
                    Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Terminal(value)) => {
                        commit(&mut output, &digest, position, resume).await?;
                        committed(resume.clone()).await.map_err(|source| Error::Receipt(Box::new(source)))?;
                        validate_terminal(&value, identity, position, &digest)?;
                        terminal = Some(value);
                    }
                    _ => {
                        return Err(Error::Invalid("follow frame is missing or repeats metadata"));
                    }
                }
            }
            terminal.ok_or(Error::Invalid("follow ended without its terminal frame"))
        }
        .await;
        stream.close().await;
        read
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn validate_identity(
    identity: &dokime_domain::model::content::wire::ExecutionFileIdentity, request: &content::FollowExecutionFile, maximum: u64,
) -> Result<(), Error> {
    let history = identity.history.as_ref().ok_or(Error::Invalid("follow identity has no history"))?;
    if identity.execution_id != request.execution_id
        || identity.relative_path != request.relative_path
        || identity.file_id.is_empty()
        || identity.file_id.len() as u64 > maximum
        || history.store.is_empty()
        || history.store.len() as u64 > maximum
        || history.epoch.is_empty()
        || history.epoch.len() as u64 > maximum
        || request.identity.as_ref().is_some_and(|expected| expected != identity)
    {
        return Err(Error::Invalid("follow returned a different original content identity"));
    }
    Ok(())
}

async fn commit(output: &mut tokio::fs::File, digest: &Sha256, position: u64, resume: &mut FileResume) -> Result<(), Error> {
    output.flush().await?;
    output.sync_data().await?;
    resume.offset = position;
    resume.sha256 = hash(digest);
    Ok(())
}

fn validate_terminal(
    terminal: &dokime_domain::model::content::wire::ExecutionFileFollowTerminal,
    identity: &dokime_domain::model::content::wire::ExecutionFileIdentity, position: u64, digest: &Sha256,
) -> Result<(), Error> {
    if terminal.length != position {
        return Err(Error::Invalid("follow terminal length differs"));
    }
    match terminal.phase {
        dokime_domain::model::content::wire::ExecutionFilePhase::Abandoned if terminal.sealed.is_none() => Ok(()),
        dokime_domain::model::content::wire::ExecutionFilePhase::Finished => {
            let seal = terminal.sealed.as_ref().ok_or(Error::Invalid("finished follow has no seal"))?;
            let saved = seal.position.as_ref().ok_or(Error::Invalid("finished seal has no saved position"))?;
            if seal.identity.as_ref() != Some(identity)
                || saved.history != identity.history
                || seal.length != position
                || seal.sha256 != hash(digest)
            {
                return Err(Error::Invalid("finished seal differs from the full recorded file"));
            }
            Ok(())
        }
        _ => Err(Error::Invalid("follow terminal has an invalid phase or seal")),
    }
}

/* === Tests === */
