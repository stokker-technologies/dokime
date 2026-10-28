/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod hub_log;
mod terminal;

fn command() -> std::process::Command {
    let mut command = std::process::Command::new(crate::support::artifacts::host_binary());
    command.arg("server");
    command
}
