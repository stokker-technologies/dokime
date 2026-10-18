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
// Sanitizes each folder and file segment of a validated relative artifact name.
pub(crate) fn safe_artifact_path(name: &str) -> String {
    name.split('/').map(safe_artifact_filename).collect::<Vec<_>>().join("/")
}

/* === Private Functions === */
fn safe_artifact_filename(filename: &str) -> String {
    let name = Path::new(filename).file_name().and_then(|name| name.to_str()).unwrap_or("artifact").trim();
    let path = Path::new(name);
    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("artifact").trim();
    let suffix = path.extension().and_then(|suffix| suffix.to_str()).map(|suffix| format!(".{suffix}")).unwrap_or_default();

    let safe_stem = slugify(stem);
    let safe_suffix =
        suffix.chars().filter(|character| character.is_ascii_alphanumeric() || *character == '.').take(24).collect::<String>();

    format!("{}{}", if safe_stem.is_empty() { "artifact" } else { &safe_stem }, safe_suffix)
}

fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut previous_dash = false;

    for character in value.trim().chars() {
        let character =
            if character.is_ascii_alphanumeric() || character == '.' || character == '_' { character.to_ascii_lowercase() } else { '-' };

        if character == '-' {
            if previous_dash {
                continue;
            }
            previous_dash = true;
        } else {
            previous_dash = false;
        }

        output.push(character);
    }

    output.trim_matches(&['.', '-', '_'][..]).to_owned()
}

/* === Tests === */
