/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{
    process::{Command, Output},
    time::Duration,
};

use crate::{Error, Result};

pub fn run_bounded(command: Command, timeout: Duration) -> Result<String> {
    let output = crate::collectors::run_command_bounded(command, timeout).map_err(|error| Error::Command(error.to_string()))?;
    text_output(output)
}

fn text_output(output: Output) -> Result<String> {
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(Error::Command(if stderr.is_empty() { format!("command exited with status {}", output.status) } else { stderr }))
}
