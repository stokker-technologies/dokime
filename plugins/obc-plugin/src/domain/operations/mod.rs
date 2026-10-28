/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod connections;
pub mod device;
pub mod readiness;
pub mod status;

pub use connections::ListConnections;
pub use device::{ReadDevice, SetPower};
pub use readiness::AskReadiness;
pub use status::GetStatus;

use std::time::Duration;

use dokime::harmos::operations::{OperationPolicy, RetryPolicy};

// The policy every bench operation declares.
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_millis(2_000),
    attempt_timeout: Duration::from_millis(2_000),
    retries: RetryPolicy::Never,
};

// The bench operations this plugin declares; each request is its operation.
#[dokime::catalog]
pub enum Operations {
    ReadDevice(ReadDevice),
    SetPower(SetPower),
    GetStatus(GetStatus),
    AskReadiness(AskReadiness),
    ListConnections(ListConnections),
}
