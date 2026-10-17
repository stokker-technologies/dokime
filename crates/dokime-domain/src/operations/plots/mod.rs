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
pub use crate::model::document::DownsampleTechnique;
pub use list_analyses::ListAnalyses;
pub use query_snapshots::QuerySnapshots;
pub use watch::Watch;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Access;
use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;
#[allow(unused_imports)]
use crate::model::catalog::wire::*;
#[allow(unused_imports)]
use crate::model::definition::wire::*;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

use crate::model::plots::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// The native bridge and server reject a request with more than 32 plots. The
// reply contains repeated envelopes in one bounded message; callers must
// lower max_points or split a batch on exhaustion.

// Watch is server-streaming. To change the request set, cancel this stream and
// open another one. Requests without a cursor receive an initial Snapshot
// before the watch begins; later frames are coalesced by the server at 100 ms
// only after a runtime state or telemetry watch advances.

// Analysis IDs Are Open; Each Contribution Declares Its Parameters And Outputs.

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    ListAnalyses(ListAnalyses),
    QuerySnapshots(QuerySnapshots),
    Watch(Watch),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListAnalyses {
    const PATH: &'static str = "/dokime.v1.Plots/ListAnalyses";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for QuerySnapshots {
    const PATH: &'static str = "/dokime.v1.Plots/QuerySnapshots";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for Watch {
    const PATH: &'static str = "/dokime.v1.Plots/Watch";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
