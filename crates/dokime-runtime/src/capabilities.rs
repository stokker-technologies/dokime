/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Action Capabilities

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, execution_manager::ExecutionManager, host::lock};
use dokime_protocol::v1;
use dokime_storage::ExecutionState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionCapabilities {
    pub start: bool,
    pub rerun: bool,
    pub cancel: bool,
    pub inspect: bool,
    pub export: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionManager {
    pub fn capabilities(&self, execution: &str) -> Result<ExecutionCapabilities> {
        let cut = self.inner.store.cut(execution)?;
        let plugin_run =
            matches!(cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()), Some(v1::origin::Kind::PluginRun(_)));
        let active = self.is_active(execution)?;
        let running = lock(&self.inner.active)?.get(execution).map(|active| active.control.is_running()).transpose()?.unwrap_or(false);
        Ok(ExecutionCapabilities {
            start: plugin_run && cut.state == ExecutionState::Open && cut.frontier == 0 && !active,
            rerun: plugin_run && cut.state == ExecutionState::Sealed && !active,
            cancel: plugin_run && running,
            inspect: true,
            export: cut.state == ExecutionState::Sealed,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
