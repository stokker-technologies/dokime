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
use super::HostTelemetryStream;
use super::host_telemetry_contract;
use dokime::telemetry::{FieldError, FieldValue, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetryRow, TelemetryStream, field};
use dokime_domain::model::values::{MeasurementUnit, Value};

/* === Constants === */

/* === Macros === */

macro_rules! host_rows {
    (
        $(
            $stream:ident {
                id: $id:literal,
                label: $label:literal,
                columns: {
                    $( $field:ident : $value_type:ident , $role:ident , $column_label:literal , $ty:ty $(, unit: $unit:literal)? ; )*
                }
            }
        )*
    ) => {
        $(
            #[dokime::message]
            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct $stream {
                $(
                    pub $field: $ty,
                )*
            }

            impl $stream {
                $(
                    pub fn $field() -> dokime::document::Column<Self> {
                        dokime::document::Column::new(stringify!($field))
                    }
                )*
            }
            impl TelemetryRow for $stream {


                fn columns() -> Vec<TelemetryColumn> {
                    vec![
                        $(
                            TelemetryColumn::declared(
                                stringify!($field),
                                $column_label,
                                <$ty as FieldValue>::VALUE_TYPE,
                                TelemetryRole::$role,
                            )
                            .optional()
                            $(.with_unit(MeasurementUnit::custom(super::identifier($unit), $unit, $unit)))?
                        ),*
                    ]
                }

                fn into_row(self) -> BTreeMap<String, Value> {
                    let mut row = BTreeMap::new();
                    $(
                        row.insert(
                            stringify!($field).to_owned(),
                            FieldValue::into_value(self.$field),
                        );
                    )*
                    row
                }

                fn from_row(values: &BTreeMap<String, Value>) -> Result<Self, FieldError> {
                    Ok(Self {
                        $( $field: field(values, stringify!($field))?, )*
                    })
                }
            }

            impl dokime::StreamSpec for $stream {
                type Row = Self;
                const ID: &'static str = $id;
                const VERSION: u32 = 1;
                const WINDOW: usize = 4096;
            }
            impl TelemetryStream for $stream {
                const PUBLISHED: bool = false;

                fn contract() -> TelemetryContract {
                    // Static Authoring Has No Runtime Instance; Boot Applies Effective Cadence.
                    host_telemetry_contract(HostTelemetryStream::$stream).typed::<Self>()
                }
            }

            impl dokime::execution::Listenable for $stream {
                type Item = Self;
                type Listening = dokime::execution::ListeningRows<Self>;

                fn subscription() -> dokime::definition::Subscription {
                    dokime::definition::Subscription::Stream(<Self as TelemetryStream>::stream_id())
                }

                fn listening(
                    streams: &dokime::telemetry::Streams,
                    _deliveries: &dokime::records::Deliveries,
                ) -> Self::Listening {
                    dokime::execution::ListeningRows::new(streams)
                }
            }
        )*

        /* === Tests === */


    };
}

// Expansions

super::table::host_streams!(host_rows);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{self, Message};

    #[test]
    fn system_memory_contract() {
        let row = System {
            memory_used_gib: Some(3.5),
            memory_total_gib: Some(8.0),
            memory_pressure_percent: Some(43.75),
            ..Default::default()
        };
        let contract = System::contract();
        let schema = contract.storage_schema.as_ref().unwrap();
        let validator = harmos::schema::Validator::from_schema(schema, &harmos::config::SchemaSettings::default()).unwrap();
        let bytes = row.encode();
        validator.validate(&bytes).unwrap();
        assert_eq!(System::decode(&bytes).unwrap(), row);
        assert_eq!(System::from_row(&row.clone().into_row()).unwrap(), row);
        assert_eq!(dokime::storage::telemetry::decode_values(&validator, &bytes).unwrap(), row.into_row());

        for columns in [System::columns(), contract.columns] {
            for field in ["memory_used_gib", "memory_total_gib"] {
                let column = columns.iter().find(|column| column.metadata.identifier.as_str() == field).unwrap();
                assert_eq!(column.value_type, dokime::ValueType::Number);
                assert_eq!(column.unit.as_ref().unwrap().symbol, "GiB");
            }
        }
        assert_eq!(contract.schema_version, 1);
    }

    #[test]
    fn host_lists_roundtrip() {
        fn round_trip<T: Message + TelemetryRow>(stream: HostTelemetryStream, field: &str) {
            let contract = host_telemetry_contract(stream);
            let schema = contract.storage_schema.as_ref().unwrap();
            let validator = harmos::schema::Validator::from_schema(schema, &harmos::config::SchemaSettings::default()).unwrap();
            for items in [vec![], vec![Value::Text("example".to_owned())]] {
                let values = BTreeMap::from([(field.to_owned(), Value::Array(items))]);
                let row = T::from_row(&values).unwrap();
                let bytes = row.encode();
                validator.validate(&bytes).unwrap();
                let decoded = T::decode(&bytes).unwrap().into_row();
                assert_eq!(decoded.get(field), values.get(field));
                assert!(!decoded.contains_key("timestamp"));
                assert_eq!(dokime::storage::telemetry::decode_values(&validator, &bytes).unwrap(), decoded);
            }
        }
        round_trip::<Bluetooth>(HostTelemetryStream::Bluetooth, "bluetooth_device_names");
    }
}
