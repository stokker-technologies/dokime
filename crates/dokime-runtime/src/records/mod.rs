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
pub mod error;
pub mod evidence;
pub mod host;
pub mod inbox;
pub mod product;
pub mod publication;
pub mod route;
pub mod set;
#[cfg(feature = "host")]
pub mod resources;

/* === Re-Exports === */
pub use crate::core::{CompatibilityStatus, ValidationMode};
pub use crate::values::{FieldError, FieldValue, field};
pub use contract::{
    BuiltinRecordType, PRODUCT_CALIBRATION_KINDS, PRODUCT_CALIBRATION_PHASES,
    PRODUCT_CONNECTION_STATES, PRODUCT_READINESS_STATES, PRODUCT_TARGET_LOCK_STATES,
    RecordCondition, RecordConditionEvaluation, RecordContract, RecordDisplay, RecordField,
    RecordFilter, RecordIconKey, RecordPattern, RecordSeverity, RecordTone,
    builtin_record_contracts, builtin_record_types, evaluate_record_condition, validate_record,
    validate_record_details,
};
pub use declared::{DeclaredRecord, message_of};
pub use error::{RecordErrorDetail, RecordFailure, Result};
pub use evidence::{Record, RecordEntry, RecordMarker};
pub use inbox::{Deliveries, RecordSource};
pub use product::{
    PRODUCT_RECORD_SCHEMA_VERSION, ProductCalibrationDetails, ProductCalibrationKind,
    ProductCalibrationPhase, ProductConnectionDetails, ProductConnectionState,
    ProductReadinessState, ProductRecordDraft, ProductTargetLockDetails, ProductTargetLockState,
};
pub use publication::{PublishReceipt, PublishStatus, RecordPublishRequest};
pub use route::{PublishRecord, RECORD_PUBLICATION_VERSION, RecordPublication};
pub use set::TestRecordSet;
#[cfg(feature = "host")]
pub use resources::{ResourceLifecycle, ResourceRecord, ResourceRegistry};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports

use crate::core::{RecordId, RecordScope};

/* === Constants === */


/* === Macros === */


/* === Type Aliases === */


/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecords {
    pub record_visibility: Vec<RecordVisibility>,
    pub session: crate::records::TestRecordSet,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordVisibility {
    pub record_id: RecordId,
    pub scope: Option<RecordScope>,
}

/* === Enums === */
#[harmos::records]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Records {
    Record(Record),
}

/* === Traits === */


/* === Implementations === */


/* === Public Functions === */
pub fn decode<'de, D: serde::Deserializer<'de> + Copy>(
    id: &harmos::DefinitionId,
    version: u32,
    payload: D,
) -> std::result::Result<Option<Records>, D::Error> {
    if id.as_str() != "dokime.event" {
        return Records::restore(id, version, payload);
    }
    if version != 1 {
        return Ok(None);
    }
    let mut value = serde_json::Value::deserialize(payload)?;
    // The retired newtype wrapper serialized transparently as its leaf payload.
    // Details and other authored maps are deliberately never traversed.
    if let Some(object) = value.as_object_mut() {
        for (old, new) in [("event_id", "record_id"), ("event_type", "record_type")] {
            if let Some(previous) = object.remove(old)
                && object.insert(new.to_owned(), previous).is_some()
            {
                return Err(serde::de::Error::custom(format!(
                    "historical fact contains both {old} and {new}"
                )));
            }
        }
    }
    serde_json::from_value(value)
        .map(|record| Some(Records::Record(record)))
        .map_err(serde::de::Error::custom)
}

/* === Private Functions === */


/* === Tests === */


