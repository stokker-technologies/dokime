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
use crate::model::execution::wire_cursor::Cursor;

use crate::model::identity::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Current owners only. An execution owner never falls back to its cycle.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationOwner {
    pub owner: Option<AnnotationOwnerOwner>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedCaseNoteOwner {
    pub cycle_id: String,
    pub case_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionNoteOwner {
    pub cycle_id: String,
    pub execution_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationSelection {
    #[row(description = "Absent selects all current annotations. Present selects this owner and descendants.")]
    pub owner: Option<AnnotationOwner>,

    #[row(description = "Also include ancestors, including session/global annotations.")]
    pub include_context: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatorNote {
    pub note_id: String,
    pub scope: Option<EvidenceScope>,
    pub author: Option<String>,
    pub message: String,
    pub created_at: Option<Timestamp>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListNotesResponse {
    pub notes: Vec<OperatorNote>,
    pub cursor: Option<Cursor>,
    pub next_after_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EditNoteResponse {
    pub note: Option<OperatorNote>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteReceipt {
    pub epoch: String,
    pub mutation_id: String,
    pub owner: Option<AnnotationOwner>,
    pub note: Option<OperatorNote>,
    pub checkpoint_targets: Vec<String>,
    pub publication: NotePublication,
    pub unconfirmed_targets: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthItem {
    pub health_item_id: String,
    pub scope: Option<EvidenceScope>,

    #[row(description = "Info, Warn or Error")]
    pub severity: Severity,

    pub message: String,
    pub detail: Option<String>,
    pub observed_at: Option<Timestamp>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListHealthResponse {
    pub items: Vec<HealthItem>,
    pub cursor: Option<Cursor>,
    pub next_after_id: String,
}

/* === Enums === */

// A committed note may have no existing persisted execution dataset. Confirmation
// verifies this exact note in all captured targets; it does not promise future
// history or create historical annotations. Retrying Create retries publication.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotePublication {
    Unspecified,
    NoExistingDatasets,
    Confirmed,
    Unconfirmed,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationOwnerOwner {
    CycleId(String),
    PlannedCase(PlannedCaseNoteOwner),
    Execution(ExecutionNoteOwner),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
