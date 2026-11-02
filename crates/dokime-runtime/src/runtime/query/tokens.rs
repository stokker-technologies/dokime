/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Authenticated Bounded Snapshot And Seek Tokens

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Local Imports
use super::history::Key;
use crate::storage::sqlite::snapshots::Cut;
use dokime_domain::model::{
    identity::wire::Timestamp,
    query::error::{Failure, Reason},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Tokens {
    pub key: [u8; 32],
    pub maximum: usize,
}

struct Buffer {
    bytes: Vec<u8>,
    maximum: usize,
}

/* === Enums === */

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Seek {
    History(Key),
    Evidence { position: u64, occurrence: String },
    Bucket { start: Timestamp, partition: String },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Token {
    Snapshot(Cut),
    Continuation { cut: Cut, selection: [u8; 32], last: Seek },
}

/* === Traits === */

/* === Implementations === */

impl Tokens {
    pub fn encode(&self, token: &Token, operation: &str) -> Result<String, Box<Failure>> {
        let bytes = bounded_json(token, self.maximum, operation)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.key).expect("fixed signing key");
        mac.update(b"dokime.query.token.v1\0");
        mac.update(&bytes);
        let result = format!("{}.{}", URL_SAFE_NO_PAD.encode(&bytes), URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()));
        if result.len() > self.maximum {
            return Err(Failure::new(Reason::ResourceLimit, operation, "Query token exceeds configured byte ceiling"));
        }
        Ok(result)
    }

    pub fn decode(&self, token: &str, operation: &str) -> Result<Token, Box<Failure>> {
        let invalid = || Failure::new(Reason::InvalidContinuation, operation, "Invalid authenticated query token");
        if token.is_empty() || token.len() > self.maximum {
            return Err(invalid());
        }
        let (body, tag) = token.split_once('.').ok_or_else(invalid)?;
        let body = URL_SAFE_NO_PAD.decode(body).map_err(|_| invalid())?;
        let tag = URL_SAFE_NO_PAD.decode(tag).map_err(|_| invalid())?;
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.key).expect("fixed signing key");
        mac.update(b"dokime.query.token.v1\0");
        mac.update(&body);
        mac.verify_slice(&tag).map_err(|_| invalid())?;
        serde_json::from_slice(&body).map_err(|_| invalid())
    }
}

impl std::io::Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("query encoding budget exceeded"));
        }
        self.bytes.try_reserve(bytes.len()).map_err(std::io::Error::other)?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn bounded_json(value: &impl Serialize, maximum: usize, operation: &str) -> Result<Vec<u8>, Box<Failure>> {
    let mut buffer = Buffer { bytes: Vec::new(), maximum };
    serde_json::to_writer(&mut buffer, value)
        .map_err(|_| Failure::new(Reason::ResourceLimit, operation, "Query encoding exceeds configured byte ceiling"))?;
    Ok(buffer.bytes)
}

// Page Sizes And Snapshot Spelling Do Not Change Selection. Alternatives And
// Conjunctions Are Sets; Projection And Reduction Output Order Remain Significant.
pub(super) fn binding(request: &impl Serialize, maximum: usize, operation: &str) -> Result<[u8; 32], Box<Failure>> {
    let bytes = bounded_json(request, maximum, operation)?;
    let mut value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| Failure::new(Reason::InvalidSelection, operation, "Invalid query request"))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("page");
        object.remove("at");
    }
    canonical(&mut value, None);
    let bytes = bounded_json(&value, maximum, operation)?;
    let mut hash = Sha256::new();
    hash.update(operation.as_bytes());
    hash.update([0]);
    hash.update(bytes);
    Ok(hash.finalize().into())
}

pub(super) fn identity(cut: &Cut) -> String {
    let bytes = serde_json::to_vec(cut).expect("bounded query cut");
    URL_SAFE_NO_PAD.encode(Sha256::digest(bytes))
}

/* === Private Functions === */

fn canonical(value: &mut serde_json::Value, name: Option<&str>) {
    match value {
        serde_json::Value::Object(object) => {
            for (name, value) in object {
                canonical(value, Some(name));
            }
        }
        serde_json::Value::Array(values) => {
            for value in values.iter_mut() {
                canonical(value, None);
            }
            if matches!(name, Some("template_ids" | "cycle_ids" | "statuses" | "phases" | "versions" | "where" | "predicates" | "values")) {
                values.sort_by_cached_key(|value| value.to_string());
                values.dedup();
            }
        }
        _ => {}
    }
}

/* === Tests === */
