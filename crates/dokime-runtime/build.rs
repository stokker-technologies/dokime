/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Require the Generated Report Template and Embed the Host Build Identity

/* === Modules === */

mod source_identity {
    include!("../build_identity.rs");
}

/* === Re-Exports === */

/* === Imports === */

use std::{env, path::Path};

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
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("Build Report Template: Cargo Must Supply Its Manifest Directory");
    let template = Path::new(&manifest).join("../../target/generated/report/report.html");
    println!("cargo:rerun-if-changed={}", template.display());
    assert!(
        template.is_file(),
        "Build Report Template: Run `mise run generate` Before Compiling dokime-runtime (missing {})",
        template.display()
    );
    source_identity::emit();
}

/* === Tests === */
