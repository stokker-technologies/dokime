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
use serde::{Deserialize, Serialize};

// Local Imports
use super::FOLLOW;

/* === Constants === */
// The grace a cancelled process gets between SIGTERM and the kill.
pub const DEFAULT_GRACE_MS: u32 = 2_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessVariable {
    pub name: String,
    pub value: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunProcessRequest {
    pub program: String,
    pub args: Vec<String>,
    // Set over the plugin's own environment and the execution identity.
    #[row(description = "Set over the plugin's own environment and the execution identity.")]
    pub env: Vec<ProcessVariable>,
    // Absent runs in the execution's artifact directory.
    #[row(description = "Absent runs in the execution's artifact directory.")]
    pub cwd: Option<String>,
    // Absent writes each line to the execution log at info; present streams the bytes into this produced artifact.
    #[row(
        description = "Absent writes each line to the execution log at info; present streams the bytes into this produced artifact."
    )]
    pub stdout_artifact: Option<String>,
    // Absent writes each line to the execution log at warning; present streams the bytes into this produced artifact.
    #[row(
        description = "Absent writes each line to the execution log at warning; present streams the bytes into this produced artifact."
    )]
    pub stderr_artifact: Option<String>,
    // Milliseconds between SIGTERM and the kill when cancelled: default 2000.
    #[row(description = "Milliseconds between SIGTERM and the kill when cancelled: default 2000.")]
    pub grace_ms: Option<u32>,
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

// Shared, so a template reaches it through `ctx.host()` without listing it.
#[harmos::operation(request = RunProcessRequest, response = ProcessExit, id = "dokime.process.run", version = 1,
        policy = FOLLOW, visibility = harmos::operations::Visibility::Shared)]
pub struct RunProcess;

/* === Enums === */
// Where one output of the process goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    Log,
    Artifact(String),
}

/* === Traits === */

/* === Implementations === */
impl RunProcessRequest {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: Vec::new(),
            cwd: None,
            stdout_artifact: None,
            stderr_artifact: None,
            grace_ms: None,
        }
    }

    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    #[must_use]
    pub fn env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push(ProcessVariable {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    #[must_use]
    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    #[must_use]
    pub fn stdout(mut self, output: Output) -> Self {
        self.stdout_artifact = output.artifact();
        self
    }

    #[must_use]
    pub fn stderr(mut self, output: Output) -> Self {
        self.stderr_artifact = output.artifact();
        self
    }

    #[must_use]
    pub fn grace(mut self, grace: std::time::Duration) -> Self {
        self.grace_ms = Some(u32::try_from(grace.as_millis()).unwrap_or(u32::MAX));
        self
    }
}

impl Output {
    fn artifact(self) -> Option<String> {
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

/* === Tests === */
