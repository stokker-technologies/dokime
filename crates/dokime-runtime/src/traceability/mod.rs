/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[cfg(feature = "host")]
mod projection;

/* === Re-Exports === */
#[cfg(feature = "host")]
pub use projection::{
    TraceabilityPageBudget, bounded_host_traceability_snapshot_from_sample, execution_traceability,
    host_traceability_snapshot_from_samples, lease_traceability_snapshot, replace_snapshots_by_source, snapshots_from_parameter_scopes,
    snapshots_from_parameters, snapshots_from_state, stated_snapshots,
};
#[cfg(feature = "host")]
pub(crate) use projection::{bounded_snapshots_from_state, host_traceability_page_candidate};
#[cfg(feature = "host")]
pub(crate) use projection::{
    resource_traceability, scope_discriminator, snapshots_from_execution_parameters, stated_execution_memory, stated_execution_snapshots,
};

/* === Imports === */

/* === Constants === */

/* === Public Functions === */

/* === Private Functions === */
