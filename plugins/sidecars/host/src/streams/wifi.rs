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
use dokime::plugin::{Collection, Plugin, Rows, Stream};
use dokime::{Error, Result, setting_value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Link, Scan and Latency Collections and Every Stream They Feed.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let link = Collection::operation(collect_link)
        .stream(streams::probe(WIFI_LINK_PROBE))
        .stream(Stream::<WifiAdapter>::new(WifiAdapter::STREAM, WifiAdapter::LABEL))
        .stream(Stream::<WifiLink>::new(WifiLink::STREAM, WifiLink::LABEL))
        .stream(Stream::<WifiSecurity>::new(WifiSecurity::STREAM, WifiSecurity::LABEL))
        .stream(Stream::<WifiCipher>::new(WifiCipher::STREAM, WifiCipher::LABEL))
        .stream(Stream::<WifiProfile>::new(WifiProfile::STREAM, WifiProfile::LABEL))
        .stream(Stream::<WifiSignal>::new(WifiSignal::STREAM, WifiSignal::LABEL))
        .stream(Stream::<WifiRssi>::new(WifiRssi::STREAM, WifiRssi::LABEL))
        .stream(Stream::<WifiPhyRate>::new(WifiPhyRate::STREAM, WifiPhyRate::LABEL))
        .stream(Stream::<WifiThroughput>::new(WifiThroughput::STREAM, WifiThroughput::LABEL))
        .interval_setting(settings::WIFI_LINK_INTERVAL)
        .on_stop(|ctx: &Context| {
            ctx.plugin::<Host>()?.wifi.stop_link();
            Ok(())
        });
    let scan = Collection::operation(collect_scan)
        .stream(streams::probe(WIFI_SCAN_PROBE))
        .stream(Stream::<WifiNetwork>::new(WifiNetwork::STREAM, WifiNetwork::LABEL))
        .stream(Stream::<WifiNetworkName>::new(WifiNetworkName::STREAM, WifiNetworkName::LABEL))
        .stream(Stream::<WifiNetworkRssi>::new(WifiNetworkRssi::STREAM, WifiNetworkRssi::LABEL))
        .stream(Stream::<WifiCongestion>::new(WifiCongestion::STREAM, WifiCongestion::LABEL))
        .interval_setting(settings::WIFI_SCAN_INTERVAL)
        .on_stop(|ctx: &Context| {
            ctx.plugin::<Host>()?.wifi.stop_scan();
            Ok(())
        });
    let latency = Collection::operation(collect_latency)
        .stream(streams::probe(WIFI_LATENCY_PROBE))
        .stream(Stream::<WifiLatency>::new(WifiLatency::STREAM, WifiLatency::LABEL))
        .stream(Stream::<WifiLoss>::new(WifiLoss::STREAM, WifiLoss::LABEL))
        .interval_setting(settings::WIFI_LATENCY_INTERVAL);
    Ok(plugin.collection(link).collection(scan).collection(latency))
}

/// Publish the Link Groups Whose Fields Were All Read; the Outcome Row Always.
pub fn publish_link(reading: readings::WifiLink, rows: &mut Rows) -> Result<()> {
    streams::outcome(rows, WIFI_LINK_PROBE, reading.probe_status, reading.probe_error)?;
    if let (Some(wifi_adapter), Some(wifi_state), Some(associated)) = (reading.wifi_adapter, reading.wifi_state, reading.associated) {
        rows.publish(WifiAdapter::STREAM, WifiAdapter { wifi_adapter, wifi_state, associated })?;
    }
    if let (Some(wifi_ssid), Some(wifi_bssid), Some(wifi_channel), Some(wifi_band)) =
        (reading.wifi_ssid, reading.wifi_bssid, reading.wifi_channel, reading.wifi_band)
    {
        rows.publish(
            WifiLink::STREAM,
            WifiLink {
                wifi_ssid,
                wifi_bssid,
                wifi_channel,
                wifi_band,
            },
        )?;
    }
    if let Some(wifi_auth) = reading.wifi_auth {
        rows.publish(WifiSecurity::STREAM, WifiSecurity { wifi_auth })?;
    }
    if let (Some(wifi_radio), Some(wifi_cipher)) = (reading.wifi_radio, reading.wifi_cipher) {
        rows.publish(WifiCipher::STREAM, WifiCipher { wifi_radio, wifi_cipher })?;
    }
    if let Some(wifi_profile) = reading.wifi_profile {
        rows.publish(WifiProfile::STREAM, WifiProfile { wifi_profile })?;
    }
    if let Some(wifi_signal_percent) = reading.wifi_signal_percent {
        rows.publish(WifiSignal::STREAM, WifiSignal { wifi_signal_percent })?;
    }
    if let Some(wifi_rssi_dbm) = reading.wifi_rssi_dbm {
        rows.publish(WifiRssi::STREAM, WifiRssi { wifi_rssi_dbm })?;
    }
    if let (Some(wifi_rx_phy_mbps), Some(wifi_tx_phy_mbps)) = (reading.wifi_rx_phy_mbps, reading.wifi_tx_phy_mbps) {
        rows.publish(WifiPhyRate::STREAM, WifiPhyRate { wifi_rx_phy_mbps, wifi_tx_phy_mbps })?;
    }
    if let (Some(wifi_rx_mbps), Some(wifi_tx_mbps)) = (reading.wifi_rx_mbps, reading.wifi_tx_mbps) {
        rows.publish(WifiThroughput::STREAM, WifiThroughput { wifi_rx_mbps, wifi_tx_mbps })?;
    }
    Ok(())
}

/// Publish One Outcome per Scan, One Row per Network per Stream Its Fields Fill, and the Associated Channel's Congestion Once.
pub fn publish_scan(readings: Vec<readings::WifiScan>, rows: &mut Rows) -> Result<()> {
    let first = readings.first().cloned().unwrap_or_default();
    streams::outcome(rows, WIFI_SCAN_PROBE, first.scan_status, first.scan_error)?;
    let congestion = (
        first.scan_network_count,
        first.active_wifi_channel,
        first.active_wifi_band,
        first.active_wifi_ssid,
        first.wifi_network_congestion_score,
        first.congestion_model,
    );
    if let (
        Some(scan_network_count),
        Some(active_wifi_channel),
        Some(active_wifi_band),
        Some(active_wifi_ssid),
        Some(wifi_network_congestion_score),
        Some(congestion_model),
    ) = congestion
    {
        let row = WifiCongestion {
            scan_network_count,
            active_wifi_channel,
            active_wifi_band,
            active_wifi_ssid,
            wifi_network_congestion_score,
            congestion_model,
        };
        rows.publish(WifiCongestion::STREAM, row)?;
    }
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
            rows.publish(WifiNetwork::STREAM, row)?;
        }
        if let (Some(wifi_ssid), Some(wifi_security)) = (reading.wifi_ssid, reading.wifi_security) {
            rows.publish(
                WifiNetworkName::STREAM,
                WifiNetworkName {
                    wifi_bssid: wifi_bssid.clone(),
                    wifi_ssid,
                    wifi_security,
                },
            )?;
        }
        if let Some(wifi_rssi_dbm) = reading.wifi_rssi_dbm {
            rows.publish(WifiNetworkRssi::STREAM, WifiNetworkRssi { wifi_bssid, wifi_rssi_dbm })?;
        }
    }
    Ok(())
}

/// Publish the Outcome, the Round Trip of a Reply and the Loss of Every Echo That Ran.
pub fn publish_latency(reading: readings::WifiLatency, rows: &mut Rows) -> Result<()> {
    streams::outcome(rows, WIFI_LATENCY_PROBE, reading.probe_status, reading.probe_error)?;
    let Some(wifi_latency_host) = reading.wifi_latency_host else {
        return Ok(());
    };
    if let Some(wifi_latency_ms) = reading.wifi_latency_ms {
        rows.publish(
            WifiLatency::STREAM,
            WifiLatency {
                wifi_latency_host: wifi_latency_host.clone(),
                wifi_latency_ms,
            },
        )?;
    }
    if let Some(wifi_packet_loss_percent) = reading.wifi_packet_loss_percent {
        rows.publish(
            WifiLoss::STREAM,
            WifiLoss {
                wifi_latency_host,
                wifi_packet_loss_percent,
            },
        )?;
    }
    Ok(())
}

/* === Private Functions === */

async fn collect_link(ctx: &Context, rows: &mut Rows) -> Result<()> {
    let options = LinkOptions {
        adapter: settings::selected(ctx, settings::WIFI_ADAPTER).await?,
        probe_timeout: settings::duration_value(ctx, settings::WIFI_LINK_PROBE_TIMEOUT).await?,
        minimum_rate_window: settings::duration_value(ctx, settings::MINIMUM_RATE_WINDOW).await?,
    };
    let reading = ctx.plugin::<Host>()?.wifi.link(options).await?;
    publish_link(reading, rows)
}

async fn collect_scan(ctx: &Context, rows: &mut Rows) -> Result<()> {
    let options = ScanOptions {
        adapter: settings::selected(ctx, settings::WIFI_ADAPTER).await?,
        probe_timeout: settings::duration_value(ctx, settings::WIFI_SCAN_PROBE_TIMEOUT).await?,
        window: settings::duration_value(ctx, settings::WIFI_SCAN_WINDOW).await?,
        minimum_interval: settings::duration_value(ctx, settings::MINIMUM_SCAN_INTERVAL).await?,
        congestion_weight: settings::number(ctx, settings::CONGESTION_NETWORK_WEIGHT).await?,
    };
    let readings = ctx.plugin::<Host>()?.wifi.scan(options).await?;
    publish_scan(readings, rows)
}

async fn collect_latency(ctx: &Context, rows: &mut Rows) -> Result<()> {
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
        reply_timeout: settings::duration_value(ctx, settings::ECHO_REPLY_TIMEOUT).await?,
        process_timeout: settings::duration_value(ctx, settings::LATENCY_PROCESS_TIMEOUT).await?,
        payload_bytes: u32::try_from(payload_bytes)
            .map_err(|_| Error::Invalid("Read Setting Echo Payload: Configure a Size Below 4 GiB in the Plugin Settings".into()))?,
    };
    let reading = ctx.plugin::<Host>()?.wifi.latency(options).await?;
    publish_latency(reading, rows)
}

/* === Tests === */
