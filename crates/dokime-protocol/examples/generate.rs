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
    for (generated, committed) in [("dokime.v1.rs", "src/v1.rs"), ("harmos.v1.rs", "src/harmos/v1.rs")] {
        let bytes = std::fs::read(output.path().join(generated))?;
        std::fs::write(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(committed), bytes)?;
    }
    Ok(())
}

/* === Tests === */
