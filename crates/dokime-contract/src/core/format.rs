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
use serde::{Deserialize, Serialize};
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
            Self::Normal { .. }
            | Self::Fixed { .. }
            | Self::Scientific { .. }
            | Self::Engineering { .. }
            | Self::SiPrefix { .. } => ValueFormatKind::Number,
            Self::AbsoluteTime { .. }
            | Self::ElapsedSinceRun { .. }
            | Self::ElapsedSinceCycle { .. } => ValueFormatKind::Time,
        }
    }
}

/* === Public Functions === */
pub fn validate_value_format(format: &ValueFormat) -> Result<(), String> {
    match format {
        ValueFormat::Normal {
            max_fraction_digits,
        } => {
            if max_fraction_digits.is_some_and(|digits| digits > 12) {
                return Err("max_fraction_digits must be 12 or fewer".to_owned());
            }
        }
        ValueFormat::Fixed { decimals } => {
            if *decimals > 12 {
                return Err("decimals must be 12 or fewer".to_owned());
            }
        }
        ValueFormat::Scientific { significant_digits }
        | ValueFormat::Engineering { significant_digits }
        | ValueFormat::SiPrefix { significant_digits } => {
            if !(1..=12).contains(significant_digits) {
                return Err("significant_digits must be between 1 and 12".to_owned());
            }
        }
        ValueFormat::AbsoluteTime { pattern } => {
            if pattern.is_empty() {
                return Err("pattern must not be empty".to_owned());
            }
            validate_time_pattern(pattern)?;
        }
        ValueFormat::ElapsedSinceRun { .. } | ValueFormat::ElapsedSinceCycle { .. } => {}
    }

    Ok(())
}

/* === Private Functions === */
fn validate_time_pattern(pattern: &str) -> Result<(), String> {
    let mut characters = pattern.chars();
    while let Some(character) = characters.next() {
        if character != '%' {
            continue;
        }
        let Some(next) = characters.next() else {
            return Err("pattern ends with a dangling `%`".to_owned());
        };
        let token = format!("%{next}");
        if !TIME_PATTERN_TOKENS.contains(&token.as_str()) {
            return Err(format!(
                "unknown time pattern token `{token}`: pattern takes one of {}",
                TIME_PATTERN_TOKENS.join(", ")
            ));
        }
    }

    Ok(())
}

/* === Tests === */
