/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod chamber;
pub mod status;

/* === Re-Exports === */

pub use chamber::{DetectPort, ReadChamber};
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

// The operations this plugin declares in CATALOGS; each request is its operation.
#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Operations {
    GetStatus(GetStatus),
    ReadChamber(ReadChamber),
    DetectPort(DetectPort),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
