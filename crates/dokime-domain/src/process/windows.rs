/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{os::windows::process::CommandExt, process::Command};

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(super) fn without_console(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}
