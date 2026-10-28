/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-env=DOKIME_NATIVE_TARGET={}", env::var("TARGET").expect("Cargo supplies TARGET"));

    if env::var_os("CARGO_FEATURE_HOST").is_some() {
        embed_report_template();
    }
    if env::var_os("CARGO_FEATURE_CONFIGURATION").is_some() {
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
    assert!(bundle.join("organization.toml").is_file(), "organization bundle has no manifest");

    // Collect The Bundle Files
    println!("cargo:rerun-if-changed={}", bundle.display());
    let mut entries = Vec::new();
    collect_assets(&bundle, &mut entries);
    entries.sort();
    let mut files = Vec::new();
    for path in entries {
        let relative = path.strip_prefix(&bundle).expect("asset stays within bundle");
        let name = relative.to_str().expect("UTF-8 name").replace('\\', "/");
        files.push((name, path));
    }

    // Emit The Embedded Table
    let mut table = String::from("static FILES: &[(&str, &[u8])] = &[\n");
    for (relative, path) in &files {
        table.push_str(&format!("    ({relative:?}, include_bytes!({:?})),\n", path.display().to_string()));
    }
    table.push_str("];\n");
    let output = Path::new(&env::var_os("OUT_DIR").expect("Cargo supplies it")).join("bundled.rs");
    fs::write(output, table).expect("write the embedded brand table");
}

fn collect_assets(directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(directory).unwrap_or_else(|error| panic!("organization folder {}: {error}", directory.display()));
    for entry in entries {
        let entry = entry.expect("organization entry");
        let path = entry.path();
        let kind = entry.file_type().expect("organization asset kind");
        assert!(!kind.is_symlink(), "organization assets may not be links");
        if kind.is_dir() {
            collect_assets(&path, files);
        } else if kind.is_file() {
            files.push(path);
        }
    }
}
