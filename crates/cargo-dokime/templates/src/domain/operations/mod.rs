/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


pub mod chamber;
pub mod status;


pub use chamber::{DetectPort, ReadChamber};
pub use status::GetStatus;


use std::time::Duration;


use dokime::harmos::operations::{OperationPolicy, RetryPolicy};


// The policy every bench operation declares.
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_millis(2_000),
    attempt_timeout: Duration::from_millis(2_000),
    retries: RetryPolicy::Never,
};


// The operations this plugin declares in CATALOGS; each request is its operation.
#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Operations {
    GetStatus(GetStatus),
    ReadChamber(ReadChamber),
    DetectPort(DetectPort),
}
