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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::Access;
use super::Endpoint;
use super::ORDINARY;
use crate::model::execution::wire_cursor::Cursor;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

use crate::model::notes::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Replaces an unsealed note's message. Only its author may edit it.

/* === Enums === */

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
