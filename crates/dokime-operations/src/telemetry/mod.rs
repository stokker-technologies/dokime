/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod list_samples;
mod watch_samples;

pub use list_samples::ListSamples;
pub use watch_samples::WatchSamples;

use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;
use dokime_domain::model::telemetry::wire::*;

#[harmos::catalog]
pub enum Operations {
    ListSamples(ListSamples),
    WatchSamples(WatchSamples),
}

impl Endpoint for ListSamples {
    const PATH: &'static str = "/dokime.v1.Telemetry/ListSamples";
}

impl Endpoint for WatchSamples {
    const PATH: &'static str = "/dokime.v1.Telemetry/WatchSamples";
}
