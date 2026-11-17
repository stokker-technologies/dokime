/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export Complete Default Preferences

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::env;
use std::fs;
use std::path::PathBuf;

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_operations::host::settings::get;

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
    let directory = env::args_os().nth(1).ok_or("Export Preferences: Supply an Output Directory")?;
    let directory = PathBuf::from(directory);
    let snapshot = get::initial_preferences();
    let bytes = snapshot.encode_to_vec();

    fs::create_dir_all(&directory).map_err(|error| format!("Export Preferences: Create {}: {error}", directory.display()))?;
    let output = directory.join("defaults.bin");
    fs::write(&output, bytes).map_err(|error| format!("Export Preferences: Write {}: {error}", output.display()))?;
    Ok(())
}

/* === Tests === */
