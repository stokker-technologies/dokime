/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Nondefault Fixture Surface for Canonical Integration and System Test Consumers

/* === Modules === */

/* === Re-Exports === */
pub use crate::domains::execution::outputs::tail::testing::early_log_stop;
pub use crate::domains::execution::run::logs::testing::logs;
pub use crate::domains::execution::run::testing::{
    drain_counts, hold_execution_checkpoint, hold_execution_seal, hold_member_capture, hold_report, hold_report_job, hold_report_request,
    hold_report_wait, hold_stream_drain, hold_watch_seal,
};
pub use crate::execution::evidence_rows::testing::capture;
pub use crate::runtime::query::records::RecordQueryFixture;

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
