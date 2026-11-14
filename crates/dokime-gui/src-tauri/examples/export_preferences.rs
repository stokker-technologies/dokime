/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_runtime_next::services::initial_preferences;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<(), String> {
    let directory = PathBuf::from(std::env::args_os().nth(1).ok_or("Export Preferences: Supply an Output Directory")?);
    let snapshot = initial_preferences();
    std::fs::create_dir_all(&directory).map_err(|error| format!("Export Preferences: Create {}: {error}", directory.display()))?;
    let output = directory.join("defaults.bin");
    std::fs::write(&output, snapshot.encode_to_vec())
        .map_err(|error| format!("Export Preferences: Write {}: {error}", output.display()))?;
    Ok(())
}
/* === Tests === */
