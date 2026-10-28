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

// A zero limit means the server ceiling of 10,000 rows. Larger values are

// clamped to it rather than rejected. `group_by` may be empty, which

// aggregates the whole filtered selection into a single row. `aggregates` must

// name at least one output.

#[harmos::catalog]
pub enum Operations {
    Aggregate(Aggregate),
}

impl Endpoint for Aggregate {
    const PATH: &'static str = "/dokime.v1.Query/Aggregate";
}
