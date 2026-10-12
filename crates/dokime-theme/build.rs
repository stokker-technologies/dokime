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
use std::{
    env, fs,
    path::{Path, PathBuf},
};
// Third Party Imports

// Local Imports

/* === Constants === */
const FOLDERS: &[&str] = &["theme", "fonts", "logos"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies it"));
    let bundle = manifest.join("../../branding");
    let bundle = bundle
        .canonicalize()
        .unwrap_or_else(|error| panic!("branding bundle {}: {error}", bundle.display()));

    // Collect The Bundle Files
    let mut files = vec![("brand.toml".to_owned(), bundle.join("brand.toml"))];
    for folder in FOLDERS {
        let directory = bundle.join(folder);
        println!("cargo:rerun-if-changed={}", directory.display());
        let mut entries: Vec<_> = fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("branding folder {}: {error}", directory.display()))
            .map(|entry| entry.expect("branding entry").path())
            .filter(|path| path.is_file())
            .collect();
        entries.sort();
        for path in entries {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("UTF-8 name");
            files.push((format!("{folder}/{name}"), path));
        }
    }
    println!(
        "cargo:rerun-if-changed={}",
        bundle.join("brand.toml").display()
    );

    // Emit The Embedded Table
    let mut table = String::from("static FILES: &[(&str, &[u8])] = &[\n");
    for (relative, path) in &files {
        table.push_str(&format!(
            "    ({relative:?}, include_bytes!({:?})),\n",
            path.display().to_string()
        ));
    }
    table.push_str("];\n");
    let output = Path::new(&env::var_os("OUT_DIR").expect("Cargo supplies it")).join("bundled.rs");
    fs::write(output, table).expect("write the embedded brand table");
}

/* === Tests === */
