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

// Local Imports
use crate::records::{DeclaredRecord, Record};

/* === Constants === */
pub const LISTENER_POLICY: harmos::operations::OperationPolicy =
    harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(30),
        attempt_timeout: std::time::Duration::from_secs(30),
        retries: harmos::operations::RetryPolicy::Never,
    };

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Delivery {
    #[row(description = "JSON-encoded [`Record`](crate::records::Record), as the host filed it.")]
    pub record_json: String,
}

/* === Enums === */

/* === Traits === */
pub trait Listener: 'static {
    const ID: &'static str;

    type Record: DeclaredRecord;
}

/* === Implementations === */
impl Delivery {
    pub fn of(record: &Record) -> Self {
        Self {
            record_json: serde_json::to_string(record).expect("a filed record serializes"),
        }
    }

    pub fn record<R: DeclaredRecord>(&self) -> crate::Result<R> {
        let record: Record = serde_json::from_str(&self.record_json).map_err(|error| {
            crate::Error::RecordDecode {
                record: R::RECORD_TYPE.to_owned(),
                reason: error.to_string(),
            }
        })?;
        R::from_record(&record)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
