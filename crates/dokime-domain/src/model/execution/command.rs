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

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;
// Local Imports
use crate::model::identity::CorrelationId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandContext {
    pub correlation_id: Option<CorrelationId>,
    pub idempotency_key: Option<String>,
    pub submitted_by: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("idempotency key requires 1–256 UTF-8 bytes, nonblank and without control characters")]
pub struct IdempotencyKeyError;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn validate_idempotency_key(key: &str) -> Result<(), IdempotencyKeyError> {
    if key.is_empty()
        || key.len() > 256
        || key.trim().is_empty()
        || key.chars().any(char::is_control)
    {
        return Err(IdempotencyKeyError);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
