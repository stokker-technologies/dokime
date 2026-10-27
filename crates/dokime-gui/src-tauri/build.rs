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
    let label = brand_text("default-organization.json", "label");
    println!("cargo:rustc-env=DOKIME_BRAND_LABEL={label}");

    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR for build scripts"));
    let contracts = dokime::harmos::operations::Contracts::of::<dokime::operations::Operations>()
        .expect("canonical desktop transfer contracts");
    harmos_packaging::proto::generate(&contracts, &output.join("grpc").join("operations")).expect("desktop transfer descriptors");
    tauri_build::build();

    // Tauri embeds its manifest in binaries only. Library test executables also
    // load native dialogs, which require Common Controls v6 before startup.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
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
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/generated/organization").join(file);
    println!("cargo:rerun-if-changed={}", path.display());
    let bytes = std::fs::read(&path).expect("generated brand file");
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).expect("generated brand JSON");
    manifest[field].as_str().filter(|text| !text.trim().is_empty()).expect("generated brand text field").to_owned()
}

/* === Tests === */
