/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operation-Specific Mutation Identities

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;

// Third Party Imports
use sha2::{Digest, Sha256};
use uuid::{Builder, Uuid};

// Local Imports
use crate::{Error, Result, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct CaseMaterializeOptions {
    pub request_id: Option<String>,
    pub parameters: HashMap<String, Vec<Value>>,
}

#[derive(Default)]
pub struct CycleCreateOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct ExecutionCreateOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct ExecutionStartOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct ExecutionStopOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct ExecutionCancelOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct ExecutionSealOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct CycleSealOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct NoteCreateOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct ArtifactCreateOptions {
    pub request_id: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn request_id(value: Option<String>) -> Result<String> {
    let value = value.unwrap_or_else(|| Uuid::now_v7().to_string());
    if value.trim().is_empty() {
        return Err(Error::Invalid("Submit Mutation: Supply a Nonempty Request Identity".into()));
    }
    Ok(value)
}

pub(super) fn execution_id(request_id: &str) -> String {
    let digest = Sha256::digest(request_id.as_bytes());
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    Builder::from_custom_bytes(bytes).into_uuid().to_string()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{execution_id, request_id};
    use uuid::Uuid;

    #[test]
    fn replay_preserves_execution_identity_and_distinct_mutations_do_not_collide() {
        let identity = request_id(Some("run-one".into())).unwrap();
        let first = execution_id(&identity);
        let replay = execution_id(&request_id(Some("run-one".into())).unwrap());
        assert_eq!(first, replay);
        assert_ne!(first, execution_id("run-two"));
        assert!(Uuid::parse_str(&first).is_ok());
        assert!(request_id(Some(String::new())).is_err());
        assert!(request_id(Some("   ".into())).is_err());
    }
}
