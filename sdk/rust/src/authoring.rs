/* ***
 *
 * Dokime
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

pub(crate) fn code_name<T>(type_name: bool) -> String {
    let name = any::type_name::<T>().rsplit("::").next().unwrap_or_default();
    if name.contains("closure") || name.starts_with("fn(") {
        return String::new();
    }
    let name = if type_name {
        name.strip_suffix("TestTemplate").or_else(|| name.strip_suffix("Plugin")).or_else(|| name.strip_suffix("Suite")).unwrap_or(name)
    } else {
        name
    };
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

pub(crate) fn type_label<T>() -> String {
    let name = any::type_name::<T>().rsplit("::").next().unwrap_or_default();
    let name =
        name.strip_suffix("TestTemplate").or_else(|| name.strip_suffix("Plugin")).or_else(|| name.strip_suffix("Suite")).unwrap_or(name);
    let characters = name.chars().collect::<Vec<_>>();
    let mut separated = String::new();
    for (index, character) in characters.iter().copied().enumerate() {
        let previous = index.checked_sub(1).map(|index| characters[index]);
        let next = characters.get(index + 1);
        let boundary = character.is_uppercase()
            && previous.is_some_and(|previous| {
                previous.is_lowercase()
                    || previous.is_numeric()
                    || (previous.is_uppercase() && next.is_some_and(|next| next.is_lowercase()))
            });
        if boundary {
            separated.push(' ');
        }
        separated.push(if character == '_' { ' ' } else { character });
    }
    let words = separated.split_whitespace().collect::<Vec<_>>();
    words
        .iter()
        .enumerate()
        .map(|(index, word)| {
            if word.chars().all(|character| !character.is_alphabetic() || character.is_uppercase()) {
                return (*word).into();
            }
            let word = word.to_lowercase();
            if index != 0
                && index + 1 != words.len()
                && matches!(
                    word.as_str(),
                    "a" | "an"
                        | "the"
                        | "and"
                        | "but"
                        | "or"
                        | "nor"
                        | "as"
                        | "at"
                        | "by"
                        | "for"
                        | "in"
                        | "of"
                        | "on"
                        | "per"
                        | "to"
                        | "via"
                )
            {
                return word;
            }
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| first.to_uppercase().collect::<String>() + characters.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{build_identifier, code_name, type_label};
    struct HTTPAccuracySuite;
    struct VoltageAccuracyTestTemplate;
    struct NumberOfRowsPlugin;
    #[test]
    fn names_follow_code_boundaries_and_preserve_explicit_identifiers() {
        assert_eq!(code_name::<HTTPAccuracySuite>(true), "http_accuracy");
        assert_eq!(type_label::<HTTPAccuracySuite>(), "HTTP Accuracy");
        assert_eq!(type_label::<NumberOfRowsPlugin>(), "Number of Rows");
        assert_eq!(code_name::<NumberOfRowsPlugin>(true), "number_of_rows");
        assert_eq!(code_name::<HTTPAccuracySuite>(false), "http_accuracy_suite");
        assert_eq!(code_name::<VoltageAccuracyTestTemplate>(true), "voltage_accuracy");
        for name in ["ReadVoltage", "readVoltage", "read_voltage"] {
            assert_eq!(build_identifier("accuracy.voltage", name, true), "accuracy.voltage.read_voltage");
        }
        assert_eq!(build_identifier("chosen", "RMS", false), "chosen.RMS");
    }
}
