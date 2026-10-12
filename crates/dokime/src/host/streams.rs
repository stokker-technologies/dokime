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
use crate::core::{Timestamp, Value};
use crate::telemetry::{
    FieldError, FieldValue, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetryRow,
    TelemetryStream, field,
};
use dokime_contract::telemetry::{HostTelemetryStream, host_telemetry_contract};

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
                fn contract() -> TelemetryContract {
                    host_telemetry_contract(HostTelemetryStream::$stream)
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

/* === Expansions === */

dokime_contract::host_streams!(host_rows);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
