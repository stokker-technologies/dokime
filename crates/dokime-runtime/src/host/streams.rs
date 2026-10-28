/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use crate::telemetry::{FieldError, FieldValue, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetryRow, TelemetryStream, field};
use dokime_domain::model::telemetry::HostTelemetryStream;
use dokime_domain::model::telemetry::host_telemetry_contract;
use dokime_domain::model::values::Value;

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


    };
}

// Expansions

dokime_domain::host_streams!(host_rows);

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::time::Timestamp;
    use harmos::Message;

    #[test]
    fn host_messages_preserve_optional_lists_and_timestamps() {
        fn round_trip<T: Message + TelemetryRow>(stream: HostTelemetryStream, field: &str) {
            let timestamp = Value::from(Timestamp::new("2026-10-15T12:34:56.123456789Z"));
            let contract = host_telemetry_contract(stream);
            let schema = contract.storage_schema.as_ref().expect("host storage schema");
            let validator =
                harmos::schema::Validator::from_schema(schema, &harmos::config::SchemaSettings::default()).expect("valid host schema");

            for list in [None, Some(Value::Null), Some(Value::Array(vec![])), Some(Value::Array(vec![Value::Text("example".to_owned())]))] {
                let mut values = BTreeMap::from([("timestamp".to_owned(), timestamp.clone())]);
                if let Some(list) = &list {
                    values.insert(field.to_owned(), list.clone());
                }
                let bytes = dokime_domain::model::telemetry::host::prepare_host_telemetry_payload(stream, &values)
                    .expect("prepare host row")
                    .into_bytes();
                validator.validate(&bytes).expect("payload matches storage schema");
                let row = T::decode(&bytes).expect("decode typed host row");
                assert_eq!(row.encode(), bytes);
                let projected = row.into_row();
                assert_eq!(projected.get(field), Some(&list.unwrap_or(Value::Null)));
                assert_eq!(projected.get("timestamp"), Some(&timestamp));
                #[cfg(feature = "storage")]
                {
                    let mut stored = crate::storage::telemetry::decode_values(&validator, &bytes).expect("project stored host payload");
                    dokime_domain::model::telemetry::host::contracts::project_host_values(stream, &mut stored)
                        .expect("restore host value semantics");
                    assert_eq!(stored, projected);
                }
            }
        }

        round_trip::<Network>(HostTelemetryStream::Network, "local_addresses");
        round_trip::<Bluetooth>(HostTelemetryStream::Bluetooth, "bluetooth_device_names");
    }
}
