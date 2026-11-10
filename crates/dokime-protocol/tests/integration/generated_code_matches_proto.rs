/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Checked-In Wire Bindings.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fs, path::Path};

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

/* === Tests === */

#[test]
fn generated_code_matches_proto() -> Result<(), Box<dyn std::error::Error>> {
    // Regenerate Into Disposable Output.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output_root = root.join("target/generated/rust/dokime-protocol");
    fs::create_dir_all(&output_root)?;
    let output = tempfile::tempdir_in(output_root)?;
    crate::protocol::generate(&root, output.path())?;

    // Compare Both Packages, Retaining Evidence on Failure.
    for (generated, committed) in [("dokime.v1.rs", "src/v1.rs"), ("harmos.v1.rs", "src/harmos/v1.rs")] {
        let actual = fs::read(output.path().join(generated))?;
        let expected = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(committed))?;
        if actual != expected {
            let output = output.keep();
            panic!("generated Rust differs from {committed}; run mise run generate -- --protocol; evidence: {}", output.display());
        }
    }
    Ok(())
}
