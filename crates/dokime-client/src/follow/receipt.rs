/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Local Follow Receipts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};

// Third Party Imports
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

// Local Imports
use super::Error;
use dokime_operations::content;

/* === Constants === */

pub(super) const FOLLOW_BUFFER_BYTES: u64 = 65_536;

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
    pub(super) path: PathBuf,
    pub(super) identity: Option<dokime_domain::model::content::wire::ExecutionFileIdentity>,
    pub(super) offset: u64,
    pub(super) sha256: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FileResume {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            identity: None,
            offset: 0,
            sha256: hash(&Sha256::new()),
        }
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

    pub fn request(&self, execution_id: String, relative_path: String) -> Result<content::FollowExecutionFile, Error> {
        Ok(content::FollowExecutionFile {
            execution_id,
            relative_path,
            identity: self.identity.clone(),
            offset: self.offset,
            max_bytes: FOLLOW_BUFFER_BYTES as u32,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn open(resume: &FileResume) -> Result<(tokio::fs::File, Sha256), Error> {
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
    let maximum = FOLLOW_BUFFER_BYTES as usize;
    let mut buffer = vec![0; remaining.min(maximum as u64) as usize];
    while remaining != 0 {
        let count = remaining.min(buffer.len() as u64) as usize;
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

pub(super) fn hash(digest: &Sha256) -> String {
    digest.clone().finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_follow_request() {
        let resume = FileResume::new(PathBuf::from("recording.bin"));
        let request = resume.request("execution".into(), "output.bin".into()).unwrap();
        assert_eq!(request.max_bytes, FOLLOW_BUFFER_BYTES as u32);
        assert_eq!(request.offset, 0);
    }
}
