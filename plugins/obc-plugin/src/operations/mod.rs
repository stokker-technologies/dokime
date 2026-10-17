/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod connections;
pub mod readiness;
pub mod status;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime_runtime::harmos::operations::{OperationPolicy, RestartPolicy, RetryPolicy};

/* === Constants === */
pub(crate) const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_millis(2_000),
    attempt_timeout: Duration::from_millis(2_000),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[dokime_runtime::catalog]
pub enum Implemented {
    GetStatus(status::GetStatus),
    AskReadiness(readiness::AskReadiness),
    ListConnections(connections::ListConnections),
}

#[dokime_runtime::catalog]
pub enum Operations {
    #[nested]
    Implemented(Implemented),
    #[nested]
    Analyses(crate::analyses::Implemented),
    Observe(crate::observers::controls::Observe),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
