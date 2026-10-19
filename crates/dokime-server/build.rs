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
    println!("cargo::rerun-if-changed=build.rs");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("Cargo sets OUT_DIR for build scripts")?);
    let contracts = harmos::operations::Contracts::of::<dokime_domain::operations::Operations>()?;
    harmos_packaging::proto::generate(&contracts, &output.join("grpc").join("operations"))?;
    Ok(())
}

/* === Tests === */
