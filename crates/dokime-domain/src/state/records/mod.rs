/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Pure Record Collections And Their State Transitions.

/* === Modules === */
pub mod resources;
pub mod set;

/* === Re-Exports === */
pub use resources::ResourceRegistry;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

use serde::{Deserialize, Serialize};
// Local Imports
use crate::model::execution::RecordScope;
use crate::model::identity::RecordId;
use crate::model::records::Record;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecords {
    pub record_visibility: Vec<RecordVisibility>,
    pub session: crate::state::records::set::TestRecordSet,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordVisibility {
    pub record_id: RecordId,
    pub scope: Option<RecordScope>,
}

/* === Enums === */
#[harmos::records]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Records {
    Record(Record),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
