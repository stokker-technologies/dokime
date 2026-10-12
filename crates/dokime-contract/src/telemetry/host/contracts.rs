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
use crate::core::{Identifier, Metadata, ValueType};
use crate::telemetry::contract::{TelemetryColumn, TelemetryContract, TelemetryRole};
use crate::telemetry::host::{HOST_STREAMS, HostTelemetryStream};

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
                pub fn host_telemetry_contract(stream: HostTelemetryStream) -> TelemetryContract {
            match stream {
                $(
                    HostTelemetryStream::$stream => TelemetryContract {
                        stream: identifier($id),
                        schema_version: 1,
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
                        interval: Some(crate::core::DurationMillis(
                            HostTelemetryStream::$stream.interval_ms(),
                        )),
                        plot_window: None,
                        allow_extra_fields: true,
                        scalar: false,
                        validation_mode: None,
                        foreign: false,
                    },
                )*
            }
        }
    };
}

/* === Expansions === */

crate::host_streams!(host_contracts);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn host_telemetry_contracts() -> Vec<TelemetryContract> {
    HOST_STREAMS
        .iter()
        .copied()
        .map(host_telemetry_contract)
        .collect()
}

/* === Private Functions === */
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
