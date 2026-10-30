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
pub use crate::domains::execution::run::logs::testing::logs;
pub use crate::execution::evidence_rows::testing::capture;
pub use crate::report::query::plots::testing::report_points;
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

pub async fn commit<T>(
    runtime: &crate::RuntimeHandle, transaction: T,
) -> std::result::Result<harmos::state::Receipt, harmos::Error<T::Error>>
where
    T: harmos::state::Transaction<crate::state::RuntimeState> + Into<crate::transactions::Transactions>,
    T::Error: From<String>,
{
    runtime.commit(transaction).await
}

/* === Private Functions === */

/* === Tests === */
