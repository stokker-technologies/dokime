/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod create_note;
mod edit_note;
mod list_health;
mod list_notes;

/* === Re-Exports === */
pub use create_note::CreateNote;
pub use edit_note::EditNote;
pub use list_health::ListHealth;
pub use list_notes::ListNotes;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
#[allow(unused_imports)]
use super::common::*;
use super::cursor::Cursor;
use super::{Access, Endpoint, ORDINARY};

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
    // Absent selects all current annotations. Present selects this owner and descendants.
    #[row(
        description = "Absent selects all current annotations. Present selects this owner and descendants."
    )]
    pub owner: Option<AnnotationOwner>,
    // Also include ancestors, including session/global annotations.
    #[row(description = "Also include ancestors, including session/global annotations.")]
    pub include_context: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListAnnotationsRequest {
    pub selection: Option<AnnotationSelection>,
    // default 50, maximum 100
    #[row(description = "default 50, maximum 100")]
    pub limit: u32,
    pub after_id: String,
    // Exact runtime revision and selection, required with after_id. Restart on change.
    #[row(
        description = "Exact runtime revision and selection, required with after_id. Restart on change."
    )]
    pub cursor: Option<Cursor>,
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
pub struct CreateNoteRequest {
    // Explicit process epoch from discovery. Retry identities expire at restart.
    #[row(
        description = "Explicit process epoch from discovery. Retry identities expire at restart."
    )]
    pub epoch: String,
    pub mutation_id: String,
    pub owner: Option<AnnotationOwner>,
    // nonblank, at most 16 KiB UTF-8. Actor comes from authentication.
    #[row(description = "nonblank, at most 16 KiB UTF-8. Actor comes from authentication.")]
    pub message: String,
}

// Replaces an unsealed note's message. Only its author may edit it.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EditNoteRequest {
    // Explicit process epoch from discovery.
    #[row(description = "Explicit process epoch from discovery.")]
    pub epoch: String,
    pub note_id: String,
    // nonblank, at most 16 KiB UTF-8. Actor comes from authentication.
    #[row(description = "nonblank, at most 16 KiB UTF-8. Actor comes from authentication.")]
    pub message: String,
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
    // Info, Warn or Error
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

#[harmos::catalog]
pub enum Operations {
    ListNotes(ListNotes),
    CreateNote(CreateNote),
    EditNote(EditNote),
    ListHealth(ListHealth),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListNotes {
    const PATH: &'static str = "/dokime.v1.Notes/ListNotes";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for CreateNote {
    const PATH: &'static str = "/dokime.v1.Notes/CreateNote";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for EditNote {
    const PATH: &'static str = "/dokime.v1.Notes/EditNote";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for ListHealth {
    const PATH: &'static str = "/dokime.v1.Health/ListHealth";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
