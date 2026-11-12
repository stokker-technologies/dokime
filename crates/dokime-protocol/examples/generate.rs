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

#[path = "../tests/support/protocol.rs"]
mod protocol;

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

/* === Private Functions === */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = tempfile::tempdir()?;
    protocol::generate(&root, output.path())?;
    for (generated, committed) in [
        ("dokime.v1.rs", "src/v1.rs"),
        ("descriptors.bin", "src/descriptors.bin"),
        ("reflection.rs", "src/reflection.rs"),
        ("sources.rs", "src/sources.rs"),
    ] {
        let bytes = std::fs::read(output.path().join(generated))?;
        let destination = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(committed);
        if std::fs::read(&destination).ok().as_deref() != Some(bytes.as_slice()) {
            std::fs::write(destination, bytes)?;
        }
    }
    Ok(())
}

/* === Tests === */
