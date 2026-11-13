/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Transfer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
use crate::{Error, Result};
use sha2::{Digest, Sha256};
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
