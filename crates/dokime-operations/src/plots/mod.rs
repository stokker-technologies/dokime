/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod list_analyses;
mod query_snapshots;
mod watch;

pub use dokime_domain::model::document::DownsampleTechnique;
pub use list_analyses::ListAnalyses;
pub use query_snapshots::QuerySnapshots;
pub use watch::Watch;

use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;

#[allow(unused_imports)]
use dokime_domain::model::catalog::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::definition::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;
use dokime_domain::model::plots::wire::*;

#[harmos::catalog]
pub enum Operations {
    ListAnalyses(ListAnalyses),
    QuerySnapshots(QuerySnapshots),
    Watch(Watch),
}

impl Endpoint for ListAnalyses {
    const PATH: &'static str = "/dokime.v1.Plots/ListAnalyses";
}

impl Endpoint for QuerySnapshots {
    const PATH: &'static str = "/dokime.v1.Plots/QuerySnapshots";
}

impl Endpoint for Watch {
    const PATH: &'static str = "/dokime.v1.Plots/Watch";
}
