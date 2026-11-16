/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::{PluginRun, execution};
use crate::plugin::{ExecutionsCleanupRequest, ExecutionsInitializeRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionsInitializeRequest {
    /// Validate the execution scope and agreement with the frozen origin.
    pub fn validate_origin(&self) -> Result<&PluginRun, String> {
        execution::validate_call(&self.execution, self.test_case.as_ref(), self.origin.as_ref())
    }
}

impl ExecutionsCleanupRequest {
    /// Validate the execution scope and agreement with the frozen origin.
    pub fn validate_origin(&self) -> Result<&PluginRun, String> {
        execution::validate_call(&self.execution, self.test_case.as_ref(), self.origin.as_ref())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
