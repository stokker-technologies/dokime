/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The One Constructor For Child Processes.
//
// A Windows GUI-subsystem parent, such as the desktop, gives every console
// child it starts a visible console window of its own. Every Dokime spawn
// starts through `command`, which starts Windows children without one; Clippy's
// `disallowed-methods` refuses any other `Command::new`.

/* === Modules === */

#[cfg(not(windows))]
mod unix;
#[cfg(windows)]
mod windows;

/* === Re-Exports === */

#[cfg(windows)]
pub use windows::CREATE_NO_WINDOW;

/* === Imports === */

// Standard Library Imports
use std::{ffi::OsStr, process::Command};

// Third Party Imports

// Local Imports
#[cfg(not(windows))]
use unix as platform;
#[cfg(windows)]
use windows as platform;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// A Child Process Command That Opens No Console Window. Convert It With
// `tokio::process::Command::from` For Async Use; A Later `creation_flags`
// Call Replaces The Flag.
pub fn command(program: impl AsRef<OsStr>) -> Command {
    #[allow(clippy::disallowed_methods, reason = "the single process constructor")]
    let mut command = Command::new(program);
    platform::without_console(&mut command);
    command
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    #[test]
    fn command_keeps_the_program() {
        assert_eq!(super::command("dokime-probe").get_program(), "dokime-probe");
    }
}
