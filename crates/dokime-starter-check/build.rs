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
use std::{fs, path::Path};
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
    let templates = Path::new("../cargo-dokime/templates/src");
    println!("cargo:rerun-if-changed={}", templates.display());
    let output = std::env::var_os("OUT_DIR").expect("Cargo supplies OUT_DIR");
    render(templates, &Path::new(&output).join("src"));
}

fn render(source: &Path, output: &Path) {
    fs::create_dir_all(output).expect("create rendered starter directory");
    for entry in fs::read_dir(source).expect("read shipped starter") {
        let entry = entry.expect("read template entry");
        let path = entry.path();
        let destination = output.join(entry.file_name());
        if path.is_dir() {
            render(&path, &destination);
            continue;
        }
        let mut text = fs::read_to_string(&path)
            .expect("read starter source")
            .replace("{{owner}}", "starter-check")
            .replace("{{package}}", "dokime-starter-check")
            .replace("{{crate}}", "dokime_starter_check");
        // include! places the root items directly in this crate; inner module
        // comments are ordinary comments there, while nested files stay intact.
        if entry.file_name() == "lib.rs" || entry.file_name() == "main.rs" {
            text = text
                .lines()
                .map(|line| {
                    line.strip_prefix("//!")
                        .map_or_else(|| line.to_owned(), |doc| format!("//{doc}"))
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
        fs::write(destination, text).expect("write rendered starter source");
    }
}

/* === Tests === */
