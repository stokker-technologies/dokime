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
use dokime::telemetry::TelemetryRow;
use dokime::telemetry::validate_telemetry_sample;
use dokime_contract::telemetry::{ValidationMode, host_telemetry_contract};
use dokime_host::{HOST_STREAMS, Host, HostTelemetryStream, host_telemetry_contracts};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) fn fixture_host_stream() {
    let host = Host::fixture();
    let streams = HOST_STREAMS
        .iter()
        .copied()
        .filter_map(|stream| host.collect_stream(stream).unwrap())
        .map(|batch| batch.stream.to_string())
        .collect::<Vec<_>>();

    assert_eq!(streams.len(), 6);
    assert!(streams.contains(&"host.system".to_owned()));
    assert!(streams.contains(&"host.wifi.latency".to_owned()));
    assert!(streams.contains(&"host.wifi.channel_occupancy".to_owned()));
}

pub(crate) fn fixture_host_contracts() {
    let host = Host::fixture();
    let contracts = host_telemetry_contracts();

    for stream in HOST_STREAMS {
        let batch = host.collect_stream(*stream).unwrap().unwrap();
        let contract = contracts
            .iter()
            .find(|contract| contract.stream == batch.stream)
            .unwrap();

        for sample in batch.samples {
            validate_telemetry_sample(Some(contract), &sample, ValidationMode::Strict).unwrap();
        }
    }
}

pub(crate) fn collected_row_stream() {
    fn reads<C: dokime::telemetry::TelemetryStream>(host: &Host, stream: HostTelemetryStream) {
        let batch = host.collect_stream(stream).unwrap().unwrap();
        assert!(!batch.samples.is_empty(), "{} collected nothing", C::ID);
        for sample in batch.samples {
            <C::Row as dokime::telemetry::TelemetryRow>::from_row(&sample.values)
                .unwrap_or_else(|error| panic!("{}: {error}", C::ID));
        }
    }

    let host = Host::fixture();
    reads::<dokime::host::System>(&host, HostTelemetryStream::System);
    reads::<dokime::host::Network>(&host, HostTelemetryStream::Network);
    reads::<dokime::host::Wifi>(&host, HostTelemetryStream::Wifi);
    reads::<dokime::host::WifiLatency>(&host, HostTelemetryStream::WifiLatency);
    reads::<dokime::host::WifiChannelOccupancy>(&host, HostTelemetryStream::WifiChannelOccupancy);
    reads::<dokime::host::Bluetooth>(&host, HostTelemetryStream::Bluetooth);
}

pub(crate) fn fixture_battery_columns() {
    let host = Host::fixture();
    let batch = host
        .collect_stream(HostTelemetryStream::System)
        .unwrap()
        .unwrap();
    let row = dokime::host::System::from_row(&batch.samples[0].values).unwrap();

    assert_eq!(row.battery_state.as_deref(), Some("discharging"));
    assert_eq!(row.battery_technology.as_deref(), Some("lithium-ion"));
    assert_eq!(row.battery_vendor.as_deref(), Some("Fixture Cells"));
    assert_eq!(row.battery_model.as_deref(), Some("FX-5700"));
    assert_eq!(row.battery_serial_number.as_deref(), Some("FX0001"));
    assert_eq!(row.battery_state_of_charge_ratio, Some(0.84));
    assert_eq!(row.battery_energy_wh, Some(42.0));
    assert_eq!(row.battery_energy_full_wh, Some(50.0));
    assert_eq!(row.battery_energy_full_design_wh, Some(57.0));
    assert_eq!(row.battery_energy_rate_w, Some(8.5));
    assert_eq!(row.battery_voltage_v, Some(11.4));
    assert!(row.battery_state_of_health_ratio.is_some());
    assert!(row.battery_temperature_c.is_some());
    assert_eq!(row.battery_time_to_empty_s, Some(17_788.0));
    // A discharging pack has no time to full: the crate reports none.
    assert_eq!(row.battery_time_to_full_s, None);
    assert_eq!(row.battery_cycle_count, Some(212));
}

pub(crate) fn fixture_active_channel_columns() {
    let host = Host::fixture();
    let batch = host
        .collect_stream(HostTelemetryStream::WifiChannelOccupancy)
        .unwrap()
        .unwrap();
    let row = dokime::host::WifiChannelOccupancy::from_row(&batch.samples[0].values).unwrap();

    assert_eq!(row.active_channel, Some(true));
    assert_eq!(row.active_wifi_channel, Some(6));
    assert_eq!(row.active_wifi_band.as_deref(), Some("2.4GHz"));
}

pub(crate) fn collected_battery_columns() {
    // Whatever this machine reports — every reading, or none on a host
    // without a battery — is a declared, correctly typed `host.system` column.
    let batch = Host::default()
        .collect_stream(HostTelemetryStream::System)
        .unwrap()
        .unwrap();
    let contract = host_telemetry_contract(HostTelemetryStream::System);
    let declared = contract
        .columns
        .iter()
        .map(|column| column.metadata.identifier.as_str())
        .collect::<Vec<_>>();
    for sample in batch.samples {
        for key in sample
            .values
            .keys()
            .filter(|key| key.starts_with("battery_"))
        {
            assert!(declared.contains(&key.as_str()), "{key} is not declared");
        }
        validate_telemetry_sample(Some(&contract), &sample, ValidationMode::Strict).unwrap();
        dokime::host::System::from_row(&sample.values).unwrap();
    }
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: fixture_host_stream");
        fixture_host_stream();
    }
    {
        eprintln!("Scenario: fixture_host_contracts");
        fixture_host_contracts();
    }
    {
        eprintln!("Scenario: collected_row_stream");
        collected_row_stream();
    }
    {
        eprintln!("Scenario: fixture_battery_columns");
        fixture_battery_columns();
    }
    {
        eprintln!("Scenario: fixture_active_channel_columns");
        fixture_active_channel_columns();
    }
    {
        eprintln!("Scenario: collected_battery_columns");
        collected_battery_columns();
    }
}
