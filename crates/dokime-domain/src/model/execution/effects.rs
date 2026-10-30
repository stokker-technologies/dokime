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

use crate::model::{execution::RecordScope, time::Timestamp};
use serde::{Deserialize, Serialize};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Pending External Changes Live In The Backend Journal Until Their Inverse Succeeds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingEffect {
    pub ticket: String,
    pub scope: RecordScope,
    pub source: String,
    pub resource: String,
    pub inverse_operation: String,
    pub inverse: Vec<u8>,
    pub description: String,
    pub prepared_at: Timestamp,
    pub applied_at: Option<Timestamp>,
    pub coalesced: Vec<EffectPreparation>,
}

// Later Calls Keep Their Own Confirmation Metadata, Never Another Inverse.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectPreparation {
    pub ticket: String,
    pub source: String,
    pub description: String,
    pub prepared_at: Timestamp,
    pub applied_at: Option<Timestamp>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
