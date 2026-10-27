/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod owner;

/* === Re-Exports === */

pub use owner::{Child, spawn, spawn_session};

/* === Imports === */

// Standard Library Imports
use std::process::Command;
use std::process::{Output, Stdio};
use std::time::Duration;

// Third Party Imports
use tokio::io::{AsyncRead, AsyncReadExt};

// Local Imports
use crate::error::Error;

/* === Constants === */

const PROCESS_OUTPUT_BYTES: u64 = 33_554_432;
const PROCESS_TAIL_BYTES: u64 = 65_536;
const TERMINAL_TICK_MS: u64 = 120;
const PROCESS_READ_BYTES: u64 = 4_096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn run(command: Command, name: &str, timeout: Option<Duration>, stdout: usize, stderr: usize) -> Result<Output, Error> {
    let mut command = tokio::process::Command::from(command);
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = spawn(command)?;
    let output = child.stdout().take().expect("stdout is piped");
    let diagnostics = child.stderr().take().expect("stderr is piped");
    let execution = async {
        tokio::try_join!(
            async { child.wait().await.map_err(Error::from) },
            read(output, stdout, format!("{name} stdout")),
            read(diagnostics, stderr, format!("{name} stderr")),
        )
    };
    let result = match timeout {
        Some(timeout) => {
            tokio::time::timeout(timeout, execution).await.map_err(|_| Error::Timeout(name.to_owned())).and_then(|result| result)
        }
        None => execution.await,
    };
    match result {
        Ok((status, stdout, stderr)) if status.success() => Ok(Output { status, stdout, stderr }),
        Ok((status, _, stderr)) => {
            Err(Error::Process { program: name.to_owned(), status, stderr: String::from_utf8_lossy(&stderr).into_owned() })
        }
        Err(error) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            Err(error)
        }
    }
}

pub(crate) async fn cargo(command: Command, name: &str, timeout: Option<Duration>, compilation: bool) -> Result<Output, Error> {
    let mut command = tokio::process::Command::from(command);
    use std::time::Instant;
    let terminal = cargo_terminal(&mut command, dokime::terminal::color_mode());
    let tick = Duration::from_millis(TERMINAL_TICK_MS);
    let progress = terminal.progress(name, tick);
    let started = Instant::now();
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = spawn(command)?;
    let stdout = child.stdout().take().expect("piped Cargo output");
    let stderr = child.stderr().take().expect("piped Cargo diagnostics");
    let execution = async {
        tokio::try_join!(
            async { child.wait().await.map_err(Error::from) },
            cargo_output(stdout, &progress, compilation, PROCESS_OUTPUT_BYTES as usize),
            cargo_diagnostics(stderr, &progress, PROCESS_TAIL_BYTES as usize, PROCESS_READ_BYTES as usize),
        )
    };
    let deadline = async {
        match timeout {
            Some(duration) => tokio::time::sleep(duration).await,
            None => std::future::pending().await,
        }
    };
    let result = tokio::select! {
        result = execution => result,
        _ = deadline => Err(Error::Timeout(name.to_owned())),
        _ = tokio::signal::ctrl_c() => Err(Error::Cancelled),
    };
    match result {
        Ok((status, stdout, stderr)) if status.success() => {
            progress.finish(format!("{name} completed in {:.1}s", started.elapsed().as_secs_f64()));
            Ok(Output { status, stdout, stderr })
        }
        Ok((status, _, _)) => {
            progress.fail(format!("{name} failed after {:.1}s", started.elapsed().as_secs_f64()));
            Err(Error::Process { program: name.to_owned(), status, stderr: "see compiler diagnostics above".into() })
        }
        Err(error) => {
            // The wrapper kills the entire Unix group / Windows Job Object and awaits exit.
            child.kill().await?;
            progress.fail(format!("{name}: {error}"));
            Err(error)
        }
    }
}

/* === Private Functions === */

async fn read(reader: impl AsyncRead + Unpin, limit: usize, label: String) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    reader.take(limit as u64 + 1).read_to_end(&mut bytes).await?;
    if bytes.len() > limit {
        return Err(Error::Output(label, limit));
    }
    Ok(bytes)
}

fn cargo_terminal(command: &mut tokio::process::Command, mode: dokime::terminal::ColorMode) -> dokime::terminal::Terminal {
    use dokime::terminal::{ColorMode, Terminal};
    let inherited = command
        .as_std()
        .get_envs()
        .find(|(key, _)| *key == "CARGO_TERM_COLOR")
        .map(|(_, value)| value.map(std::ffi::OsStr::to_owned))
        .unwrap_or_else(|| std::env::var_os("CARGO_TERM_COLOR"));
    if mode == ColorMode::Auto
        && let Some(value) = inherited
    {
        // Keep forced compiler ANSI through our renderer even when stderr is piped.
        return Terminal::new(match value.to_str() {
            Some("always") => ColorMode::Always,
            Some("never") => ColorMode::Never,
            _ => ColorMode::Auto,
        });
    }
    let terminal = Terminal::new(mode);
    command.env("CARGO_TERM_COLOR", if terminal.color_enabled() { "always" } else { "never" });
    terminal
}

async fn cargo_output(
    reader: impl AsyncRead + Unpin, progress: &dokime::terminal::Progress, compilation: bool, limit: usize,
) -> Result<Vec<u8>, Error> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut reader = BufReader::new(reader.take((limit as u64).saturating_add(1)));
    let mut output = Vec::new();
    loop {
        let start = output.len();
        if reader.read_until(b'\n', &mut output).await? == 0 {
            break;
        }
        if output.len() > limit {
            return Err(Error::Output("Cargo stdout".into(), limit));
        }
        if compilation {
            if let Ok(message) = serde_json::from_slice::<serde_json::Value>(&output[start..]) {
                if let Some(rendered) = message["message"]["rendered"].as_str() {
                    progress.log(rendered.trim_end());
                }
            } else {
                progress.log(String::from_utf8_lossy(&output[start..]).trim_end());
            }
        }
    }
    Ok(output)
}

async fn cargo_diagnostics(
    mut reader: impl AsyncRead + Unpin, progress: &dokime::terminal::Progress, tail: usize, read_bytes: usize,
) -> Result<Vec<u8>, Error> {
    // Retain only an error tail; long successful builds must not fail due to log volume.
    let mut output = Vec::new();
    let mut chunk = vec![0; read_bytes];
    let mut pending = Vec::new();
    loop {
        let count = reader.read(&mut chunk).await?;
        if count == 0 {
            if !pending.is_empty() {
                progress.log(String::from_utf8_lossy(&pending).trim_end());
            }
            break;
        }
        pending.extend_from_slice(&chunk[..count]);
        // Pipe reads can split a line or UTF-8 character anywhere. Preserve normal
        // compiler lines while bounding an unusually large unterminated message.
        let complete = pending
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map(|index| index + 1)
            .or_else(|| (pending.len() >= tail).then_some(pending.len()));
        if let Some(complete) = complete {
            progress.log(String::from_utf8_lossy(&pending[..complete]).trim_end());
            pending.drain(..complete);
        }
        output.extend_from_slice(&chunk[..count]);
        if output.len() > tail {
            output.drain(..output.len() - tail);
        }
    }
    Ok(output)
}

/* === Tests === */
