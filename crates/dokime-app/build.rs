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
use std::env;
use std::fs;
use std::path::PathBuf;
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
fn main() {
    println!(
        "cargo:rustc-env=DOKIME_NATIVE_TARGET={}",
        env::var("TARGET").expect("Cargo supplies TARGET")
    );

    embed_report_template();
}

fn embed_report_template() {
    let source = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR for build scripts"),
    )
    .join("dist-report/report.html");

    // Emitted unconditionally, including when the file does not exist yet:
    // that is what makes Cargo re-run this script once the report is built.
    println!("cargo:rerun-if-changed={}", source.display());

    if !source.is_file() {
        eprintln!(
            "the report template is not built; run `mise run prepare:report` (or `pnpm --dir crates/dokime-gui run build:report`)"
        );
        std::process::exit(1);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Cargo sets OUT_DIR for build scripts"));
    let destination = out_dir.join("report-template.html");

    fs::copy(&source, &destination).unwrap_or_else(|error| {
        panic!(
            "failed to copy the report template from {} to {}: {error}",
            source.display(),
            destination.display()
        )
    });
}

/* === Tests === */
