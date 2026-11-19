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
use dokime_protocol::host::CyclePhase;
use crate::{Result, execution_manager::ExecutionManager, host::lock};
use dokime_protocol::model::origin;
use dokime_storage::{ExecutionCut, ExecutionState};

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
        let plugin_run = is_plugin_run(&cut);
        let active = lock(&self.inner.active)?;
        let running = active.get(execution).map(|active| active.control.is_running()).transpose()?.unwrap_or(false);
        let active = active.contains_key(execution);
        let sealed_cycle = self
            .inner
            .store
            .client_cycle(&cut.execution.cycle)?
            .is_some_and(|cycle| cycle.sealed || cycle.phase == CyclePhase::Sealing as i32);
        Ok(ExecutionCapabilities {
            start: !sealed_cycle && can_start(&cut, active),
            rerun: plugin_run && cut.state == ExecutionState::Sealed && !active,
            cancel: plugin_run && running,
            inspect: true,
            export: cut.state == ExecutionState::Sealed,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn can_start(cut: &ExecutionCut, active: bool) -> bool {
    let plugin_run = is_plugin_run(cut);
    let unstarted = cut.state == ExecutionState::Open && cut.frontier == 0;
    plugin_run && unstarted && !active
}

fn is_plugin_run(cut: &ExecutionCut) -> bool {
    matches!(cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()), Some(origin::Kind::PluginRun(_)))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::can_start;
    use dokime_protocol::model::{Origin, origin};
    use dokime_storage::{Execution, ExecutionCut, ExecutionState};

    #[test]
    fn start_requires_unstarted_open_plugin_evidence_without_an_active_run() {
        let mut cut = ExecutionCut {
            execution: Execution {
                origin: Some(Origin {
                    kind: Some(origin::Kind::PluginRun(Default::default())),
                }),
                ..Default::default()
            },
            state: ExecutionState::Open,
            frontier: 0,
            seal: Vec::new(),
        };
        assert!(can_start(&cut, false));
        assert!(!can_start(&cut, true));
        cut.frontier = 1;
        assert!(!can_start(&cut, false));
        cut.frontier = 0;
        for state in [ExecutionState::Closed, ExecutionState::Sealed] {
            cut.state = state;
            assert!(!can_start(&cut, false));
        }
        cut.state = ExecutionState::Open;
        cut.execution.origin = Some(Origin {
            kind: Some(origin::Kind::Import(Default::default())),
        });
        assert!(!can_start(&cut, false));
        cut.execution.origin = Some(Origin::default());
        assert!(!can_start(&cut, false));
        cut.execution.origin = None;
        assert!(!can_start(&cut, false));
    }
}
