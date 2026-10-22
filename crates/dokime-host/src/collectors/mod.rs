/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod bluetooth;
pub mod network;
pub mod system;
pub mod wifi;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    process::{Command, Output, Stdio},
    thread,
    time::Duration,
};

// Third Party Imports
use tokio::io::{AsyncRead, AsyncReadExt};

// Local Imports
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn integer(value: u64) -> Value {
    Value::Integer(value.min(i64::MAX as u64) as i64)
}

pub(crate) fn maybe_insert(values: &mut std::collections::BTreeMap<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        values.insert(key.to_owned(), value);
    }
}

pub(crate) fn run_command_bounded(command: Command, timeout: Duration) -> std::io::Result<Output> {
    // Collectors can run inside an existing Tokio runtime. Keep this synchronous
    // boundary on its own joined thread so block_on never nests runtimes.
    thread::Builder::new()
        .name("host-probe".to_owned())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
            runtime.block_on(async move {
                let mut command = tokio::process::Command::from(command);
                command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
                let mut child = command.spawn()?;
                let stdout = child.stdout.take().expect("stdout is piped");
                let stderr = child.stderr.take().expect("stderr is piped");
                let result = capture(stdout, stderr, child.wait(), timeout).await;
                if result.is_err() {
                    // This also waits for the direct child, avoiding zombies.
                    // The async pipe readers are cancelled, so inherited handles
                    // in descendants cannot strand blocking reader threads.
                    let _ = child.kill().await;
                }
                result
            })
        })?
        .join()
        .map_err(|_| std::io::Error::other("host probe worker panicked"))?
}

/* === Private Functions === */

async fn capture(
    mut stdout: impl AsyncRead + Unpin, mut stderr: impl AsyncRead + Unpin,
    wait: impl std::future::Future<Output = std::io::Result<std::process::ExitStatus>>, timeout: Duration,
) -> std::io::Result<Output> {
    let capture = async {
        let mut stdout_bytes = Vec::new();
        let mut stderr_bytes = Vec::new();
        let (_, _, status) = tokio::try_join!(stdout.read_to_end(&mut stdout_bytes), stderr.read_to_end(&mut stderr_bytes), wait,)?;
        Ok(Output { status, stdout: stdout_bytes, stderr: stderr_bytes })
    };
    tokio::time::timeout(timeout, capture)
        .await
        .unwrap_or_else(|_| Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "command timed out")))
}

/* === Tests === */
