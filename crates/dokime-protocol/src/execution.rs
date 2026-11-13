/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Validate Frozen Execution Origins and Plugin Call Envelopes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::dokime::plugin::v1::{CriteriaEvaluateRequest, ExecutionsCleanupRequest, ExecutionsInitializeRequest, StepsExecuteRequest};
use crate::v1::{Origin, PluginRun, TestCase, origin};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PluginRun {
    /// Check the frozen identity before a plugin execution call is admitted.
    pub fn validate(&self) -> Result<(), String> {
        if self.test_case.is_none() {
            return Err("plugin-run origin requires its frozen TestCase".into());
        }
        if self.plugin.trim().is_empty() || self.template.trim().is_empty() || self.digest.is_empty() {
            return Err("plugin-run origin requires plugin, template and catalog digest".into());
        }
        let started_at = self.started_at.as_ref().ok_or("plugin-run origin requires the plugin process start time")?;
        if !(-62_135_596_800..=253_402_300_799).contains(&started_at.seconds) || !(0..1_000_000_000).contains(&started_at.nanos) {
            return Err("plugin-run origin has an invalid protobuf process start time".into());
        }
        Ok(())
    }
}

impl ExecutionsInitializeRequest {
    /// Validate the execution scope and agreement with the frozen origin.
    pub fn validate_origin(&self) -> Result<&PluginRun, String> {
        validate_call(&self.execution, self.test_case.as_ref(), self.origin.as_ref())
    }
}

impl StepsExecuteRequest {
    /// Validate the execution scope and agreement with the frozen origin.
    pub fn validate_origin(&self) -> Result<&PluginRun, String> {
        validate_call(&self.execution, self.test_case.as_ref(), self.origin.as_ref())
    }
}

impl CriteriaEvaluateRequest {
    /// Validate the execution scope and agreement with the frozen origin.
    pub fn validate_origin(&self) -> Result<&PluginRun, String> {
        validate_call(&self.execution, self.test_case.as_ref(), self.origin.as_ref())
    }
}

impl ExecutionsCleanupRequest {
    /// Validate the execution scope and agreement with the frozen origin.
    pub fn validate_origin(&self) -> Result<&PluginRun, String> {
        validate_call(&self.execution, self.test_case.as_ref(), self.origin.as_ref())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn validate_call<'a>(execution: &str, test_case: Option<&TestCase>, origin: Option<&'a Origin>) -> Result<&'a PluginRun, String> {
    if execution.trim().is_empty() {
        return Err("plugin execution call requires an execution identifier".into());
    }
    let origin = origin.ok_or("plugin execution call requires its frozen origin")?;
    let Some(origin::Kind::PluginRun(run)) = &origin.kind else {
        return Err("plugin execution call requires a PluginRun origin; imports cannot invoke plugin execution operations".into());
    };
    run.validate()?;
    let test_case = test_case.ok_or("plugin execution call requires its full TestCase")?;
    if run.test_case.as_ref() != Some(test_case) {
        return Err("plugin execution call TestCase differs from its frozen origin".into());
    }
    Ok(run)
}

/* === Tests === */
