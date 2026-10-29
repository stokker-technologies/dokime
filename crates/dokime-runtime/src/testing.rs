/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Nondefault Fixture Surface For Canonical Integration And System Test Consumers

pub use crate::domains::execution::run::logs::testing::logs;
pub use crate::execution::evidence_rows::testing::capture;
pub use crate::report::query::plots::testing::report_points;
pub use crate::runtime::storage::telemetry::testing::telemetry;
pub use crate::transport::buffer::testing::{records, stream};
