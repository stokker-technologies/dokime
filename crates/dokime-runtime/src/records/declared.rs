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
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::Value;
use dokime_domain::model::records::evidence::Record;
use crate::records::RecordContract;
use dokime_domain::model::records::publication::RecordPublishRequest;
use crate::records::RecordSeverity;
use crate::values::FieldError;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */
pub trait DeclaredRecord: Sized {
    const RECORD_TYPE: &'static str;

    const SCHEMA_VERSION: u32 = 1;

    const SEVERITY: RecordSeverity;

    fn contract() -> RecordContract;

    fn record_type() -> Identifier {
        Identifier::declared(Self::RECORD_TYPE, "record type")
    }

    fn message(&self) -> String;

    fn into_details(self) -> BTreeMap<String, Value>;

    fn from_record(record: &Record) -> crate::Result<Self>;

    fn from_details(values: &BTreeMap<String, Value>) -> Result<Self, FieldError>;

    #[must_use]
    fn severity(self, severity: RecordSeverity) -> RecordPublishRequest {
        let mut request = self.request();
        request.severity = severity;

        request
    }

    fn request(self) -> RecordPublishRequest {
        let mut request = RecordPublishRequest::new(Self::record_type(), Self::SEVERITY);
        request.message = Some(self.message());
        request.details = self.into_details();

        request
    }
}

pub trait RecordMessage {
    fn stated(&self, label: &str) -> String;
}

/* === Implementations === */
impl RecordMessage for String {
    fn stated(&self, _label: &str) -> String {
        self.clone()
    }
}

impl RecordMessage for Option<String> {
    fn stated(&self, label: &str) -> String {
        self.clone().unwrap_or_else(|| label.to_owned())
    }
}

/* === Public Functions === */

pub fn message_of(marked: &impl RecordMessage, label: &str) -> String {
    marked.stated(label)
}

/* === Private Functions === */

/* === Tests === */
