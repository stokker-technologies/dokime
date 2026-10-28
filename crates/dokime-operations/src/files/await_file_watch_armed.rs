/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<AwaitFileWatchArmed>], response = FileWatchArmed, id = "dokime.files.await_file_watch_armed",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Shared)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AwaitFileWatchArmed {
    pub instance_id: u64,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(AwaitFileWatchArmed);
}
