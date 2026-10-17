/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod resources;
pub mod evidence;
pub mod error;
pub mod product;
pub mod publication;
pub mod contract;

/* === Re-Exports === */
pub use resources::{ResourceRecord, ResourceLifecycle};
pub use evidence::{Record, RecordEntry, RecordMarker};
pub use error::{RecordErrorDetail, RecordFailure};
pub use product::*;
pub use publication::{PublishReceipt, PublishStatus, RecordPublishRequest};
pub use crate::model::validation::CompatibilityStatus;
pub use crate::model::validation::ValidationMode;
pub use contract::BuiltinRecordType;
pub use contract::PRODUCT_CALIBRATION_KINDS;
pub use contract::PRODUCT_CALIBRATION_PHASES;
pub use contract::PRODUCT_CONNECTION_STATES;
pub use contract::PRODUCT_READINESS_STATES;
pub use contract::PRODUCT_TARGET_LOCK_STATES;
pub use contract::RecordCondition;
pub use contract::RecordConditionEvaluation;
pub use contract::RecordContract;
pub use contract::RecordDisplay;
pub use contract::RecordField;
pub use contract::RecordIconKey;
pub use contract::RecordPattern;
pub use contract::RecordSeverity;
pub use contract::RecordTone;
pub use contract::builtin_record_contracts;
pub use contract::builtin_record_types;

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
