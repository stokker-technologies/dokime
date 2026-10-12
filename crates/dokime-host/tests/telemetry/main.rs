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
use dokime::telemetry::validate_telemetry_sample;
use dokime_contract::telemetry::ValidationMode;
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
}
