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
use std::mem::size_of;

// Third Party Imports
use base64::Engine;
use harmos_domain::Error;
use harmos_domain::Result;
use sha2::Digest;
use sha2::Sha256;

// Local Imports
use super::ExecutionCheckpoint;

/* === Constants === */

const VERSION: u8 = 1;
const BYTES: usize = 1 + 32 + size_of::<u64>();
const TEXT: usize = BYTES.div_ceil(3) * 4 - 1;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct PageCursor {
    identity: [u8; 32],
    after: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos_domain::MemorySize for PageCursor {
    fn memory_size(&self) -> usize {
        size_of::<Self>()
    }
}

impl PageCursor {
    pub fn requested(after: Option<&str>) -> Result<u64> {
        after.map(decode).transpose().map(|cursor| cursor.map_or(0, |(_, position)| position))
    }

    pub fn open(checkpoint: &ExecutionCheckpoint, kind: &str, source: &str, after: Option<&str>) -> Result<Self> {
        checkpoint.validate().map_err(|_| invalid("checkpoint does not bind its evidence boundary"))?;
        let identity = identity(checkpoint, kind, source);
        let Some(after) = after else {
            return Ok(Self { identity, after: 0 });
        };
        let (selected, position) = decode(after)?;
        if selected != identity {
            return Err(invalid("native page cursor belongs to another checkpoint or source"));
        }
        if position == 0 || position > checkpoint.evidence_through.sequence {
            return Err(invalid("native page cursor exceeds its evidence boundary"));
        }
        Ok(Self { identity, after: position })
    }

    pub fn position(&self) -> u64 {
        self.after
    }

    pub fn memory() -> usize {
        size_of::<String>() + TEXT
    }

    pub fn encode(&self, position: u64) -> Result<String> {
        if position == 0 || position <= self.after {
            return Err(invalid("native continuation did not advance"));
        }
        let mut bytes = [0u8; BYTES];
        bytes[0] = VERSION;
        bytes[1..33].copy_from_slice(&self.identity);
        bytes[33..].copy_from_slice(&position.to_be_bytes());
        let mut text = String::new();
        text.try_reserve_exact(TEXT).map_err(|_| exhausted())?;
        if text.capacity() > TEXT {
            return Err(exhausted());
        }
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode_string(bytes, &mut text);
        if text.len() != TEXT || text.capacity() > TEXT {
            return Err(exhausted());
        }
        Ok(text)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn decode(after: &str) -> Result<([u8; 32], u64)> {
    if after.len() != TEXT {
        return Err(invalid("native page cursor length differs"));
    }
    let mut bytes = [0u8; BYTES];
    let length = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode_slice(after, &mut bytes)
        .map_err(|_| invalid("invalid native page cursor"))?;
    if length != BYTES || bytes[0] != VERSION {
        return Err(invalid("invalid native page cursor version"));
    }
    let identity = bytes[1..33].try_into().expect("fixed native cursor identity");
    let position = u64::from_be_bytes(bytes[33..].try_into().expect("fixed native cursor position"));
    if position == 0 {
        return Err(invalid("native page cursor is not a saved admission"));
    }
    Ok((identity, position))
}

fn identity(checkpoint: &ExecutionCheckpoint, kind: &str, source: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    for text in [
        "dokime.native-evidence-page.v1",
        kind,
        source,
        checkpoint.execution_id.as_str(),
        checkpoint.position.history.store.as_str(),
        checkpoint.position.history.epoch.as_str(),
        checkpoint.evidence_through.history.store.as_str(),
        checkpoint.evidence_through.history.epoch.as_str(),
    ] {
        digest.update((text.len() as u64).to_be_bytes());
        digest.update(text.as_bytes());
    }
    // Validation Binds The Opaque Version Injectively To This History And Position.
    digest.update(checkpoint.position.sequence.to_be_bytes());
    digest.update(checkpoint.evidence_through.sequence.to_be_bytes());
    digest.finalize().into()
}

fn invalid(message: &'static str) -> Error {
    Error::InvalidInput {
        field: "checkpoint evidence cursor",
        reason: message,
    }
}

fn exhausted() -> Error {
    Error::ResourceExhausted { resource: "native page cursor" }
}

/* === Tests === */
