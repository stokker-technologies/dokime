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
use std::path::{Path, PathBuf};

// Third Party Imports

// Local Imports

/* === Constants === */
// The brand bundle folders the runtime embeds beside `organization.toml`.
const BRAND_FOLDERS: &[&str] = &["theme", "fonts", "logos"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() {
    println!("cargo:rustc-env=DOKIME_NATIVE_TARGET={}", env::var("TARGET").expect("Cargo supplies TARGET"));

    if env::var_os("CARGO_FEATURE_HOST").is_some() {
        embed_report_template();
        embed_brand_bundle();
    }
}

fn embed_report_template() {
    let source = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR for build scripts"))
        .join("dist-report/report.html");

    // Emitted unconditionally, including when the file does not exist yet:
    // that is what makes Cargo re-run this script once the report is built.
    println!("cargo:rerun-if-changed={}", source.display());

    if !source.is_file() {
        eprintln!("the report template is missing; run `mise run prepare:report` and commit the generated resources");
        std::process::exit(1);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Cargo sets OUT_DIR for build scripts"));
    let destination = out_dir.join("report-template.html");

    fs::copy(&source, &destination).unwrap_or_else(|error| {
        panic!("failed to copy the report template from {} to {}: {error}", source.display(), destination.display())
    });
}

fn embed_brand_bundle() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies it"));
    let bundle = manifest.join("dist-brand");
    let bundle = bundle.canonicalize().unwrap_or_else(|error| panic!("organization bundle {}: {error}", bundle.display()));

    // Collect The Bundle Files
    let mut files = vec![("organization.toml".to_owned(), bundle.join("organization.toml"))];
    for folder in BRAND_FOLDERS {
        let directory = bundle.join(folder);
        println!("cargo:rerun-if-changed={}", directory.display());
        let mut entries: Vec<_> = fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("organization folder {}: {error}", directory.display()))
            .map(|entry| entry.expect("organization entry").path())
            .filter(|path| path.is_file())
            .collect();
        entries.sort();
        for path in entries {
            let name = path.file_name().and_then(|name| name.to_str()).expect("UTF-8 name");
            files.push((format!("{folder}/{name}"), path));
        }
    }
    println!("cargo:rerun-if-changed={}", bundle.join("organization.toml").display());

    // Emit The Embedded Table
    let mut table = String::from("static FILES: &[(&str, &[u8])] = &[\n");
    for (relative, path) in &files {
        table.push_str(&format!("    ({relative:?}, include_bytes!({:?})),\n", path.display().to_string()));
    }
    table.push_str("];\n");
    let output = Path::new(&env::var_os("OUT_DIR").expect("Cargo supplies it")).join("bundled.rs");
    fs::write(output, table).expect("write the embedded brand table");
}

/* === Tests === */
