/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Child Processes: Every Host Probe Command Starts Here and Ends Within Its Timeout

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

// Third Party Imports
use tokio::io::{AsyncRead, AsyncReadExt};

// Local Imports

/* === Constants === */

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// A Child Process Command That Opens No Console Window.
#[allow(clippy::disallowed_methods, reason = "the host plugin starts its probes here without the runtime crate")]
pub fn command(program: &str) -> Command {
    #[cfg(windows)]
    {
        let mut command = Command::new(program);
        std::os::windows::process::CommandExt::creation_flags(&mut command, CREATE_NO_WINDOW);
        command
    }
    #[cfg(not(windows))]
    {
        Command::new(program)
    }
}

/// Run a Command With Null Input, Capturing Its Output Within the Timeout and Killing It After.
///
/// Collectors call this from blocking tasks; a joined thread with its own runtime never nests runtimes.
pub fn bounded_output(command: Command, timeout: Duration) -> io::Result<Output> {
    std::thread::Builder::new()
        .name("host-probe".to_owned())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
            runtime.block_on(async move {
                let mut command = tokio::process::Command::from(command);
                command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
                let mut child = command.spawn()?;
                let stdout = child.stdout.take().expect("Standard Output Is Piped");
                let stderr = child.stderr.take().expect("Standard Error Is Piped");
                let result = capture(stdout, stderr, child.wait(), timeout).await;
                if result.is_err() {
                    // Killing Also Waits for the Direct Child; the Cancelled Readers Strand No Threads.
                    let _ = child.kill().await;
                }
                result
            })
        })?
        .join()
        .map_err(|_| io::Error::other("Host Probe Worker Panicked"))?
}

/* === Private Functions === */

async fn capture(
    mut stdout: impl AsyncRead + Unpin, mut stderr: impl AsyncRead + Unpin,
    wait: impl std::future::Future<Output = io::Result<std::process::ExitStatus>>, timeout: Duration,
) -> io::Result<Output> {
    let capture = async {
        let mut stdout_bytes = Vec::new();
        let mut stderr_bytes = Vec::new();
        let (_, _, status) = tokio::try_join!(stdout.read_to_end(&mut stdout_bytes), stderr.read_to_end(&mut stderr_bytes), wait)?;
        Ok(Output {
            status,
            stdout: stdout_bytes,
            stderr: stderr_bytes,
        })
    };
    tokio::time::timeout(timeout, capture).await.unwrap_or_else(|_| Err(io::Error::new(io::ErrorKind::TimedOut, "Command Timed Out")))
}

/* === Tests === */
