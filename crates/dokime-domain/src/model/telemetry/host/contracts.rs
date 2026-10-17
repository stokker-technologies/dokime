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
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::telemetry::contract::TelemetryColumn;
use crate::model::telemetry::contract::TelemetryContract;
use crate::model::telemetry::contract::TelemetryRole;
use crate::model::telemetry::host::HOST_STREAMS;
use crate::model::telemetry::host::HostTelemetryStream;
use crate::model::time::Timestamp;
use crate::model::values::FieldError;
use crate::model::values::FieldValue;
use crate::model::values::Value;
use crate::model::values::ValueType;
use harmos::Message;
use std::collections::BTreeMap;

/* === Constants === */

/* === Macros === */
macro_rules! host_contracts {
    (
        $(
            $stream:ident {
                id: $id:literal,
                label: $label:literal,
                columns: {
                    $( $field:ident : $value_type:ident , $role:ident , $column_label:literal , $ty:ty ; )*
                }
            }
        )*
    ) => {
        $(
            #[harmos::message]
            #[derive(Clone, PartialEq)]
            struct $stream { $( $field: $ty, )* }
        )*

        pub fn host_telemetry_payload(
            stream: HostTelemetryStream, values: &BTreeMap<String, Value>,
        ) -> Result<Vec<u8>, crate::model::values::FieldError> {
            match stream {
                $( HostTelemetryStream::$stream => {
                    let declared = [$( stringify!($field), )*];
                    if let Some(name) = values.keys().find(|name| name.as_str() != "timestamp" && !declared.contains(&name.as_str())) {
                        return Err(crate::model::values::FieldError { field: name.clone(), declared: ValueType::Object,
                            found: Some("undeclared host row field".to_owned()) });
                    }
                    let row = $stream { $( $field: crate::model::values::field(values, stringify!($field))?, )* };
                    Ok(row.encode_to_vec())
                }, )*
            }
        }

        pub fn project_host_values(
            stream: HostTelemetryStream,
            values: &mut BTreeMap<String, Value>,
        ) -> Result<(), FieldError> {
            match stream {
                $( HostTelemetryStream::$stream => {
                    $( project_host_field(values, stringify!($field), ValueType::$value_type)?; )*
                }, )*
            }
            Ok(())
        }

        pub fn host_telemetry_contract(stream: HostTelemetryStream) -> TelemetryContract {
            match stream {
                $(
                    HostTelemetryStream::$stream => TelemetryContract {
                        stream: identifier($id),
                        schema_version: 1,
                        storage_schema: None,
                        metadata: Metadata::new(identifier($id), $label, None),
                        template_id: None,
                        owner: None,
                        columns: ::std::vec![
                            $(
                                column(
                                    stringify!($field),
                                    $column_label,
                                    ValueType::$value_type,
                                    TelemetryRole::$role,
                                )
                            ),*
                        ],
                        default_x: Some("timestamp".to_owned()),
                        // The host declares its cadence the way every other
                        // observer does, so a template that leases a host
                        // stream never states one.
                        interval: Some(crate::model::time::DurationMillis(
                            HostTelemetryStream::$stream.interval_ms(),
                        )),
                        plot_window: None,
                        allow_extra_fields: true,
                        scalar: false,
                        validation_mode: None,
                        foreign: false,
                    }.typed::<$stream>(),
                )*
            }
        }
    };
}

/* === Expansions === */

crate::host_streams!(host_contracts);

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct HostTimestamp {
    pub value: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct HostTextList {
    pub values: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl FieldValue for HostTimestamp {
    const VALUE_TYPE: ValueType = ValueType::Datetime;

    fn into_value(self) -> Value {
        Value::from(Timestamp::new(self.value))
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        Timestamp::from_value(value).map(|timestamp| Self {
            value: timestamp.as_str().to_owned(),
        })
    }
}

impl FieldValue for HostTextList {
    const VALUE_TYPE: ValueType = ValueType::Array;

    fn into_value(self) -> Value {
        self.values.into_value()
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        Vec::<String>::from_value(value).map(|values| Self { values })
    }
}

/* === Public Functions === */
pub fn host_telemetry_contracts() -> Vec<TelemetryContract> {
    HOST_STREAMS
        .iter()
        .copied()
        .map(host_telemetry_contract)
        .collect()
}

/* === Private Functions === */
fn project_host_field(
    values: &mut BTreeMap<String, Value>,
    name: &str,
    kind: ValueType,
) -> Result<(), FieldError> {
    let member = match kind {
        ValueType::Datetime => "value",
        ValueType::Array => "values",
        _ => return Ok(()),
    };
    let Some(value) = values.remove(name) else {
        return Ok(());
    };
    if value == Value::Null {
        values.insert(name.to_owned(), value);
        return Ok(());
    }
    let invalid = || FieldError {
        field: name.to_owned(),
        declared: kind.clone(),
        found: Some("invalid host message value".to_owned()),
    };
    let Value::Object(mut object) = value else {
        return Err(invalid());
    };
    if object.len() != 1 {
        return Err(invalid());
    }
    let value = object.remove(member).ok_or_else(invalid)?;
    let value = match (&kind, value) {
        (ValueType::Datetime, Value::Text(text)) => {
            Value::Datetime(Timestamp::new(text).to_datetime().map_err(|_| invalid())?)
        }
        (ValueType::Array, Value::Array(items)) => {
            if items.iter().any(|item| !matches!(item, Value::Text(_))) {
                return Err(invalid());
            }
            Value::Array(items)
        }
        _ => return Err(invalid()),
    };
    values.insert(name.to_owned(), value);
    Ok(())
}

fn column(
    identifier_value: &str,
    label: &str,
    value_type: ValueType,
    role: TelemetryRole,
) -> TelemetryColumn {
    TelemetryColumn::new(
        Metadata::new(identifier(identifier_value), label, None),
        value_type,
        role,
    )
    .optional()
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).expect("host telemetry identifiers are valid")
}

/* === Tests === */
