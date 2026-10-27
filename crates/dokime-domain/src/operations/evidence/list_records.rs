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

#[harmos_domain::operation(response = RecordPage, id = "dokime.evidence.list_records",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
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

    // Nonzero asks for the tail: the last `latest` matching records up to the
    // current frontier, oldest first, in one complete page whose cursors sit at
    // that frontier. Valid only with both cursors absent; follow with
    // WatchRecords after the returned cursor.
    #[row(
        description = "Nonzero asks for the tail: the last `latest` matching records up to the current frontier, oldest first, in one complete page whose cursors sit at that frontier. Valid only with both cursors absent; follow with WatchRecords after the returned cursor."
    )]
    pub latest: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
