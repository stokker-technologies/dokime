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
#[harmos::operation(response = (), output = RecordPage, id = "dokime.evidence.watch_records", version = 1,
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Evidence", method = "WatchRecords"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchRecords {
    pub selection: Option<CapturedRecordSelection>,
    // Absent replays from zero, then follows; present resumes captured facts.
    #[row(description = "Absent replays from zero, then follows; present resumes captured facts.")]
    pub after: Option<Cursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
