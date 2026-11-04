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

// Local Imports
use super::ORDINARY;
use dokime_domain::model::execution::wire_cursor::Cursor;

use dokime_domain::model::notes::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

// Replaces an unsealed note's message. Only its author may edit it.

#[harmos::catalog]
pub enum Operations {
    ListNotes(ListNotes),
    CreateNote(CreateNote),
    EditNote(EditNote),
    ListHealth(ListHealth),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
