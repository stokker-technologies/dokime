/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

// Copies the current plan, authored Definition/Operator cycle inputs, documents,
// and policy into a new ready cycle. No execution evidence or derived parameters
// are copied. The source remains untouched, including if currently running.
// Copy does not certify provider availability; Run validates that separately.
#[harmos::operation(resources = [crate::hosting::Host<CopyCycle>], response = CreateCycleReply, id = "dokime.execution.copy_cycle",
        policy = MUTATION, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "CopyCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CopyCycle {
    pub source_cycle_id: String,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(CopyCycle);
}
