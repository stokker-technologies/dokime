/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod source_identity {
    include!("../../build_identity.rs");
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env, fs,
    path::{Path, PathBuf},
};

// Third Party Imports
use serde_json::Value;

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
    println!("cargo:rerun-if-changed=build.rs");
    source_identity::emit();
    let manifest = env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory");
    let manifest = PathBuf::from(manifest);

    // The Shell Names the Product from the Generated Brand, Like the Web UI
    let label = brand_text("default-organization.json", "label");
    println!("cargo:rustc-env=DOKIME_BRAND_LABEL={label}");

    // Track Added and Removed Files When the Frontend Is Embedded.
    if !tauri_build::is_dev() {
        let frontend = Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
        println!("cargo:rerun-if-changed={}", frontend.display());
    }
    embed_organization(&manifest);
    tauri_build::build();

    // Tauri embeds its manifest in binaries only. Library test executables also
    // load native dialogs, which require Common Controls v6 before startup.
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
        // A side-by-side `.manifest` file stays behind when nextest archives the test executable,
        // so every target, including library unit tests, embeds its own manifest.
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        // The Binary Already Links Tauri's Manifest Resource; Disable A Second Linker-Generated One.
        println!("cargo:rustc-link-arg-bin=dokime-gui=/MANIFEST:NO");
    }
}

fn brand_text(file: &str, field: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../packages/document/src/generated/organization").join(file);
    println!("cargo:rerun-if-changed={}", path.display());
    let bytes = fs::read(&path).expect("generated brand file");
    let manifest: Value = serde_json::from_slice(&bytes).expect("generated brand JSON");
    manifest[field].as_str().filter(|text| !text.trim().is_empty()).expect("generated brand text field").to_owned()
}

fn embed_organization(manifest: &Path) {
    let root = manifest.join("../../../organization").canonicalize().expect("organization directory");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut pending = vec![root.clone()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).expect("organization entries") {
            let entry = entry.expect("organization file");
            let path = entry.path();
            if entry.file_type().expect("organization file type").is_dir() {
                pending.push(path);
            } else {
                let relative = path.strip_prefix(&root).expect("pack file").to_string_lossy().replace('\\', "/");
                files.push(format!("({relative:?}, include_bytes!({:?}) as &[u8]),", path.to_str().expect("pack path")));
            }
        }
    }
    files.sort();
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("build output"));
    fs::write(output.join("organization.rs"), format!("&[{}]", files.join("\n"))).expect("embedded organization");
}

/* === Tests === */
