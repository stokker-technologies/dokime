/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod aggregate;

pub use aggregate::Aggregate;

use super::Endpoint;
use super::ORDINARY;
use dokime_domain::model::query::wire::*;

#[harmos::catalog]
pub enum Operations {
    Aggregate(Aggregate),
}

impl Endpoint for Aggregate {
    const PATH: &'static str = "/dokime.v1.Query/Aggregate";
}
