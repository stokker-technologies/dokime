/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::{Origin, PluginRun, TestCase, origin};

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
            return Err("provide the frozen TestCase in the plugin-run origin".into());
        }
        let missing_plugin = self.plugin.trim().is_empty();
        let missing_template = self.template.trim().is_empty();
        let missing_digest = self.digest.is_empty();
        if missing_plugin || missing_template || missing_digest {
            return Err("provide the plugin, template and catalog digest in the plugin-run origin".into());
        }
        let started_at = self.started_at.as_ref().ok_or("provide the plugin process start time in the plugin-run origin")?;
        let valid_seconds = (-62_135_596_800..=253_402_300_799).contains(&started_at.seconds);
        let valid_nanos = (0..1_000_000_000).contains(&started_at.nanos);
        if !valid_seconds || !valid_nanos {
            return Err("provide a valid protobuf process start time in the plugin-run origin".into());
        }
        Ok(())
    }
}

/* === Public Functions === */

pub(crate) fn validate_call<'a>(
    execution: &str, test_case: Option<&TestCase>, origin: Option<&'a Origin>,
) -> Result<&'a PluginRun, String> {
    if execution.trim().is_empty() {
        return Err("provide an execution identifier for the plugin call".into());
    }
    let origin = origin.ok_or("provide the frozen origin for the plugin execution call")?;
    let Some(origin::Kind::PluginRun(run)) = &origin.kind else {
        return Err("invoke plugin execution operations with a PluginRun origin; imported executions cannot invoke plugins".into());
    };
    run.validate()?;
    let test_case = test_case.ok_or("provide the full TestCase for the plugin execution call")?;
    if run.test_case.as_ref() != Some(test_case) {
        return Err("use the TestCase stored in the frozen plugin-run origin".into());
    }
    Ok(run)
}

/* === Private Functions === */

/* === Tests === */
