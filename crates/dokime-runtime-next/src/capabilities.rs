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
use crate::{Cycle, Error, Host, Result, host::lock};
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

impl Host {
    pub fn capabilities(&self, execution: &str) -> Result<ExecutionCapabilities> {
        let cut = self.inner.store.cut(execution)?;
        let plugin_run =
            matches!(cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()), Some(v1::origin::Kind::PluginRun(_)));
        let active = lock(&self.inner.active)?.contains_key(execution);
        Ok(ExecutionCapabilities {
            start: plugin_run && cut.state == ExecutionState::Open && cut.frontier == 0 && !active,
            rerun: plugin_run && cut.state == ExecutionState::Sealed && !active,
            cancel: plugin_run && active,
            inspect: true,
            export: cut.state == ExecutionState::Sealed,
        })
    }
    pub async fn start_execution(&self, execution: &str) -> Result<Cycle> {
        if !self.capabilities(execution)?.start {
            return Err(Error::Invalid("Start Execution: Select Unstarted Open PluginRun Evidence".into()));
        }
        crate::execution::start_existing(self.clone(), execution).await
    }
    pub async fn rerun(&self, execution: &str) -> Result<Cycle> {
        if !self.capabilities(execution)?.rerun {
            return Err(Error::Invalid("Rerun Execution: Select a Sealed PluginRun Execution".into()));
        }
        let origin = self
            .inner
            .store
            .cut(execution)?
            .execution
            .origin
            .ok_or_else(|| Error::Invalid("Rerun Execution: Supply Its Frozen Origin".into()))?;
        self.start_cycle(vec![origin]).await
    }
    pub fn export_execution(&self, execution: &str) -> Result<Vec<u8>> {
        if !self.capabilities(execution)?.export {
            return Err(Error::Invalid("Export Execution: Seal the Execution Before Exporting Evidence".into()));
        }
        Ok(self.inner.store.export(execution)?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
