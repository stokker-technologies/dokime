/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Full-Scale System Stories: The Large-Payload And Long-Running Sizes Of Paths Whose
// Small Deterministic Guards Run In The Integration And Acceptance Lanes.
#![recursion_limit = "256"]

mod execution_read_contention;
mod execution_sampling;
mod telemetry_plot_history;

#[path = "../support/services/mod.rs"]
pub mod service_support;
