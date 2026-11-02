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
pub mod device;
pub mod readiness;
pub mod status;

/* === Re-Exports === */

pub use connections::ListConnections;
pub use device::{ReadDevice, SetPower};
pub use readiness::AskReadiness;
pub use status::GetStatus;

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};

/* === Constants === */

// The policy every bench operation declares.
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_millis(2_000),
    attempt_timeout: Duration::from_millis(2_000),
    cancellation_timeout: std::time::Duration::ZERO,
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

// The bench operations this plugin declares; each request is its operation.
#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Operations {
    ReadDevice(ReadDevice),
    SetPower(SetPower),
    GetStatus(GetStatus),
    AskReadiness(AskReadiness),
    ListConnections(ListConnections),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
