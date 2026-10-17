/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod contract;
pub mod declared;
pub mod host;
pub mod inbox;
pub mod route;

/* === Re-Exports === */
pub use crate::values::{FieldError, FieldValue, field};
pub use contract::{
    BuiltinRecordType, PRODUCT_CALIBRATION_KINDS, PRODUCT_CALIBRATION_PHASES, PRODUCT_CONNECTION_STATES, PRODUCT_READINESS_STATES,
    PRODUCT_TARGET_LOCK_STATES, RecordCondition, RecordConditionEvaluation, RecordContract, RecordDisplay, RecordField, RecordFilter,
    RecordIconKey, RecordPattern, RecordSeverity, RecordTone, builtin_record_contracts, builtin_record_types, evaluate_record_condition,
    validate_record, validate_record_details,
};
pub use declared::{DeclaredRecord, message_of};
pub use dokime_domain::model::validation::CompatibilityStatus;
pub use dokime_domain::model::validation::ValidationMode;
pub use inbox::{Deliveries, RecordSource};
pub use route::{PublishRecord, RECORD_PUBLICATION_VERSION, RecordPublication};

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
