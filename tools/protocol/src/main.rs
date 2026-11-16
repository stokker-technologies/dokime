/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Generate the Checked-In Protocol Bindings.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;
use std::fs;
use std::path::Path;

// Third Party Imports

// Local Imports
use dokime_protocol_generator::protocol;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = tempfile::tempdir()?;
    protocol::generate(&root, output.path())?;
    let generated_root = root.join("crates/dokime-protocol/src/generated");
    fs::create_dir_all(&generated_root)?;
    // Remove Outputs Whose Contract Package Was Deleted.
    for entry in fs::read_dir(&generated_root)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "rs") && !output.path().join(path.file_name().unwrap()).is_file() {
            fs::remove_file(path)?;
        }
    }
    for entry in fs::read_dir(output.path())? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("Generated Filename Is Not UTF-8")?;
        if name.starts_with("harmos.") {
            continue;
        }
        let destination = match name {
            "descriptors.bin" | "reflection.rs" | "sources.rs" => root.join("crates/dokime-protocol/src").join(name),
            _ => generated_root.join(name),
        };
        let bytes = fs::read(entry.path())?;
        if fs::read(&destination).ok().as_deref() != Some(bytes.as_slice()) {
            fs::write(destination, bytes)?;
        }
    }
    Ok(())
}

/* === Tests === */
