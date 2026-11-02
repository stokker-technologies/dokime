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
use super::{HOST_STREAMS, HostTelemetryStream};
use dokime_domain::model::identity::{Identifier, Metadata};
use dokime_domain::model::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};
use dokime_domain::model::values::{MeasurementUnit, ValueType};

/* === Constants === */

/* === Macros === */

macro_rules! host_contracts {
    (
        $( $stream:ident { id: $id:literal, label: $label:literal, columns: {
            $( $field:ident : $value_type:ident, $role:ident, $column_label:literal, $ty:ty $(, unit: $unit:literal)?; )*
        } } )*
    ) => {
        $(
            #[dokime::message]
            #[derive(Clone, PartialEq)]
            struct $stream { $( $field: $ty, )* }
        )*

        pub fn host_telemetry_contract(
            stream: HostTelemetryStream,
        ) -> TelemetryContract {
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
                                $(.with_unit(MeasurementUnit::custom(identifier($unit), $unit, $unit)))?
                            ),*
                        ],
                        default_x: Some("timestamp".to_owned()),
                        interval: None,
                        allow_extra_fields: true,
                        scalar: false,
                        validation_mode: None,
                        foreign: false,
                    }.normalized().typed::<$stream>(),
                )*
            }
        }
    };
}

super::table::host_streams!(host_contracts);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn host_telemetry_contracts() -> Vec<TelemetryContract> {
    HOST_STREAMS.iter().copied().map(host_telemetry_contract).collect()
}

/* === Private Functions === */

fn column(id: &str, label: &str, value_type: ValueType, role: TelemetryRole) -> TelemetryColumn {
    TelemetryColumn::new(Metadata::new(identifier(id), label, None), value_type, role).optional()
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).expect("host telemetry identifiers are valid")
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::schema::{Cardinality, graph::Kind};

    #[test]
    fn native_row_schemas() {
        for contract in host_telemetry_contracts() {
            assert_eq!(contract.schema_version, 1);
            assert_eq!(contract.columns.iter().filter(|column| column.metadata.identifier.as_str() == "timestamp").count(), 1);
            let graph = contract.storage_schema.as_ref().unwrap();
            assert!(graph.root().unwrap().fields().iter().all(|field| field.name != "timestamp"));
        }

        let contract = host_telemetry_contract(HostTelemetryStream::Bluetooth);
        let graph = contract.storage_schema.unwrap();
        let field = graph.root().unwrap().fields().iter().find(|field| field.name == "bluetooth_device_names").unwrap();
        assert_eq!(field.cardinality, Cardinality::Repeated);
        assert_eq!(field.kind, Kind::String);
    }
}
