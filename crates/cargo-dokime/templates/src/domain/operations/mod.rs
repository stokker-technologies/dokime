/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::operations::{chamber, status};
use dokime_runtime::harmos::operations::{OperationPolicy, RestartPolicy, RetryPolicy};

/* === Constants === */
/// The policy every bench operation declares.
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
/// The operations this plugin provides, named by `provides` and bound as served.
#[dokime_runtime::catalog]
pub enum Operations {
    GetStatus(status::GetStatus),
    ReadChamber(chamber::ReadChamber),
    DetectPort(chamber::DetectPort),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
