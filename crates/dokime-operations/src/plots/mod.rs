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
use super::Endpoint;
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

impl Endpoint for ListAnalyses {
    const PATH: &'static str = "/dokime.v1.Plots/ListAnalyses";
}

impl Endpoint for QuerySnapshots {
    const PATH: &'static str = "/dokime.v1.Plots/QuerySnapshots";
}

impl Endpoint for Watch {
    const PATH: &'static str = "/dokime.v1.Plots/Watch";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
