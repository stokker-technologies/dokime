/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Deliver The Operator's Interrupt To An Armed Wait

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Resolve When The Operator Presses Ctrl+C, Or Ctrl+Break On Windows.
pub(crate) async fn requested() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        // A Process Created In A New Group Ignores Ctrl+C, And Its Descendants Inherit That Across
        // New Consoles Such As Pseudo-Terminals. An Armed Wait Advertises Ctrl+C, So It Restores
        // Processing; Ctrl+Break Reaches A Process Group Even Where Ctrl+C Cannot.
        // SAFETY: A Null Handler Only Clears This Process's Ignore-Ctrl+C Attribute.
        if unsafe { windows_sys::Win32::System::Console::SetConsoleCtrlHandler(None, 0) } == 0 {
            tracing::debug!(error = %std::io::Error::last_os_error(), "ctrl+c processing was not restored");
        }
        let mut ctrl_break = tokio::signal::windows::ctrl_break()?;
        tokio::select! {
            interrupted = tokio::signal::ctrl_c() => interrupted,
            Some(()) = ctrl_break.recv() => Ok(()),
        }
    }
    #[cfg(not(windows))]
    tokio::signal::ctrl_c().await
}

/* === Private Functions === */

/* === Tests === */
