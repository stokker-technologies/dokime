/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Steps

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::{PluginRun, execution};
use crate::plugin::StepsExecuteRequest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl StepsExecuteRequest {
    /// Validate the execution scope and agreement with the frozen origin.
    pub fn validate_origin(&self) -> Result<&PluginRun, String> {
        execution::validate_call(&self.execution, self.test_case.as_ref(), self.origin.as_ref())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
