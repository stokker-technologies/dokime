/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// `RunProcess` served in the plugin process: the child runs in the
// execution's artifact folder with the execution's identity in its
// environment, each output line reaches the execution log unless redirected
// into a produced artifact, and its exit is recorded as `process.exited`.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::Stdio;
use std::time::{Duration, Instant};

// Third Party Imports
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::watch;

// Local Imports
use super::Binding;
use crate::execution::log::Level;
use crate::host::ProcessExited;
use crate::records::DeclaredRecord;
use dokime_domain::model::process::wire::DEFAULT_GRACE_MS;
use dokime_domain::model::process::wire::ProcessExit;
use dokime_operations::process::RunProcess;

/* === Constants === */

// How long the output readers may outlast the child, for a grandchild that
// holds a pipe open.

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn run(binding: Binding, request: RunProcess, mut cancel: watch::Receiver<bool>) -> Result<ProcessExit, String> {
    if request.program.is_empty() {
        return Err("a process needs a program".into());
    }

    // Run In The Artifact Folder With The Execution's Identity
    let artifacts = binding.artifacts.clone();
    let folder = tokio::task::spawn_blocking(move || artifacts.root())
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| format!("the execution's artifact folder: {error}"))?;
    let cwd = request.cwd.clone().map_or_else(|| folder.clone(), std::path::PathBuf::from);
    let mut command = tokio::process::Command::new(&request.program);
    command.args(&request.args).current_dir(&cwd).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    // The plugin's own connection to its host is never handed down.
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("HARMOS_") {
            command.env_remove(name);
        }
    }
    let step = binding.handles.step().unwrap_or_default();
    let identity = &binding.identity;
    command
        .env("DOKIME_EXECUTION_ID", &identity.execution_id)
        .env("DOKIME_CYCLE_ID", &identity.cycle_id)
        .env("DOKIME_TEMPLATE_ID", &identity.template_id)
        .env("DOKIME_PLUGIN_ID", &identity.plugin_id)
        .env("DOKIME_STEP_ID", &step.step_id)
        .env("DOKIME_ATTEMPT", &step.attempt)
        .env("DOKIME_ARTIFACTS_DIR", &folder);
    for variable in &request.env {
        command.env(&variable.name, &variable.value);
    }

    // Spawn, Then Route Each Output
    let started = Instant::now();
    let mut child = command.spawn().map_err(|error| format!("{} could not start: {error}", request.program))?;
    let stdout = route(&binding, child.stdout.take(), request.stdout_artifact.clone(), Level::Info)?;
    let stderr = route(&binding, child.stderr.take(), request.stderr_artifact.clone(), Level::Warn)?;

    // Wait For The Exit, Or Terminate On Cancellation
    let grace = Duration::from_millis(request.grace_ms.unwrap_or(DEFAULT_GRACE_MS).into());
    let (status, cancelled) = tokio::select! {
        status = child.wait() => (status, false),
        () = cancelled(&mut cancel) => (terminate(&mut child, grace).await, true),
    };
    let duration = started.elapsed();
    for reader in [stdout, stderr] {
        let mut reader = reader;
        if tokio::time::timeout(Duration::from_secs(2), &mut reader).await.is_err() {
            reader.abort();
        }
    }
    let status = status.map_err(|error| format!("{} could not be awaited: {error}", request.program))?;

    // Record The Exit
    let exit = ProcessExit {
        exit_code: status.code(),
        duration_ms: u64::try_from(duration.as_millis()).unwrap_or(u64::MAX),
        cancelled,
    };
    binding.journal.record(
        ProcessExited {
            program: request.program,
            exit_code: exit.exit_code.map(i64::from),
            duration_ms: i64::try_from(exit.duration_ms).unwrap_or(i64::MAX),
            cancelled,
        }
        .request(),
    );
    Ok(exit)
}

/* === Private Functions === */

// Pumps one output into the execution log line by line, or into a produced
// artifact byte for byte.
fn route(
    binding: &Binding, output: Option<impl AsyncRead + Unpin + Send + 'static>, artifact: Option<String>, level: Level,
) -> Result<tokio::task::JoinHandle<()>, String> {
    let journal = binding.journal.clone();
    let Some(mut output) = output else {
        return Ok(tokio::spawn(async {}));
    };
    let Some(artifact) = artifact else {
        return Ok(tokio::spawn(async move {
            let mut lines = BufReader::new(output).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                journal.log(level, line);
            }
        }));
    };
    let read_bytes = 64 * 1024;
    let mut writer = binding.artifacts.create(&artifact).map_err(|error| format!("artifact {artifact:?}: {error}"))?;
    Ok(tokio::spawn(async move {
        let mut buffer = vec![0; read_bytes];
        loop {
            match output.read(&mut buffer).await {
                Ok(0) => break,
                Ok(read) => {
                    let written = writer.write_all(&buffer[..read]).and_then(|()| writer.flush());
                    if let Err(error) = written {
                        journal.log(Level::Error, format!("artifact {artifact:?} stopped taking process output: {error}"));
                        writer.abandon();
                        return;
                    }
                }
                Err(error) => {
                    journal.log(Level::Warn, format!("process output for {artifact:?} ended early: {error}"));
                    break;
                }
            }
        }
        if let Err(error) = writer.finish() {
            journal.log(Level::Error, format!("artifact {artifact:?}: {error}"));
        }
    }))
}

async fn cancelled(cancel: &mut watch::Receiver<bool>) {
    if cancel.wait_for(|cancelled| *cancelled).await.is_err() {
        // The handle is gone without asking; the process runs to its exit.
        std::future::pending::<()>().await;
    }
}

// SIGTERM, a grace period, then the kill; Windows has only the last.
async fn terminate(child: &mut Child, grace: Duration) -> std::io::Result<std::process::ExitStatus> {
    #[cfg(unix)]
    if let Some(pid) = child.id().and_then(|pid| i32::try_from(pid).ok()) {
        // SAFETY: `kill` takes plain integers; the pid is our own unreaped child.
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
        if let Ok(status) = tokio::time::timeout(grace, child.wait()).await {
            return status;
        }
    }
    #[cfg(not(unix))]
    let _ = grace;
    child.start_kill()?;
    child.wait().await
}

/* === Tests === */
