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
        Ok((status, _, stderr)) => Err(Error::Process {
            program: name.to_owned(),
            status,
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        }),
        Err(error) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            Err(error)
        }
    }
}

async fn read(reader: impl AsyncRead + Unpin, limit: usize, label: String) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    reader.take(limit as u64 + 1).read_to_end(&mut bytes).await?;
    if bytes.len() > limit {
        return Err(Error::Output(label, limit));
    }
    Ok(bytes)
}

/* === Tests === */
