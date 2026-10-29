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
use crate::telemetry::{FieldError, FieldValue, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetryRow, TelemetryStream, field};
use dokime_domain::model::telemetry::HostTelemetryStream;
use dokime_domain::model::telemetry::host_telemetry_contract;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

macro_rules! host_rows {
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
            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct $stream {
                $(
                    pub $field: $ty,
                )*
            }

            impl $stream {
                $(
                    pub fn $field() -> crate::document::Column<Self> {
                        crate::document::Column::new(stringify!($field))
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

            impl crate::StreamSpec for $stream {
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

            #[cfg(feature = "native")]
            impl crate::execution::Listenable for $stream {
                type Item = Self;
                type Listening = crate::execution::ListeningRows<Self>;

                fn subscription() -> crate::definition::Subscription {
                    crate::definition::Subscription::Stream(<Self as TelemetryStream>::stream_id())
                }

                fn listening(
                    streams: &crate::telemetry::Streams,
                    _deliveries: &crate::records::Deliveries,
                ) -> Self::Listening {
                    crate::execution::ListeningRows::new(streams)
                }
            }
        )*

        /* === Tests === */


    };
}

// Expansions

dokime_domain::host_streams!(host_rows);

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
    use harmos::Message;

    #[test]
    fn host_messages_use_native_repeated_fields() {
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
                assert_eq!(crate::storage::telemetry::decode_values(&validator, &bytes).unwrap(), decoded);
            }
        }
        round_trip::<Network>(HostTelemetryStream::Network, "local_addresses");
        round_trip::<Bluetooth>(HostTelemetryStream::Bluetooth, "bluetooth_device_names");
    }
}
