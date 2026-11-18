/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Declaration Identifiers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

use std::any;

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

pub(crate) fn code_name<T>() -> String {
    let name = any::type_name::<T>().rsplit("::").next().unwrap_or_default();
    if name.contains("closure") || name.starts_with("fn(") {
        return String::new();
    }
    let name = name.strip_suffix("Template").or_else(|| name.strip_suffix("Suite")).unwrap_or(name);
    build_identifier("", name, true)
}

pub(crate) fn build_identifier(parent: &str, name: &str, generated: bool) -> String {
    let local = if generated {
        let characters = name.chars().collect::<Vec<_>>();
        let mut result = String::new();
        for (index, character) in characters.iter().copied().enumerate() {
            let previous = index.checked_sub(1).map(|index| characters[index]);
            let next = characters.get(index + 1);
            let boundary = character.is_uppercase()
                && previous.is_some_and(|previous| {
                    previous.is_lowercase()
                        || previous.is_numeric()
                        || (previous.is_uppercase() && next.is_some_and(|next| next.is_lowercase()))
                });
            if boundary && !result.ends_with('_') {
                result.push('_');
            }
            result.extend(character.to_lowercase());
        }
        result
    } else {
        name.into()
    };
    if parent.is_empty() { local } else { format!("{parent}.{local}") }
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{build_identifier, code_name};
    struct HTTPAccuracySuite;
    struct VoltageAccuracyTemplate;
    #[test]
    fn names_follow_code_boundaries_and_preserve_explicit_identifiers() {
        assert_eq!(code_name::<HTTPAccuracySuite>(), "http_accuracy");
        assert_eq!(code_name::<VoltageAccuracyTemplate>(), "voltage_accuracy");
        for name in ["ReadVoltage", "readVoltage", "read_voltage"] {
            assert_eq!(build_identifier("accuracy.voltage", name, true), "accuracy.voltage.read_voltage");
        }
        assert_eq!(build_identifier("chosen", "RMS", false), "chosen.RMS");
    }
}
