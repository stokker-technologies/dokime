/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::model::values::ElapsedUnit;
use crate::model::values::ValueFormat;

pub fn normal() -> ValueFormat {
    ValueFormat::Normal { max_fraction_digits: None }
}

pub fn fixed(decimals: u8) -> ValueFormat {
    ValueFormat::Fixed { decimals }
}

pub fn engineering() -> ValueFormat {
    ValueFormat::Engineering { significant_digits: 3 }
}

pub fn clock(pattern: impl Into<String>) -> ValueFormat {
    ValueFormat::AbsoluteTime { pattern: pattern.into() }
}

pub fn elapsed_seconds() -> ValueFormat {
    ValueFormat::ElapsedSinceRun { unit: ElapsedUnit::Seconds }
}

pub fn elapsed_ms() -> ValueFormat {
    ValueFormat::ElapsedSinceRun { unit: ElapsedUnit::Milliseconds }
}
