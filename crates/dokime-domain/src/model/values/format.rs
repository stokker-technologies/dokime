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
use serde::Deserialize;
use serde::Serialize;

// Local Imports

/* === Constants === */

pub const TIME_PATTERN_TOKENS: &[&str] = &["%Y", "%m", "%d", "%H", "%M", "%S", "%L", "%%"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ValueFormat {
    Normal {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_fraction_digits: Option<u8>,
    },
    Fixed {
        decimals: u8,
    },
    Scientific {
        significant_digits: u8,
    },
    Engineering {
        significant_digits: u8,
    },
    SiPrefix {
        significant_digits: u8,
    },
    AbsoluteTime {
        pattern: String,
    },
    ElapsedSinceRun {
        unit: ElapsedUnit,
    },
    ElapsedSinceCycle {
        unit: ElapsedUnit,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElapsedUnit {
    Seconds,
    Milliseconds,
    Minutes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueFormatKind {
    Number,
    Time,
}

/* === Traits === */

/* === Implementations === */

impl harmos::MemorySize for ValueFormat {
    fn memory_size(&self) -> usize {
        let heap = match self {
            Self::AbsoluteTime { pattern } => pattern.capacity(),
            _ => 0,
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}

impl ValueFormat {
    pub(crate) fn kind(&self) -> ValueFormatKind {
        match self {
            Self::Normal { .. } | Self::Fixed { .. } | Self::Scientific { .. } | Self::Engineering { .. } | Self::SiPrefix { .. } => {
                ValueFormatKind::Number
            }
            Self::AbsoluteTime { .. } | Self::ElapsedSinceRun { .. } | Self::ElapsedSinceCycle { .. } => ValueFormatKind::Time,
        }
    }
}

/* === Public Functions === */

pub fn validate_value_format(format: &ValueFormat) -> Result<(), String> {
    validate_value_format_with(format, &crate::model::validation::diagnostic::text)
}

pub(crate) fn validate_value_format_with<E>(format: &ValueFormat, error: &impl Fn(std::fmt::Arguments<'_>) -> E) -> Result<(), E> {
    match format {
        ValueFormat::Normal { max_fraction_digits } => {
            if max_fraction_digits.is_some_and(|digits| digits > 12) {
                return Err(error(format_args!("max_fraction_digits must be 12 or fewer")));
            }
        }
        ValueFormat::Fixed { decimals } => {
            if *decimals > 12 {
                return Err(error(format_args!("decimals must be 12 or fewer")));
            }
        }
        ValueFormat::Scientific { significant_digits }
        | ValueFormat::Engineering { significant_digits }
        | ValueFormat::SiPrefix { significant_digits } => {
            if !(1..=12).contains(significant_digits) {
                return Err(error(format_args!("significant_digits must be between 1 and 12")));
            }
        }
        ValueFormat::AbsoluteTime { pattern } => {
            if pattern.is_empty() {
                return Err(error(format_args!("pattern must not be empty")));
            }
            validate_time_pattern(pattern, error)?;
        }
        ValueFormat::ElapsedSinceRun { .. } | ValueFormat::ElapsedSinceCycle { .. } => {}
    }

    Ok(())
}

/* === Private Functions === */

fn validate_time_pattern<E>(pattern: &str, error: &impl Fn(std::fmt::Arguments<'_>) -> E) -> Result<(), E> {
    let mut characters = pattern.chars();
    while let Some(character) = characters.next() {
        if character != '%' {
            continue;
        }
        let Some(next) = characters.next() else {
            return Err(error(format_args!("pattern ends with a dangling `%`")));
        };
        if !TIME_PATTERN_TOKENS.iter().any(|token| token.chars().skip(1).eq(std::iter::once(next))) {
            return Err(error(format_args!("unknown time pattern token `%{next}`: pattern takes one of {Tokens}")));
        }
    }

    Ok(())
}

struct Tokens;

impl std::fmt::Display for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, token) in TIME_PATTERN_TOKENS.iter().enumerate() {
            if index != 0 {
                f.write_str(", ")?;
            }
            f.write_str(token)?;
        }
        Ok(())
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::validation::diagnostic;

    #[test]
    fn diagnostic_byte_visitor_matches_format_validation() {
        let formats = [
            ValueFormat::Normal { max_fraction_digits: Some(13) },
            ValueFormat::Fixed { decimals: 13 },
            ValueFormat::Scientific { significant_digits: 0 },
            ValueFormat::Engineering { significant_digits: 13 },
            ValueFormat::SiPrefix { significant_digits: 1 },
            ValueFormat::AbsoluteTime { pattern: String::new() },
            ValueFormat::AbsoluteTime { pattern: "%".into() },
            ValueFormat::AbsoluteTime { pattern: "%🦀".into() },
            ValueFormat::AbsoluteTime {
                pattern: "%Y-%m-%d %% %H:%M:%S.%L".into(),
            },
            ValueFormat::ElapsedSinceRun { unit: ElapsedUnit::Seconds },
        ];
        for format in formats {
            let counted = validate_value_format_with(&format, &diagnostic::size);
            let rendered = validate_value_format(&format).map_err(|issue| issue.len());
            assert_eq!(counted, rendered);
        }
    }
}
