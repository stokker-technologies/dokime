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
use crate::records::RecordContract;
use crate::records::RecordSeverity;
use crate::records::builtin_record_contracts;
use crate::records::declared::DeclaredRecord;
use crate::values::{FieldError, field};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::evidence::Record;
use dokime_domain::model::values::Value;

/* === Constants === */
const PRODUCT_SCHEMA_VERSION: u32 = dokime_domain::model::records::product::PRODUCT_RECORD_SCHEMA_VERSION;

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
    [resource_id, resource_kind, connection_transport, connection_endpoint, status_message,]
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

host_record!(TargetLost, "target.lost", RecordSeverity::Warning, "a target lock was lost", [resource_id, resource_kind, status_message]);

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TestCycleCompleted;

// A process a step ran exited, by itself or cancelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessExited {
    pub program: String,
    pub exit_code: Option<i64>,
    pub duration_ms: i64,
    pub cancelled: bool,
}

// Something a step left running that the step-end sweep cancelled: a
// `process`, a `watch` or another submitted `operation`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepHandleCancelled {
    pub step: String,
    pub kind: String,
    pub name: String,
}

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
            return Err(crate::Error::record_mismatched(Self::RECORD_TYPE, &record.record_type));
        }

        Ok(Self)
    }

    fn from_details(_values: &BTreeMap<String, Value>) -> Result<Self, FieldError> {
        Ok(Self)
    }
}

impl DeclaredRecord for ProcessExited {
    const RECORD_TYPE: &'static str = "process.exited";
    const SEVERITY: RecordSeverity = RecordSeverity::Info;

    fn contract() -> RecordContract {
        builtin(Self::RECORD_TYPE)
    }

    fn message(&self) -> String {
        match (self.cancelled, self.exit_code) {
            (true, _) => format!("{} was cancelled", self.program),
            (false, Some(code)) => format!("{} exited with {code}", self.program),
            (false, None) => format!("{} ended by a signal", self.program),
        }
    }

    fn into_details(self) -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("program".to_owned(), Value::Text(self.program)),
            ("exit_code".to_owned(), self.exit_code.map_or(Value::Null, Value::Integer)),
            ("duration_ms".to_owned(), Value::Integer(self.duration_ms)),
            ("cancelled".to_owned(), Value::Bool(self.cancelled)),
        ])
    }

    fn from_record(record: &Record) -> crate::Result<Self> {
        checked::<Self>(record)
    }

    fn from_details(values: &BTreeMap<String, Value>) -> Result<Self, FieldError> {
        Ok(Self {
            program: field(values, "program")?,
            exit_code: field(values, "exit_code")?,
            duration_ms: field(values, "duration_ms")?,
            cancelled: field(values, "cancelled")?,
        })
    }
}

impl DeclaredRecord for StepHandleCancelled {
    const RECORD_TYPE: &'static str = "test.step.handle_cancelled";
    const SEVERITY: RecordSeverity = RecordSeverity::Warning;

    fn contract() -> RecordContract {
        builtin(Self::RECORD_TYPE)
    }

    fn message(&self) -> String {
        format!("step `{}` ended with its {} `{}` still running; it was cancelled", self.step, self.kind, self.name)
    }

    fn into_details(self) -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("step".to_owned(), Value::Text(self.step)),
            ("kind".to_owned(), Value::Text(self.kind)),
            ("name".to_owned(), Value::Text(self.name)),
        ])
    }

    fn from_record(record: &Record) -> crate::Result<Self> {
        checked::<Self>(record)
    }

    fn from_details(values: &BTreeMap<String, Value>) -> Result<Self, FieldError> {
        Ok(Self { step: field(values, "step")?, kind: field(values, "kind")?, name: field(values, "name")? })
    }
}

#[cfg(feature = "native")]
impl crate::execution::Listenable for TestCycleCompleted {
    type Item = Self;
    type Listening = crate::execution::ListeningRecords<Self>;

    fn subscription() -> crate::definition::Subscription {
        crate::definition::Subscription::Record(<Self as DeclaredRecord>::contract().record_type)
    }

    fn listening(_streams: &crate::telemetry::Streams, deliveries: &crate::records::Deliveries) -> Self::Listening {
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

fn checked<R: DeclaredRecord>(record: &Record) -> crate::Result<R> {
    if record.record_type.as_str() != R::RECORD_TYPE {
        return Err(crate::Error::record_mismatched(R::RECORD_TYPE, &record.record_type));
    }
    R::from_details(&record.details).map_err(|error| crate::Error::record_field(R::RECORD_TYPE, error))
}

fn builtin(record_type: &str) -> RecordContract {
    builtin_record_contracts().into_iter().find(|contract| contract.record_type.as_str() == record_type).unwrap_or_else(|| {
        panic!("the host declares a builtin contract for {record_type:?}");
    })
}

/* === Tests === */
