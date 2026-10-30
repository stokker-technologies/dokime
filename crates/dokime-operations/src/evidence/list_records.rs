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

#[harmos::operation(resources = [crate::hosting::Host<ListRecords>], response = RecordPage, id = "dokime.evidence.list_records",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Evidence", method = "ListRecords"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListRecords {
    pub selection: Option<CapturedRecordSelection>,

    #[row(
        description = "Both absent starts replay at zero through the current applied frontier. Continuations require BOTH cursors returned in the previous page."
    )]
    pub after: Option<Cursor>,

    pub through: Option<Cursor>,

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
