/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod list_analyses;
mod query_snapshots;
mod watch;

/* === Re-Exports === */

pub use dokime_domain::model::document::DownsampleTechnique;
pub use list_analyses::ListAnalyses;
pub use query_snapshots::QuerySnapshots;
pub use watch::Watch;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::FOLLOW;
use super::ORDINARY;

use dokime_domain::model::plots::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ListAnalyses(ListAnalyses),
    QuerySnapshots(QuerySnapshots),
    Watch(Watch),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
