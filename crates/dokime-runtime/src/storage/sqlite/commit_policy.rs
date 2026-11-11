/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SQLite Commit Group Bounds And Maintenance Cadence

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Bounds Count Admitted Rows And Encoded Request Bytes; No Timer Delays A Lone Writer.
pub struct StorageCommitPolicy {
    pub rows: NonZeroUsize,
    pub bytes: NonZeroUsize,
    pub checkpoint_groups: NonZeroUsize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn defaults() -> StorageCommitPolicy {
    StorageCommitPolicy {
        rows: NonZeroUsize::new(65_536).unwrap(),
        bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
        checkpoint_groups: NonZeroUsize::new(64).unwrap(),
    }
}

/* === Private Functions === */

/* === Tests === */
