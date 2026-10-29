/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::time::Duration;

use harmos::operations::OperationPolicy;

// Bounds One Provider Cycle Run, Which Ends When Its Work Settles Or Is Cancelled.
const PROVIDER: OperationPolicy = crate::policy(Duration::from_secs(86_400));

// The Portable Identity Of One Provider Cycle Run. The Host Resolves Its
// Process-Local Invocation From The Request ID.
#[harmos::operation(resources = [crate::hosting::Host<RunProviderCycle>], id = "dokime.execution.provider.run", policy = PROVIDER)]
#[derive(Clone, PartialEq)]
pub struct RunProviderCycle {
    pub request_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(RunProviderCycle);
}
