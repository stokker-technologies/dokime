/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Query Snapshot Contracts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::execution::wire_checkpoint::ExecutionCheckpoint;
use crate::model::identity::wire::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Tokens Are Opaque Authenticated References, Never Authorization Grants.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub token: String,
    pub identity: String,
    pub expires_at: Option<Timestamp>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SnapshotRef {
    Token(String),
    Checkpoint(ExecutionCheckpoint),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
