/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Content Transfer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{
    Error, Result,
    client::{Context, Output},
};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Download {
    pub source: Source,
    pub length: u64,
    pub expected_digest: Option<String>,
}

/* === Enums === */

pub(super) enum Source {
    Stored { store: dokime_storage::Store, identifier: String },
    Temporary(std::fs::File),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn maximum(context: &Context, requested: u32) -> Result<usize> {
    if requested as usize > dokime_storage::CONTENT_CHUNK_BYTES {
        return Err(Error::Invalid("Read Execution File: Request at Most 64 KiB".into()));
    }
    let maximum = if requested == 0 { dokime_storage::CONTENT_CHUNK_BYTES } else { requested as usize };
    Ok(maximum.min(context.settings.message_bytes / 2))
}
pub(super) fn file_identity(context: &Context, entry: &dokime_storage::ContentEntry) -> v1::ExecutionFileIdentity {
    v1::ExecutionFileIdentity {
        history: Some(v1::StoreHistory {
            store: context.host.identity().backend_id,
            epoch: "v1".into(),
        }),
        file_id: entry.identifier.clone(),
        execution_id: entry.execution.clone(),
        relative_path: entry.path.clone(),
    }
}
pub(super) fn download<O: Message + Send + 'static>(
    context: &Context, download: Download, metadata: O, chunk: fn(v1::ArtifactChunk) -> O, complete: fn(v1::ArtifactCompletion) -> O,
    terminal: fn() -> O,
) -> Result<Output<O>> {
    let Download { mut source, length, expected_digest } = download;
    if metadata.encoded_len() > context.settings.message_bytes {
        return Err(Error::Invalid("Download Content: Shorten Metadata to Fit the Message Budget".into()));
    }
    let maximum = maximum(context, 0)?;
    let timeout = context.settings.read_timeout;
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let result = super::complete("Download Content", timeout, async {
            sender.send(Ok(metadata)).await.map_err(|_| Error::Cancelled)?;
            let mut hash = Sha256::new();
            let mut offset = 0;
            while offset < length {
                let size = maximum.min((length - offset) as usize);
                let bytes = match &mut source {
                    Source::Stored { store, identifier } => store.content_window(identifier, offset, size)?,
                    Source::Temporary(file) => {
                        let mut bytes = vec![0; size];
                        read_temporary(file, offset, &mut bytes)?;
                        bytes
                    }
                };
                if bytes.len() != size {
                    return Err(Error::Invalid("Download Content: Retry After the Original Production Changed".into()));
                }
                hash.update(&bytes);
                let frame = chunk(v1::ArtifactChunk { offset, data: bytes });
                offset += size as u64;
                sender.send(Ok(frame)).await.map_err(|_| Error::Cancelled)?;
            }
            let sha256: String = hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
            if expected_digest.is_some_and(|expected| expected != sha256) {
                return Err(Error::Invalid("Download Content: Restore a Backup Matching Its Stored Digest".into()));
            }
            sender.send(Ok(complete(v1::ArtifactCompletion { size_bytes: length, sha256 }))).await.map_err(|_| Error::Cancelled)?;
            sender.send(Ok(terminal())).await.map_err(|_| Error::Cancelled)?;
            Ok(())
        })
        .await;
        if let Err(error) = result {
            let _ = sender.send(Err(super::status(error))).await;
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}
pub(super) fn temporary(file: std::fs::File) -> Result<(std::fs::File, u64, String)> {
    let length = file.metadata().map_err(|error| Error::Invalid(format!("Inspect Temporary Export: {error}")))?.len();
    let mut hash = Sha256::new();
    let mut offset = 0;
    let mut buffer = vec![0; dokime_storage::CONTENT_CHUNK_BYTES];
    while offset < length {
        let size = buffer.len().min((length - offset) as usize);
        read_temporary(&file, offset, &mut buffer[..size])?;
        hash.update(&buffer[..size]);
        offset += size as u64;
    }
    let digest = hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
    Ok((file, length, digest))
}

/* === Private Functions === */

fn read_temporary(file: &std::fs::File, offset: u64, bytes: &mut [u8]) -> Result<()> {
    let mut read = 0;
    while read < bytes.len() {
        #[cfg(windows)]
        let count = {
            use std::os::windows::fs::FileExt;
            file.seek_read(&mut bytes[read..], offset + read as u64)
        };
        #[cfg(unix)]
        let count = {
            use std::os::unix::fs::FileExt;
            file.read_at(&mut bytes[read..], offset + read as u64)
        };
        let count = count.map_err(|error| Error::Invalid(format!("Read Temporary Export: {error}")))?;
        if count == 0 {
            return Err(Error::Invalid("Download Content: Materialize Its Temporary Output Again".into()));
        }
        read += count;
    }
    Ok(())
}

/* === Tests === */
