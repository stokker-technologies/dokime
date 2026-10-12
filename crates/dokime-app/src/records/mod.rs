/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod resources;

/* === Re-Exports === */
pub use dokime::records::*;
pub use resources::{ResourceLifecycle, ResourceRecord, ResourceRegistry};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use dokime::core::{RecordId, RecordScope};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecords {
    pub record_visibility: Vec<RecordVisibility>,
    pub session: dokime::records::TestRecordSet,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordVisibility {
    pub record_id: RecordId,
    pub scope: Option<RecordScope>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
