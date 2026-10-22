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
use std::{
    future::Future,
    path::{Path, PathBuf},
};

// Third Party Imports
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

// Local Imports
use super::{Client, DownloadError as Error, contracts::content};

/* === Constants === */

const COMMIT_BYTES: u64 = 1024 * 1024;
const BUFFER_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A Caller-Owned Receipt For One Local Recording's Committed Prefix.
//
// Persist this receipt with the recording. Cancelled or failed readers retain
// committed bytes; a resumed reader verifies them before removing an uncommitted
// suffix. This receipt binds the original content identity, not a connection epoch.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileResume {
    path: PathBuf,
    identity: Option<dokime_domain::model::content::wire::ExecutionFileIdentity>,
    offset: u64,
    sha256: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FileResume {
    pub fn new(path: PathBuf) -> Self {
        Self { path, identity: None, offset: 0, sha256: hash(&Sha256::new()) }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn identity(&self) -> Option<&dokime_domain::model::content::wire::ExecutionFileIdentity> {
        self.identity.as_ref()
    }
    pub fn committed_offset(&self) -> u64 {
        self.offset
    }
    pub fn prefix_sha256(&self) -> &str {
        &self.sha256
    }

    pub fn request(&self, execution_id: String, relative_path: String) -> content::FollowExecutionFile {
        content::FollowExecutionFile {
            execution_id,
            relative_path,
            identity: self.identity.clone(),
            offset: self.offset,
            max_bytes: BUFFER_BYTES as u32,
        }
    }
}

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
        &self, request: content::FollowExecutionFile, resume: &mut FileResume, mut committed: F,
    ) -> Result<dokime_domain::model::content::wire::ExecutionFileFollowTerminal, Error>
    where
        F: FnMut(FileResume) -> Fut + Send,
        Fut: Future<Output = Result<(), E>> + Send,
        E: std::error::Error + Send + Sync + 'static,
    {
        if request.offset != resume.offset
            || request.identity != resume.identity
            || request.max_bytes > BUFFER_BYTES as u32
            || resume.identity.is_none() && (resume.offset != 0 || resume.sha256 != hash(&Sha256::new()))
        {
            return Err(Error::Invalid("follow request differs from its local committed receipt"));
        }
        let stream = self.content_follow_execution_file(request.clone()).await?;
        let read = async {
            let frame = stream.message().await?.ok_or(Error::Invalid("follow has no metadata"))?;
            let Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Metadata(metadata)) = frame.payload else {
                return Err(Error::Invalid("follow must begin with metadata"));
            };
            let identity = metadata.identity.as_ref().ok_or(Error::Invalid("follow metadata has no identity"))?;
            validate_identity(identity, &request)?;
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
                    Some(dokime_domain::model::content::wire::ExecutionFileFollowFramePayload::Chunk(chunk)) => {
                        let limit = if request.max_bytes == 0 { BUFFER_BYTES } else { request.max_bytes as usize };
                        if chunk.offset != position || chunk.data.is_empty() || chunk.data.len() > limit {
                            return Err(Error::Invalid("follow chunks are not contiguous or exceed their bound"));
                        }
                        let next = position.checked_add(chunk.data.len() as u64).ok_or(Error::Invalid("follow length overflow"))?;
                        output.write_all(&chunk.data).await?;
                        digest.update(&chunk.data);
                        position = next;
                        if position - resume.offset >= COMMIT_BYTES {
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
    identity: &dokime_domain::model::content::wire::ExecutionFileIdentity, request: &content::FollowExecutionFile,
) -> Result<(), Error> {
    let history = identity.history.as_ref().ok_or(Error::Invalid("follow identity has no history"))?;
    if identity.execution_id != request.execution_id
        || identity.relative_path != request.relative_path
        || identity.file_id.is_empty()
        || identity.file_id.len() > 4096
        || history.store.is_empty()
        || history.store.len() > 4096
        || history.epoch.is_empty()
        || history.epoch.len() > 4096
        || request.identity.as_ref().is_some_and(|expected| expected != identity)
    {
        return Err(Error::Invalid("follow returned a different original content identity"));
    }
    Ok(())
}

async fn open(resume: &FileResume) -> Result<(tokio::fs::File, Sha256), Error> {
    let mut options = tokio::fs::OpenOptions::new();
    options.read(true).write(true);
    if resume.identity.is_none() {
        options.create_new(true);
    }
    let mut file = options.open(&resume.path).await?;
    let metadata = file.metadata().await?;
    if !metadata.is_file() || metadata.len() < resume.offset {
        return Err(Error::Invalid("local recording is missing its committed prefix"));
    }
    let mut digest = Sha256::new();
    let mut remaining = resume.offset;
    let mut buffer = vec![0; BUFFER_BYTES];
    while remaining != 0 {
        let count = remaining.min(BUFFER_BYTES as u64) as usize;
        file.read_exact(&mut buffer[..count]).await?;
        digest.update(&buffer[..count]);
        remaining -= count as u64;
    }
    if hash(&digest) != resume.sha256 {
        return Err(Error::Invalid("local committed prefix digest differs"));
    }
    // Only The Uncommitted Suffix May Be Discarded After Prefix Verification.
    if metadata.len() != resume.offset {
        file.set_len(resume.offset).await?;
    }
    file.seek(std::io::SeekFrom::Start(resume.offset)).await?;
    Ok((file, digest))
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

fn hash(digest: &Sha256) -> String {
    digest.clone().finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */
