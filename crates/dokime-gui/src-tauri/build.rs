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
fn main() {
    // The Shell Names The Product From The Generated Brand, Like The Web UI
    let label = brand_text("default-brand.json", "label");
    println!("cargo:rustc-env=DOKIME_BRAND_LABEL={label}");
    let bundle = brand_text("tauri.brand.json", "productName");
    println!("cargo:rustc-env=DOKIME_BUNDLE_NAME={bundle}");

    let artifacts = dokime_domain::operations::artifacts().expect("canonical desktop transfer contracts");
    artifacts.export_build("operations").expect("desktop transfer descriptors");
    tauri_build::build();

    // Tauri embeds its manifest in binaries only. Library test executables also
    // load native dialogs, which require Common Controls v6 before startup.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
}

/// One nonblank text field of a generated brand file, tracked for rebuilds.
fn brand_text(file: &str, field: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../branding/generated").join(file);
    println!("cargo:rerun-if-changed={}", path.display());
    let bytes = std::fs::read(&path).expect("generated brand file");
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).expect("generated brand JSON");
    manifest[field].as_str().filter(|text| !text.trim().is_empty()).expect("generated brand text field").to_owned()
}

/* === Tests === */
