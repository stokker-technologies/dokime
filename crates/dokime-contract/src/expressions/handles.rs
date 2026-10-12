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
use crate::expressions::data::{DataExpression, DataSource, DataTransform};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordMarker {
    record_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordField {
    record_type: String,
    detail: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RecordMarker {
    pub fn new(record_type: impl Into<String>) -> Self {
        Self {
            record_type: record_type.into(),
        }
    }

    pub fn record_type(&self) -> &str {
        &self.record_type
    }

    pub fn into_record_type(self) -> String {
        self.record_type
    }
}

impl From<&str> for RecordMarker {
    fn from(record_type: &str) -> Self {
        Self::new(record_type)
    }
}

impl From<String> for RecordMarker {
    fn from(record_type: String) -> Self {
        Self::new(record_type)
    }
}

impl From<&String> for RecordMarker {
    fn from(record_type: &String) -> Self {
        Self::new(record_type.clone())
    }
}

impl RecordField {
    pub fn new(record_type: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            record_type: record_type.into(),
            detail: detail.into(),
        }
    }

    pub fn record_type(&self) -> &str {
        &self.record_type
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl From<RecordField> for DataExpression {
    fn from(field: RecordField) -> Self {
        DataExpression {
            label: None,
            source: DataSource::Record {
                record_type: field.record_type,
            },
            transforms: vec![DataTransform::Detail { name: field.detail }],
        }
    }
}

impl From<&RecordField> for DataExpression {
    fn from(field: &RecordField) -> Self {
        Self::from(field.clone())
    }
}

impl From<RecordField> for RecordMarker {
    fn from(field: RecordField) -> Self {
        Self::new(field.record_type)
    }
}

/* === Public Functions === */
pub fn marker(record_type: impl Into<String>) -> RecordMarker {
    RecordMarker::new(record_type)
}

pub fn detail(record_type: impl Into<String>, detail: impl Into<String>) -> RecordField {
    RecordField::new(record_type, detail)
}

/* === Private Functions === */

/* === Tests === */
