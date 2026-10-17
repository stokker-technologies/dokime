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
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[harmos::operation(response = RecordPage, id = "dokime.evidence.list_records", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Evidence", method = "ListRecords"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListRecords {
    pub selection: Option<CapturedRecordSelection>,
    // Both absent starts replay at zero through the current applied frontier.
    // Continuations require BOTH cursors returned in the previous page.
    #[row(
        description = "Both absent starts replay at zero through the current applied frontier. Continuations require BOTH cursors returned in the previous page."
    )]
    pub after: Option<Cursor>,
    pub through: Option<Cursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
