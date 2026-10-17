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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Exact inspected sealed revision and captured evidence in one native history.
// Reading this identity never creates or publishes a newer checkpoint.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    pub execution_identifier: String,
    pub position: Option<StorageCursor>,
    // Original canonical Harmos ObjectVersionToken bytes.
    #[row(description = "Original canonical Harmos ObjectVersionToken bytes.")]
    pub version: Vec<u8>,
    pub evidence_through: Option<StorageCursor>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageCursor {
    pub history: Option<StoreHistory>,
    pub sequence: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreHistory {
    pub store: String,
    pub epoch: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
