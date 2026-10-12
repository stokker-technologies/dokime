/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


/* === Modules === */
pub mod chamber;
pub mod status;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};

/* === Constants === */
pub(crate) const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_millis(2_000),
    attempt_timeout: Duration::from_millis(2_000),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[dokime::catalog]
pub enum Implemented {
    GetStatus(status::GetStatus),
    ReadChamber(chamber::ReadChamber),
    DetectPort(chamber::DetectPort),
}

#[dokime::catalog]
pub enum Operations {
    #[nested]
    Implemented(Implemented),
    Start(crate::observers::controls::Start),
    Stop(crate::observers::controls::Stop),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
