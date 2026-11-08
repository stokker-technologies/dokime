/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Format Identity And Refusal

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

// Third Party Imports

// Local Imports
use super::{Error, Result};

/* === Constants === */

/* === Public Functions === */

// Refuse Only The Obsolete Product State That This Runtime Cannot Replay.
// Evidence And Obsolete Source-Digest Markers Do Not Define Root Compatibility.
pub fn check(root: &Path) -> Result<()> {
    for name in ["harmos/journal.jsonl", "harmos/state.json"] {
        if root.join(name).exists() {
            return Err(Error::Incompatible(format!("{name} uses an unreadable built-in persistent-state encoding")));
        }
    }
    Ok(())
}
