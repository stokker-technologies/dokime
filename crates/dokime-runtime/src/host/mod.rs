/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Application Lifecycle Records, Independent Of The Host Telemetry Plugin.
pub use crate::records::host::{
    ProcessExited, StepHandleCancelled, TargetAcquired, TargetLost, TestCycleCompleted, ToolConnectionAcquired, ToolConnectionLost,
};
