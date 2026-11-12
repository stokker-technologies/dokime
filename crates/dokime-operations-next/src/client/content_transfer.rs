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
use std::io::{Read, Seek};

// Third Party Imports
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, client::{Context, Output}};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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
        history: Some(v1::StoreHistory { store: context.host.identity().backend_id, epoch: "v1".into() }),
        file_id: entry.identifier.clone(), execution_id: entry.execution.clone(), relative_path: entry.path.clone(),
    }
}
pub(super) fn download<O: Message + Send + 'static>(context: &Context, mut source: Source, length: u64, expected_digest: Option<String>, metadata: O,
    chunk: fn(v1::ArtifactChunk) -> O, complete: fn(v1::ArtifactCompletion) -> O, terminal: fn() -> O) -> Result<Output<O>> {
    if metadata.encoded_len() > context.settings.message_bytes {
        return Err(Error::Invalid("Download Content: Shorten Metadata to Fit the Message Budget".into()));
    }
    let maximum = maximum(context,0)?;
    let timeout = context.settings.read_timeout;
    let (sender,output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let result = super::complete("Download Content",timeout,async {
            sender.send(Ok(metadata)).await.map_err(|_| Error::Cancelled)?;
            let mut hash = Sha256::new();
            let mut offset=0;
            while offset < length {
                let size = maximum.min((length-offset) as usize);
                let bytes = match &mut source {
                    Source::Stored { store,identifier } => store.content_window(identifier,offset,size)?,
                    Source::Temporary(file) => { let mut bytes=vec![0;size]; file.read_exact(&mut bytes).map_err(|error| Error::Invalid(format!("Read Temporary Export: {error}")))?; bytes }
                };
                if bytes.len()!=size { return Err(Error::Invalid("Download Content: Retry After the Original Production Changed".into())); }
                hash.update(&bytes);
                let frame=chunk(v1::ArtifactChunk {offset,data:bytes});
                offset+=size as u64;
                sender.send(Ok(frame)).await.map_err(|_| Error::Cancelled)?;
            }
            let sha256: String = hash.finalize().iter().map(|byte|format!("{byte:02x}")).collect();
            if expected_digest.is_some_and(|expected| expected != sha256) {
                return Err(Error::Invalid("Download Content: Restore a Backup Matching Its Stored Digest".into()));
            }
            sender.send(Ok(complete(v1::ArtifactCompletion {size_bytes:length,sha256}))).await.map_err(|_| Error::Cancelled)?;
            sender.send(Ok(terminal())).await.map_err(|_| Error::Cancelled)?;
            Ok(())
        }).await;
        if let Err(error)=result { let _=sender.send(Err(super::status(error))).await; }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}
pub(super) fn temporary(mut file: std::fs::File) -> Result<(Source,u64,String)> {
    file.rewind().map_err(|error| Error::Invalid(format!("Rewind Export: {error}")))?;
    let mut hash=Sha256::new();
    let mut buffer=vec![0;dokime_storage::CONTENT_CHUNK_BYTES];
    let mut length=0;
    loop {
        let count=file.read(&mut buffer).map_err(|error| Error::Invalid(format!("Read Export: {error}")))?;
        if count==0 { break; }
        hash.update(&buffer[..count]); length+=count as u64;
    }
    let digest=hash.finalize().iter().map(|byte|format!("{byte:02x}")).collect();
    file.rewind().map_err(|error| Error::Invalid(format!("Rewind Export: {error}")))?;
    Ok((Source::Temporary(file),length,digest))
}

/* === Private Functions === */

/* === Tests === */
