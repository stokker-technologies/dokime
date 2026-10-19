/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution mode and step commands

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::identity::StepId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    #[default]
    Automatic,
    Interactive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum StepCommand {
    Next,
    RunStep { step_id: StepId },
    Finish,
}

/* === Traits === */

/* === Implementations === */
impl ExecutionMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Interactive => "interactive",
        }
    }
}

impl harmos_domain::MemorySize for StepCommand {
    fn memory_size(&self) -> usize {
        match self {
            Self::RunStep { step_id } => {
                let owned = harmos_domain::MemorySize::memory_size(step_id);
                if owned == usize::MAX {
                    return usize::MAX;
                }
                std::mem::size_of::<Self>().saturating_add(owned.saturating_sub(std::mem::size_of_val(step_id)))
            }
            Self::Next | Self::Finish => std::mem::size_of::<Self>(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
