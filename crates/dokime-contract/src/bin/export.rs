/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Maintainer exporter that writes the operation contract artifacts into this build's output directory.

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

/* === Private Functions === */
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if !arguments.is_empty() {
        return Err(
            "the exporter takes no arguments; output belongs to the selected Cargo build".into(),
        );
    }
    let artifacts = dokime_contract::operations::artifacts()?;
    let directory = std::path::Path::new(env!("OUT_DIR"));
    let output = artifacts.export(directory, "operations")?;
    println!("{}", output.display());
    Ok(())
}

/* === Tests === */
