/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Packaging invariants of the scaffold templates the tool embeds.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

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

/* === Tests === */
#[test]
fn templates_avoid_manifests() {
    // Cargo treats a nested `Cargo.toml` as a separate package and drops the
    // template from the published crate.
    let templates = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
    let mut pending = vec![templates];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            assert_ne!(
                path.file_name().and_then(|name| name.to_str()),
                Some("Cargo.toml"),
                "{} must not be named like a real manifest",
                path.display()
            );
        }
    }
}
