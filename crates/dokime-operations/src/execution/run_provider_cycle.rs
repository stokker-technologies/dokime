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
use harmos::operations::OperationPolicy;

// Local Imports

/* === Constants === */

// Bounds One Provider Cycle Run, Which Ends When Its Work Settles Or Is Cancelled.
const PROVIDER: OperationPolicy = OperationPolicy {
    cancellation_timeout: crate::STOP_SETTLEMENT.timeout,
    ..crate::policy(Duration::from_secs(86_400))
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The Portable Identity Of One Provider Cycle Run. The Host Resolves Its
// Process-Local Invocation From The Request ID.
#[harmos::operation(resources = [crate::hosting::Host<RunProviderCycle>], id = "dokime.execution.provider.run", policy = PROVIDER)]
#[derive(Clone, PartialEq)]
pub struct RunProviderCycle {
    pub request_id: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(RunProviderCycle);
}
