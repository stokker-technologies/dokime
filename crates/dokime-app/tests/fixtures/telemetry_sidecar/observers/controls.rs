/* ***
 *
 * Dokime
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
use dokime::harmos::operations::observers::{Control, Status};
use dokime::harmos::operations::{
    OperationDeclaration, OperationPolicy, ReplaySafety, RetryPolicy, Visibility, WireOperation,
};
// Local Imports

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(1),
    attempt_timeout: Duration::from_secs(1),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Start;

pub struct Stop;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl OperationDeclaration for Start {
    type Request = Control;
    type Response = ();
    const ID: &'static str = "validation-telemetry.observers.start";
    const VISIBILITY: Visibility = Visibility::Host;
    const REPLAY: ReplaySafety = ReplaySafety::Idempotent;
    const POLICY: OperationPolicy = POLICY;
}

impl WireOperation for Start {
    const VERSION: u32 = 1;
}

impl OperationDeclaration for Stop {
    type Request = Control;
    type Response = Status;
    const ID: &'static str = "validation-telemetry.observers.stop";
    const VISIBILITY: Visibility = Visibility::Host;
    const REPLAY: ReplaySafety = ReplaySafety::Idempotent;
    const POLICY: OperationPolicy = POLICY;
}

impl WireOperation for Stop {
    const VERSION: u32 = 1;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
