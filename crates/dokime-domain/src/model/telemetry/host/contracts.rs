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
use harmos::Message;

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

/* === Constants === */

const UNDECLARED_FIELD: &str = "undeclared host row field";

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

        pub struct PreparedHostPayload(HostPayload);

        enum HostPayload { $( $stream(Box<$stream>), )* }

        impl PreparedHostPayload {
            pub fn encoded_len(&self) -> usize {
                match &self.0 { $( HostPayload::$stream(row) => row.encoded_size(), )* }
            }

            pub fn into_bytes(self) -> Vec<u8> {
                match self.0 { $( HostPayload::$stream(row) => row.encode(), )* }
            }
        }

        pub fn prepare_host_telemetry_payload(
            stream: HostTelemetryStream, values: &BTreeMap<String, Value>,
        ) -> Result<PreparedHostPayload, crate::model::values::FieldError> {
            match stream {
                $( HostTelemetryStream::$stream => {
                    let declared = [$( stringify!($field), )*];
                    if let Some(name) = values.keys().find(|name| name.as_str() != "timestamp" && !declared.contains(&name.as_str())) {
                        return Err(crate::model::values::FieldError { field: name.clone(), declared: ValueType::Object,
                            found: Some(UNDECLARED_FIELD.to_owned()) });
                    }
                    let row = $stream { $( $field: crate::model::values::field(values, stringify!($field))?, )* };
                    Ok(PreparedHostPayload(HostPayload::$stream(Box::new(row))))
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

        pub fn host_telemetry_contract(
            stream: HostTelemetryStream,
        ) -> TelemetryContract {
            let interval_ms = match stream {
                HostTelemetryStream::System | HostTelemetryStream::Network => 1_000,
                HostTelemetryStream::Wifi | HostTelemetryStream::Bluetooth => 5_000,
                HostTelemetryStream::WifiLatency => 10_000,
                HostTelemetryStream::WifiChannelOccupancy => 30_000,
            };
            let interval = crate::model::time::DurationMillis(interval_ms);
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
                        interval: Some(interval),
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

// Expansions

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
        let Some(Value::Datetime(timestamp)) = value else {
            return Timestamp::from_value(value).map(|timestamp| Self { value: timestamp.as_str().to_owned() });
        };
        let mut bytes = Vec::new();
        timestamp
            .to_offset(time::UtcOffset::UTC)
            .format_into(&mut bytes, &time::format_description::well_known::Rfc3339)
            .expect("a stored datetime is representable as RFC3339");
        Ok(Self {
            value: String::from_utf8(bytes).expect("RFC3339 output is UTF-8"),
        })
    }
}

impl FieldValue for HostTextList {
    const VALUE_TYPE: ValueType = ValueType::Array;

    fn into_value(self) -> Value {
        self.values.into_value()
    }

    fn from_value(value: Option<&Value>) -> Result<Self, FieldError> {
        let Some(Value::Array(items)) = value else {
            return Vec::<String>::from_value(value).map(|values| Self { values });
        };
        let mut values = Vec::with_capacity(items.len());
        for item in items {
            values.push(String::from_value(Some(item))?);
        }
        Ok(Self { values })
    }
}

/* === Public Functions === */

pub fn host_telemetry_contracts() -> Vec<TelemetryContract> {
    let mut contracts = Vec::with_capacity(HOST_STREAMS.len());
    for stream in HOST_STREAMS.iter().copied() {
        let contract = host_telemetry_contract(stream);
        contracts.push(contract);
    }
    contracts
}

/* === Private Functions === */

fn project_host_field(values: &mut BTreeMap<String, Value>, name: &str, kind: ValueType) -> Result<(), FieldError> {
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
        (ValueType::Datetime, Value::Text(text)) => Value::Datetime(Timestamp::new(text).to_datetime().map_err(|_| invalid())?),
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

fn column(identifier_value: &str, label: &str, value_type: ValueType, role: TelemetryRole) -> TelemetryColumn {
    TelemetryColumn::new(Metadata::new(identifier(identifier_value), label, None), value_type, role).optional()
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).expect("host telemetry identifiers are valid")
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_payload_preserves_native_timestamp_and_text_list() {
        let timestamp = time::OffsetDateTime::UNIX_EPOCH.to_offset(time::UtcOffset::from_hms(2, 0, 0).unwrap());
        let addresses = vec!["192.0.2.1".to_owned(), "2001:db8::1".to_owned()];
        let values = BTreeMap::from([
            ("timestamp".to_owned(), Value::Datetime(timestamp)),
            ("local_addresses".to_owned(), Value::Array(addresses.iter().cloned().map(Value::Text).collect())),
        ]);
        let prepared = prepare_host_telemetry_payload(HostTelemetryStream::Network, &values).unwrap();
        let length = prepared.encoded_len();
        let bytes = prepared.into_bytes();
        assert_eq!(bytes.len(), length);
        let row = Network::decode(&bytes).unwrap();
        assert_eq!(row.timestamp.unwrap().value, crate::model::time::datetime_text(timestamp));
        assert_eq!(row.local_addresses.unwrap().values, addresses);
        assert!(row.sample_index.is_none());
    }

    #[test]
    fn staged_payload_rejects_unknown_fields() {
        let values = BTreeMap::from([("unknown.field".to_owned(), Value::Bool(true))]);
        let error = match prepare_host_telemetry_payload(HostTelemetryStream::System, &values) {
            Ok(_) => panic!("unexpected accepted field"),
            Err(error) => error,
        };
        assert_eq!(error.field, "unknown.field");
    }
}
