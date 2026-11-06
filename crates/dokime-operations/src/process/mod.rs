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
use super::*;
use dokime_domain::model::process::wire::*;

/* === Constants === */

const POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
    cancellation_timeout: crate::STOP_SETTLEMENT.timeout,
    ..FOLLOW
};

/* === Macros === */

#[cfg(feature = "runtime")]
crate::hosted!(RunProcess);

/* === Type Aliases === */

/* === Structs === */

// Shared, so a template reaches it through `ctx.host()` without listing it.
#[harmos::operation(resources = [crate::hosting::Host<RunProcess>], response = ProcessExit, id = "dokime.process.run",
        policy = POLICY, visibility = harmos::operations::Visibility::Public)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunProcess {
    pub program: String,
    pub args: Vec<String>,

    #[row(description = "Set over the plugin's own environment and the execution identity.")]
    pub env: Vec<ProcessVariable>,

    #[row(description = "Absent runs in the execution's artifact directory.")]
    pub cwd: Option<String>,

    #[row(description = "Absent writes each line to the execution log at info; present streams the bytes into this produced artifact.")]
    pub stdout_artifact: Option<String>,

    #[row(description = "Absent writes each line to the execution log at warning; present streams the bytes into this produced artifact.")]
    pub stderr_artifact: Option<String>,

    #[row(description = "Milliseconds between SIGTERM and the kill when cancelled: default 2000.")]
    pub grace_ms: Option<u32>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RunProcess {
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
        self.env.push(ProcessVariable { name: name.into(), value: value.into() });
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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
