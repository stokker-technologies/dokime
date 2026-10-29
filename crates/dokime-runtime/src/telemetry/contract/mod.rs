/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod conditions;
mod validation;

/* === Re-Exports === */

pub use conditions::evaluate_telemetry_condition;
pub use dokime_domain::model::telemetry::contract::*;
pub use validation::{
    check_normalized_telemetry_sample, check_telemetry_sample, telemetry_sample_check_memory, validate_normalized_telemetry_sample,
    validate_telemetry_sample,
};

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
