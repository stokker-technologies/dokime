/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

/* === Modules === */

pub mod fields;
pub mod format;
pub mod unit;
pub mod value;
pub mod wire;

/* === Re-Exports === */

pub use fields::*;
pub use format::ElapsedUnit;
pub use format::TIME_PATTERN_TOKENS;
pub use format::ValueFormat;
pub use format::validate_value_format;
pub use unit::MeasurementUnit;
pub use unit::UnitFamily;
pub use value::Value;
pub use value::ValueType;
pub use value::datetime_seconds;
pub use value::numeric_cmp;
pub use value::numeric_value;
pub use value::value_matches_type;

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

/* === Tests === */
