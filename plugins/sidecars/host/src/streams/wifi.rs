/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Wi-Fi Collections: the Link, the Scan and the Latency Probe, Each One Native Measurement per Tick

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Host;
use crate::collectors::wifi::{LatencyOptions, LinkOptions, ScanOptions};
use crate::domain::readings;
use crate::domain::telemetry::{
    WIFI_LATENCY_PROBE, WIFI_LINK_PROBE, WIFI_SCAN_PROBE, WifiAdapter, WifiCipher, WifiCongestion, WifiLatency, WifiLink, WifiLoss,
    WifiNetwork, WifiNetworkName, WifiNetworkRssi, WifiPhyRate, WifiProfile, WifiRssi, WifiSecurity, WifiSignal, WifiThroughput,
};
use crate::settings;
use crate::streams;
use dokime::operations::Context;
use dokime::plugin::{Collection, CollectionOutput, Plugin};
use dokime::{Error, Result, setting_value};
use harmos::plugin::Stream;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Link, Scan and Latency Collections and Every Stream They Feed.
pub fn catalog(plugin: Plugin, host: Arc<Host>) -> Result<Plugin> {
    let link = Collection::operation({
        let host = host.clone();
        async move |transport, rows: CollectionOutput| {
            let ctx = Context::from_collection(transport, host.clone())?;
            collect_link(&ctx, &rows).await.map_err(Into::into)
        }
    })
    .stream(streams::probe(WIFI_LINK_PROBE))
    .stream(Stream::<WifiAdapter>::new(WifiAdapter::STREAM).description(WifiAdapter::LABEL))
    .stream(Stream::<WifiLink>::new(WifiLink::STREAM).description(WifiLink::LABEL))
    .stream(Stream::<WifiSecurity>::new(WifiSecurity::STREAM).description(WifiSecurity::LABEL))
    .stream(Stream::<WifiCipher>::new(WifiCipher::STREAM).description(WifiCipher::LABEL))
    .stream(Stream::<WifiProfile>::new(WifiProfile::STREAM).description(WifiProfile::LABEL))
    .stream(Stream::<WifiSignal>::new(WifiSignal::STREAM).description(WifiSignal::LABEL))
    .stream(Stream::<WifiRssi>::new(WifiRssi::STREAM).description(WifiRssi::LABEL))
    .stream(Stream::<WifiPhyRate>::new(WifiPhyRate::STREAM).description(WifiPhyRate::LABEL))
    .stream(Stream::<WifiThroughput>::new(WifiThroughput::STREAM).description(WifiThroughput::LABEL))
    .interval_setting(settings::WIFI_LINK_INTERVAL)
    .on_start({
        let host = host.clone();
        async move |_| host.wifi.start_link().map_err(Into::into)
    })
    .on_stop({
        let host = host.clone();
        async move |_| {
            host.wifi.stop_link();
            Ok(())
        }
    });
    let scan = Collection::operation({
        let host = host.clone();
        async move |transport, rows: CollectionOutput| {
            let ctx = Context::from_collection(transport, host.clone())?;
            collect_scan(&ctx, &rows).await.map_err(Into::into)
        }
    })
    .stream(streams::probe(WIFI_SCAN_PROBE))
    .stream(Stream::<WifiNetwork>::new(WifiNetwork::STREAM).description(WifiNetwork::LABEL))
    .stream(Stream::<WifiNetworkName>::new(WifiNetworkName::STREAM).description(WifiNetworkName::LABEL))
    .stream(Stream::<WifiNetworkRssi>::new(WifiNetworkRssi::STREAM).description(WifiNetworkRssi::LABEL))
    .stream(Stream::<WifiCongestion>::new(WifiCongestion::STREAM).description(WifiCongestion::LABEL))
    .interval_setting(settings::WIFI_SCAN_INTERVAL)
    .on_stop({
        let host = host.clone();
        async move |_| {
            host.wifi.stop_scan();
            Ok(())
        }
    });
    let latency = Collection::operation({
        let host = host.clone();
        async move |transport, rows: CollectionOutput| {
            let ctx = Context::from_collection(transport, host.clone())?;
            collect_latency(&ctx, &rows).await.map_err(Into::into)
        }
    })
    .stream(streams::probe(WIFI_LATENCY_PROBE))
    .stream(Stream::<WifiLatency>::new(WifiLatency::STREAM).description(WifiLatency::LABEL))
    .stream(Stream::<WifiLoss>::new(WifiLoss::STREAM).description(WifiLoss::LABEL))
    .interval_setting(settings::WIFI_LATENCY_INTERVAL);
    Ok(plugin.collection(link).collection(scan).collection(latency))
}

/// Publish the Link Groups Whose Fields Were All Read; the Outcome Row Always.
pub fn publish_link(reading: readings::WifiLink, rows: &CollectionOutput) {
    streams::outcome(rows, WIFI_LINK_PROBE, reading.probe_status, reading.probe_error);
    if let (Some(wifi_adapter), Some(wifi_state), Some(associated)) = (reading.wifi_adapter, reading.wifi_state, reading.associated) {
        rows.publish(&Stream::new(WifiAdapter::STREAM), WifiAdapter { wifi_adapter, wifi_state, associated });
    }
    if let (Some(wifi_ssid), Some(wifi_bssid), Some(wifi_channel), Some(wifi_band)) =
        (reading.wifi_ssid, reading.wifi_bssid, reading.wifi_channel, reading.wifi_band)
    {
        rows.publish(
            &Stream::new(WifiLink::STREAM),
            WifiLink {
                wifi_ssid,
                wifi_bssid,
                wifi_channel,
                wifi_band,
            },
        );
    }
    if let Some(wifi_auth) = reading.wifi_auth {
        rows.publish(&Stream::new(WifiSecurity::STREAM), WifiSecurity { wifi_auth });
    }
    if let (Some(wifi_radio), Some(wifi_cipher)) = (reading.wifi_radio, reading.wifi_cipher) {
        rows.publish(&Stream::new(WifiCipher::STREAM), WifiCipher { wifi_radio, wifi_cipher });
    }
    if let Some(wifi_profile) = reading.wifi_profile {
        rows.publish(&Stream::new(WifiProfile::STREAM), WifiProfile { wifi_profile });
    }
    if let Some(wifi_signal_percent) = reading.wifi_signal_percent {
        rows.publish(&Stream::new(WifiSignal::STREAM), WifiSignal { wifi_signal_percent });
    }
    if let Some(wifi_rssi_dbm) = reading.wifi_rssi_dbm {
        rows.publish(&Stream::new(WifiRssi::STREAM), WifiRssi { wifi_rssi_dbm });
    }
    if let (Some(wifi_rx_phy_mbps), Some(wifi_tx_phy_mbps)) = (reading.wifi_rx_phy_mbps, reading.wifi_tx_phy_mbps) {
        rows.publish(&Stream::new(WifiPhyRate::STREAM), WifiPhyRate { wifi_rx_phy_mbps, wifi_tx_phy_mbps });
    }
    if let (Some(wifi_rx_mbps), Some(wifi_tx_mbps)) = (reading.wifi_rx_mbps, reading.wifi_tx_mbps) {
        rows.publish(&Stream::new(WifiThroughput::STREAM), WifiThroughput { wifi_rx_mbps, wifi_tx_mbps });
    }
}

/// Publish One Outcome and Congestion Row per Scan, and Each Network's Available Measurements.
pub fn publish_scan(readings: Vec<readings::WifiScan>, rows: &CollectionOutput) {
    let mut first = readings.first().cloned().unwrap_or_default();
    streams::outcome(rows, WIFI_SCAN_PROBE, first.scan_status.take(), first.scan_error.take());
    let congestion = WifiCongestion::from(first);
    rows.publish(&Stream::new(WifiCongestion::STREAM), congestion);
    for reading in readings {
        let Some(wifi_bssid) = reading.wifi_bssid else {
            continue;
        };
        if let (Some(wifi_signal_percent), Some(wifi_channel), Some(wifi_band), Some(associated), Some(network_count)) =
            (reading.wifi_signal_percent, reading.wifi_channel, reading.wifi_band, reading.associated, reading.network_count)
        {
            let row = WifiNetwork {
                wifi_bssid: wifi_bssid.clone(),
                wifi_signal_percent,
                wifi_channel,
                wifi_band,
                associated,
                network_count,
            };
            rows.publish(&Stream::new(WifiNetwork::STREAM), row);
        }
        if let (Some(wifi_ssid), Some(wifi_security)) = (reading.wifi_ssid, reading.wifi_security) {
            rows.publish(
                &Stream::new(WifiNetworkName::STREAM),
                WifiNetworkName {
                    wifi_bssid: wifi_bssid.clone(),
                    wifi_ssid,
                    wifi_security,
                },
            );
        }
        if let Some(wifi_rssi_dbm) = reading.wifi_rssi_dbm {
            rows.publish(&Stream::new(WifiNetworkRssi::STREAM), WifiNetworkRssi { wifi_bssid, wifi_rssi_dbm });
        }
    }
}

/// Publish the Outcome, the Round Trip of a Reply and the Loss of Every Echo That Ran.
pub fn publish_latency(reading: readings::WifiLatency, rows: &CollectionOutput) {
    streams::outcome(rows, WIFI_LATENCY_PROBE, reading.probe_status, reading.probe_error);
    let Some(wifi_latency_host) = reading.wifi_latency_host else {
        return;
    };
    if let Some(wifi_latency_ms) = reading.wifi_latency_ms {
        rows.publish(
            &Stream::new(WifiLatency::STREAM),
            WifiLatency {
                wifi_latency_host: wifi_latency_host.clone(),
                wifi_latency_ms,
            },
        );
    }
    if let Some(wifi_packet_loss_percent) = reading.wifi_packet_loss_percent {
        rows.publish(
            &Stream::new(WifiLoss::STREAM),
            WifiLoss {
                wifi_latency_host,
                wifi_packet_loss_percent,
            },
        );
    }
}

/* === Private Functions === */

async fn collect_link(ctx: &Context, rows: &CollectionOutput) -> Result<()> {
    let options = LinkOptions {
        adapter: settings::selected(ctx, settings::WIFI_ADAPTER).await?,
        probe_timeout: ctx.duration_setting(settings::WIFI_LINK_PROBE_TIMEOUT).await?,
        minimum_rate_window: ctx.duration_setting(settings::MINIMUM_RATE_WINDOW).await?,
    };
    let reading = ctx.plugin::<Arc<Host>>()?.wifi.link(options).await?;
    publish_link(reading, rows);
    Ok(())
}

async fn collect_scan(ctx: &Context, rows: &CollectionOutput) -> Result<()> {
    let options = ScanOptions {
        adapter: settings::selected(ctx, settings::WIFI_ADAPTER).await?,
        probe_timeout: ctx.duration_setting(settings::WIFI_SCAN_PROBE_TIMEOUT).await?,
        window: ctx.duration_setting(settings::WIFI_SCAN_WINDOW).await?,
        minimum_interval: ctx.duration_setting(settings::MINIMUM_SCAN_INTERVAL).await?,
        congestion_weight: settings::number(ctx, settings::CONGESTION_NETWORK_WEIGHT).await?,
    };
    let readings = ctx.plugin::<Arc<Host>>()?.wifi.scan(options).await?;
    publish_scan(readings, rows);
    Ok(())
}

async fn collect_latency(ctx: &Context, rows: &CollectionOutput) -> Result<()> {
    let host = match settings::selected(ctx, settings::LATENCY_HOST).await? {
        Some(host) => host,
        None => match settings::setting(settings::LATENCY_HOST)?.default_value().value {
            Some(setting_value::Value::Text(host)) => host,
            _ => return Err(Error::Invalid("Read Setting Latency Host: Declare a Text Default".into())),
        },
    };
    let payload_bytes = settings::integer(ctx, settings::ECHO_PAYLOAD_BYTES).await?;
    let options = LatencyOptions {
        host,
        reply_timeout: ctx.duration_setting(settings::ECHO_REPLY_TIMEOUT).await?,
        process_timeout: ctx.duration_setting(settings::LATENCY_PROCESS_TIMEOUT).await?,
        payload_bytes: u32::try_from(payload_bytes)
            .map_err(|_| Error::Invalid("Read Setting Echo Payload: Configure a Size Below 4 GiB in the Plugin Settings".into()))?,
    };
    let reading = ctx.plugin::<Arc<Host>>()?.wifi.latency(options).await?;
    publish_latency(reading, rows);
    Ok(())
}

/* === Tests === */
