/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<WatchRecords>], response = (), output = RecordPage, id = "dokime.evidence.watch_records",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Evidence", method = "WatchRecords"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchRecords {
    pub selection: Option<CapturedRecordSelection>,

    #[row(description = "Absent replays from zero, then follows; present resumes captured facts.")]
    pub after: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(WatchRecords);
}
