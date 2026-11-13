/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Generate the Checked-In Protocol Bindings.

/* === Modules === */

mod protocol;

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
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = tempfile::tempdir()?;
    protocol::generate(&root, output.path())?;
    let generated_root = root.join("crates/dokime-protocol/src/generated");
    std::fs::create_dir_all(&generated_root)?;
    for entry in std::fs::read_dir(output.path())? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("Generated Filename Is Not UTF-8")?;
        if name.starts_with("harmos.") {
            continue;
        }
        let destination = match name {
            "descriptors.bin" | "reflection.rs" | "sources.rs" => root.join("crates/dokime-protocol/src").join(name),
            _ => generated_root.join(name),
        };
        let bytes = std::fs::read(entry.path())?;
        if std::fs::read(&destination).ok().as_deref() != Some(bytes.as_slice()) {
            std::fs::write(destination, bytes)?;
        }
    }
    let retired = root.join("crates/dokime-protocol/src/v1.rs");
    if retired.exists() {
        std::fs::remove_file(retired)?;
    }
    Ok(())
}

/* === Tests === */
