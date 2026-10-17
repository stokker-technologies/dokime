/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A system process a step runs. The SDK serves it inside the plugin process,
// so its working directory, execution log, records and produced artifacts are
// the calling execution's own: called, it runs to its exit; submitted, its
// instance cancels it with SIGTERM, a grace period and a kill on Unix, and
// TerminateProcess on Windows.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;
// Local Imports

/* === Types === */

// The grace a cancelled process gets between SIGTERM and the kill.
pub const DEFAULT_GRACE_MS: u32 = 2_000;

#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessVariable {
    pub name: String,
    pub value: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessExit {
    // Absent when a signal ended the process.
    #[row(description = "Absent when a signal ended the process.")]
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    // The instance was cancelled before the process exited by itself.
    #[row(description = "The instance was cancelled before the process exited by itself.")]
    pub cancelled: bool,
}

// Where one output of the process goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    Log,
    Artifact(String),
}

impl Output {
    pub(crate) fn artifact(self) -> Option<String> {
        match self {
            Self::Log => None,
            Self::Artifact(name) => Some(name),
        }
    }
}

impl ProcessExit {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }

    pub fn duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.duration_ms)
    }
}

/* === Public Functions === */

/* === Private Functions === */
