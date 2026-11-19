/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compile the Canonical Generated SDK Transport Behind an Unpublished Crate

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use std::{env, fs, path::PathBuf};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo Manifest Directory"));
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo Build Output"));
    for (source, destination) in [("resources.rs", "client_resources.rs"), ("types.rs", "client_types.rs"), ("schemas.rs", "client_schemas.rs")] {
        let path = manifest.join("../../sdk/rust/src/client").join(source);
        println!("cargo:rerun-if-changed={}", path.display());
        fs::copy(path, output.join(destination)).expect("Compile the Canonical Internal Client Source");
    }
}

/* === Tests === */
