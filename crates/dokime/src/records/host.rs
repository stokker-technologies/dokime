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
use crate::core::{Identifier, Value};
use crate::records::declared::DeclaredRecord;
use crate::records::{Record, RecordContract, RecordSeverity, builtin_record_contracts};
use crate::values::{FieldError, field};

/* === Constants === */
const PRODUCT_SCHEMA_VERSION: u32 = crate::records::PRODUCT_RECORD_SCHEMA_VERSION;

/* === Macros === */
macro_rules! host_record {
    (
        $record:ty,
        $identifier:literal,
        $severity:expr,
        $message:literal,
        [$($field:ident),+ $(,)?]
    ) => {
        impl DeclaredRecord for $record {
            const RECORD_TYPE: &'static str = $identifier;
            const SCHEMA_VERSION: u32 = PRODUCT_SCHEMA_VERSION;
            const SEVERITY: RecordSeverity = $severity;

            fn contract() -> RecordContract {
                builtin(Self::RECORD_TYPE)
            }

            fn message(&self) -> String {
                self.status_message
                    .clone()
                    .unwrap_or_else(|| $message.to_owned())
            }

            fn into_details(self) -> BTreeMap<String, Value> {
                let mut details = BTreeMap::new();
                $(details.insert(
                    ::std::stringify!($field).to_owned(),
                    <_ as crate::values::FieldValue>::into_value(self.$field),
                );)+

                details
            }

            fn from_record(record: &Record) -> crate::Result<Self> {
                if record.record_type.as_str() != Self::RECORD_TYPE {
                    return Err(crate::Error::record_mismatched(
                        Self::RECORD_TYPE,
                        &record.record_type,
                    ));
                }

                Self::from_details(&record.details)
                    .map_err(|error| crate::Error::record_field(Self::RECORD_TYPE, error))
            }

            fn from_details(values: &BTreeMap<String, Value>) -> Result<Self, FieldError> {
                Ok(Self {
                    $($field: field(values, ::std::stringify!($field))?,)+
                })
            }
        }

        #[cfg(feature = "native")]
        impl crate::execution::Listenable for $record {
            type Item = Self;
            type Listening = crate::execution::ListeningRecords<Self>;

            fn subscription() -> crate::definition::Subscription {
                crate::definition::Subscription::Record(<Self as crate::records::DeclaredRecord>::contract().record_type)
            }

            fn listening(
                _streams: &crate::telemetry::Streams,
                deliveries: &crate::records::Deliveries,
            ) -> Self::Listening {
                crate::execution::ListeningRecords::new(deliveries)
            }
        }
    };
}

host_record!(
    ToolConnectionAcquired,
    "tool.connection.acquired",
    RecordSeverity::Info,
    "a unit came online",
    [
        resource_id,
        resource_kind,
        connection_transport,
        connection_endpoint,
        status_message,
    ]
);

host_record!(
    ToolConnectionLost,
    "tool.connection.lost",
    RecordSeverity::Warning,
    "a unit went offline",
    [resource_id, resource_kind, status_message]
);

host_record!(
    TargetAcquired,
    "target.acquired",
    RecordSeverity::Info,
    "a target lock was acquired",
    [resource_id, resource_kind, status_message]
);

host_record!(
    TargetLost,
    "target.lost",
    RecordSeverity::Warning,
    "a target lock was lost",
    [resource_id, resource_kind, status_message]
);

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TestCycleCompleted;

#[derive(Clone, Debug, PartialEq)]
pub struct ToolConnectionAcquired {
    pub resource_id: String,
    pub resource_kind: String,
    pub connection_transport: Option<String>,
    pub connection_endpoint: Option<String>,
    pub status_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolConnectionLost {
    pub resource_id: String,
    pub resource_kind: String,
    pub status_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TargetAcquired {
    pub resource_id: String,
    pub resource_kind: String,
    pub status_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TargetLost {
    pub resource_id: String,
    pub resource_kind: String,
    pub status_message: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl DeclaredRecord for TestCycleCompleted {
    const RECORD_TYPE: &'static str = "test.cycle.completed";
    const SCHEMA_VERSION: u32 = 1;
    const SEVERITY: RecordSeverity = RecordSeverity::Info;

    fn contract() -> RecordContract {
        builtin(Self::RECORD_TYPE)
    }

    fn message(&self) -> String {
        "a test cycle completed".to_owned()
    }

    fn into_details(self) -> BTreeMap<String, Value> {
        BTreeMap::new()
    }

    fn from_record(record: &Record) -> crate::Result<Self> {
        if record.record_type.as_str() != Self::RECORD_TYPE {
            return Err(crate::Error::record_mismatched(
                Self::RECORD_TYPE,
                &record.record_type,
            ));
        }

        Ok(Self)
    }

    fn from_details(_values: &BTreeMap<String, Value>) -> Result<Self, FieldError> {
        Ok(Self)
    }
}

#[cfg(feature = "native")]
impl crate::execution::Listenable for TestCycleCompleted {
    type Item = Self;
    type Listening = crate::execution::ListeningRecords<Self>;

    fn subscription() -> crate::definition::Subscription {
        crate::definition::Subscription::Record(<Self as DeclaredRecord>::contract().record_type)
    }

    fn listening(
        _streams: &crate::telemetry::Streams,
        deliveries: &crate::records::Deliveries,
    ) -> Self::Listening {
        crate::execution::ListeningRecords::new(deliveries)
    }
}

/* === Public Functions === */

pub fn declared() -> Vec<Identifier> {
    vec![
        ToolConnectionAcquired::record_type(),
        ToolConnectionLost::record_type(),
        TargetAcquired::record_type(),
        TargetLost::record_type(),
        TestCycleCompleted::record_type(),
    ]
}

/* === Private Functions === */
/* === Functions === */

fn builtin(record_type: &str) -> RecordContract {
    builtin_record_contracts()
        .into_iter()
        .find(|contract| contract.record_type.as_str() == record_type)
        .unwrap_or_else(|| {
            panic!("the host declares a builtin contract for {record_type:?}");
        })
}

/* === Tests === */
