/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime_domain::model;
use dokime_host::{HOST_STREAMS, Host, HostTelemetryStream, host_telemetry_contracts};
use dokime_runtime::telemetry::TelemetryRow;
use dokime_runtime::telemetry::validate_telemetry_sample;
use model::telemetry::ValidationMode;

#[test]
fn host_cadence_matches_contracts() {
    let host = Host::fixture();
    let collector = dokime_host::host_telemetry_collector(Default::default());
    let mut plugin = dokime_host::installed_plugin();
    let expected = plugin.clone();
    dokime_host::configure_contracts(&mut plugin);
    assert_eq!(plugin, expected);
    assert_eq!(plugin.source, dokime_host::HOST_PLUGIN_SOURCE);
    assert_eq!(dokime_host::HOST_SOURCE_ID, "host.telemetry");

    for stream in HOST_STREAMS {
        let contract = plugin.telemetry_contracts.iter().find(|entry| entry.stream.as_str() == stream.as_str()).unwrap();
        let interval = contract.interval.as_ref().unwrap().0;
        assert_eq!(collector.interval(stream.as_str()), std::time::Duration::from_millis(interval));
        let batch = host.collect_stream(*stream).unwrap().unwrap();
        assert_eq!(batch.stream, contract.stream);
    }
}

#[test]
fn fixture_host_stream() {
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

#[test]
fn fixture_host_contracts() {
    let host = Host::fixture();
    let contracts = host_telemetry_contracts();

    for stream in HOST_STREAMS {
        let batch = host.collect_stream(*stream).unwrap().unwrap();
        let contract = contracts.iter().find(|contract| contract.stream == batch.stream).unwrap();

        for sample in batch.samples {
            validate_telemetry_sample(Some(contract), &sample, ValidationMode::Strict).unwrap();
        }
    }
}

#[test]
fn collected_row_stream() {
    fn reads<C: dokime_runtime::telemetry::TelemetryStream>(host: &Host, stream: HostTelemetryStream) {
        let batch = host.collect_stream(stream).unwrap().unwrap();
        assert!(!batch.samples.is_empty(), "{} collected nothing", C::ID);
        for sample in batch.samples {
            <C::Row as dokime_runtime::telemetry::TelemetryRow>::from_row(&sample.values)
                .unwrap_or_else(|error| panic!("{}: {error}", C::ID));
        }
    }

    let host = Host::fixture();
    reads::<dokime_runtime::host::System>(&host, HostTelemetryStream::System);
    reads::<dokime_runtime::host::Network>(&host, HostTelemetryStream::Network);
    reads::<dokime_runtime::host::Wifi>(&host, HostTelemetryStream::Wifi);
    reads::<dokime_runtime::host::WifiLatency>(&host, HostTelemetryStream::WifiLatency);
    reads::<dokime_runtime::host::WifiChannelOccupancy>(&host, HostTelemetryStream::WifiChannelOccupancy);
    reads::<dokime_runtime::host::Bluetooth>(&host, HostTelemetryStream::Bluetooth);
}

#[test]
fn fixture_battery_columns() {
    let host = Host::fixture();
    let batch = host.collect_stream(HostTelemetryStream::System).unwrap().unwrap();
    let row = dokime_runtime::host::System::from_row(&batch.samples[0].values).unwrap();

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

#[test]
fn fixture_channel_columns() {
    let host = Host::fixture();
    let batch = host.collect_stream(HostTelemetryStream::WifiChannelOccupancy).unwrap().unwrap();
    let row = dokime_runtime::host::WifiChannelOccupancy::from_row(&batch.samples[0].values).unwrap();

    assert_eq!(row.active_channel, Some(true));
    assert_eq!(row.active_wifi_channel, Some(6));
    assert_eq!(row.active_wifi_band.as_deref(), Some("2.4GHz"));
}
