/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod builtins;
mod conditions;
mod pattern;
mod schema;
mod severity;

/* === Re-Exports === */
pub use builtins::{
    BuiltinRecordType, PRODUCT_CALIBRATION_KINDS, PRODUCT_CALIBRATION_PHASES,
    PRODUCT_CONNECTION_STATES, PRODUCT_READINESS_STATES, PRODUCT_TARGET_LOCK_STATES,
    builtin_record_contracts, builtin_record_types,
};
pub use conditions::{RecordCondition, RecordConditionEvaluation};
pub use pattern::RecordPattern;
pub use schema::{RecordContract, RecordDisplay, RecordField, RecordIconKey, RecordTone};
pub use severity::RecordSeverity;

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
