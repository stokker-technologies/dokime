/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File Capture Operations

/* === Modules === */

pub mod start;
pub mod stop;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Host-Owned Local File Capture Bounds; Requests Carry No Limits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureSettings {
    /// Directories Plugins May Capture From; Empty Refuses Every Local Capture.
    pub roots: Vec<PathBuf>,
    pub maximum_files: usize,
    pub maximum_bytes: u64,
    pub maximum_captures: usize,
    pub scan_entries: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            maximum_files: 64,
            maximum_bytes: 256 * 1024 * 1024,
            maximum_captures: 256,
            scan_entries: 100_000,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
