/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Nondefault Fixture Surface For Canonical Integration And System Test Consumers

/* === Modules === */

/* === Re-Exports === */
pub use crate::domains::execution::outputs::tail::testing::early_log_stop;
pub use crate::domains::execution::run::logs::testing::logs;
pub use crate::domains::execution::run::testing::{drain_counts, hold_report};
pub use crate::execution::evidence_rows::testing::capture;

pub use crate::runtime::storage::captures::testing::joined_drains;
pub use crate::runtime::storage::telemetry::testing::telemetry;
pub use crate::transport::buffer::testing::{records, stream};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
