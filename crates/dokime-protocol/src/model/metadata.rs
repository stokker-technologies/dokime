/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Metadata

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::Metadata;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn from_label(label: String) -> Metadata {
    let mut identifier = String::new();
    for word in label.split(|character: char| !character.is_alphanumeric()).filter(|word| !word.is_empty()) {
        if !identifier.is_empty() {
            identifier.push('.');
        }
        identifier.push_str(&word.to_lowercase());
    }
    Metadata {
        identifier,
        label,
        description: String::new(),
        ..Default::default()
    }
}

/* === Private Functions === */

/* === Tests === */
