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
use std::collections::BTreeSet;
// Third Party Imports

// Local Imports
use crate::model::document::DocumentCard;
use crate::model::document::DocumentCardBody;
use crate::model::document::DocumentSection;
use crate::model::document::Plot;
use crate::model::document::PlotDefinition;
use crate::model::document::PlotStyle;
use crate::model::document::PlotType;
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;

/* === Constants === */
const HOST_SYSTEM: &str = "host.system";
const HOST_WIFI: &str = "host.wifi";
const HOST_WIFI_LATENCY: &str = "host.wifi.latency";
const HOST_WIFI_CHANNEL_OCCUPANCY: &str = "host.wifi.channel_occupancy";
const HOST_BLUETOOTH: &str = "host.bluetooth";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct HostPlotSpec<'a> {
    identifier: &'a str,
    label: &'a str,
    description: &'a str,
    plot_type: PlotType,
    stream: &'a str,
    x: Option<&'a str>,
    y: &'a [&'a str],
    unit: Option<&'a str>,
    style: PlotStyle,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn sections(tracking_requested: bool) -> Vec<DocumentSection> {
    let streams = BTreeSet::from([
        HOST_SYSTEM,
        HOST_WIFI,
        HOST_WIFI_LATENCY,
        HOST_WIFI_CHANNEL_OCCUPANCY,
        HOST_BLUETOOTH,
    ]);
    vec![
        wifi_section(&streams),
        wifi_congestion_section(&streams, tracking_requested),
        bluetooth_section(),
        system_section(),
    ]
}

/* === Private Functions === */
fn wifi_section(streams: &BTreeSet<&str>) -> DocumentSection {
    let mut cards = Vec::new();
    if streams.contains(HOST_WIFI_LATENCY) {
        cards.push(plot_card(plot(HostPlotSpec {
            identifier: "hub.host.wifi.latency",
            label: "WiFi Latency",
            description: "Round-trip latency from the host to the configured latency probe, measured in milliseconds.",
            plot_type: PlotType::TimeSeries,
            stream: HOST_WIFI_LATENCY,
            x: Some("timestamp"),
            y: &["wifi_latency_ms"],
            unit: Some("ms"),
            style: PlotStyle::Line,
        })));
    }
    if streams.contains(HOST_WIFI_CHANNEL_OCCUPANCY) {
        cards.push(plot_card(plot(HostPlotSpec {
            identifier: "hub.host.wifi.stream",
            label: "WiFi Channel",
            description: "Active WiFi stream derived from the stream occupancy scan.",
            plot_type: PlotType::TimeSeries,
            stream: HOST_WIFI_CHANNEL_OCCUPANCY,
            x: Some("timestamp"),
            y: &["active_wifi_channel"],
            unit: Some("stream"),
            style: PlotStyle::Line,
        })));
    } else if streams.contains(HOST_WIFI) {
        cards.push(plot_card(plot(HostPlotSpec {
            identifier: "hub.host.wifi.stream",
            label: "WiFi Channel",
            description: "Active WiFi stream reported by the host WiFi adapter.",
            plot_type: PlotType::TimeSeries,
            stream: HOST_WIFI,
            x: Some("timestamp"),
            y: &["wifi_channel"],
            unit: Some("stream"),
            style: PlotStyle::Line,
        })));
    }
    if streams.contains(HOST_WIFI) {
        cards.extend([
            plot_card(plot(HostPlotSpec {
                identifier: "hub.host.wifi.signal",
                label: "WiFi Signal",
                description: "Current signal quality reported by the host WiFi adapter.",
                plot_type: PlotType::TimeSeries,
                stream: HOST_WIFI,
                x: Some("timestamp"),
                y: &["wifi_signal_percent"],
                unit: Some("%"),
                style: PlotStyle::Line,
            })),
            plot_card(plot(HostPlotSpec {
                identifier: "hub.host.wifi.phy",
                label: "WiFi PHY",
                description: "Current receive and transmit PHY rates reported by the host WiFi adapter.",
                plot_type: PlotType::TimeSeries,
                stream: HOST_WIFI,
                x: Some("timestamp"),
                y: &["wifi_rx_phy_mbps", "wifi_tx_phy_mbps"],
                unit: Some("Mbps"),
                style: PlotStyle::Line,
            })),
            plot_card(plot(HostPlotSpec {
                identifier: "hub.host.wifi.throughput",
                label: "Network Adapter Throughput",
                description: "Receive and transmit throughput calculated from the host network adapter byte counters.",
                plot_type: PlotType::TimeSeries,
                stream: HOST_WIFI,
                x: Some("timestamp"),
                y: &["wifi_rx_mbps", "wifi_tx_mbps"],
                unit: Some("Mbps"),
                style: PlotStyle::Line,
            })),
        ]);
    }

    section("hub.host.network_wifi", "Network / WiFi", cards)
}

fn wifi_congestion_section(streams: &BTreeSet<&str>, tracking_requested: bool) -> DocumentSection {
    let mut cards = Vec::new();
    if streams.contains(HOST_WIFI_CHANNEL_OCCUPANCY) {
        cards.push(plot_card(plot(HostPlotSpec {
            identifier: "hub.host.wifi.congestion_index",
            label: "WiFi Congestion Index",
            description: "Custom 0-100% KPI computed from networks on the active host WiFi stream plus adjacent overlapping networks weighted by stream overlap.",
            plot_type: PlotType::TimeSeries,
            stream: HOST_WIFI_CHANNEL_OCCUPANCY,
            x: Some("timestamp"),
            y: &["wifi_network_congestion_score"],
            unit: Some("%"),
            style: PlotStyle::Line,
        })));
    }
    if streams.contains(HOST_WIFI_CHANNEL_OCCUPANCY) {
        let mut occupancy = Plot::wifi_channel_occupancy("WiFi Channel Occupancy");
        occupancy.id("hub.host.wifi.channel_occupancy");
        occupancy.metadata.description = Some(
            "Full WLAN scans count nearby networks per stream across WiFi bands. This scan is intentionally slower than PHY, throughput, and latency sampling.".to_owned(),
        );
        cards.push(plot_card(occupancy));
    }
    if tracking_requested && streams.contains(HOST_WIFI) {
        let mut congestion_map = plot(HostPlotSpec {
            identifier: "hub.host.wifi.congestion_map",
            label: "WiFi Network Congestion Map",
            description: "Tracking XY positions joined with the latest available WiFi Congestion Index at each observation so congestion can be reviewed spatially.",
            plot_type: PlotType::XyValueHeatmap,
            stream: "tracking",
            x: Some("position_x_m"),
            y: &["position_y_m"],
            unit: Some("%"),
            style: PlotStyle::Points,
        });
        congestion_map.z_stream = Some(HOST_WIFI.to_owned());
        congestion_map.z_field = Some("wifi_network_congestion_score".to_owned());
        cards.push(plot_card(congestion_map));
    }

    section(
        "hub.host.network_wifi_congestion",
        "Network / WiFi Congestion",
        cards,
    )
}

fn bluetooth_section() -> DocumentSection {
    section(
        "hub.host.network_bluetooth",
        "Network / Bluetooth",
        vec![plot_card(plot(HostPlotSpec {
            identifier: "hub.host.bluetooth.devices",
            label: "Host Bluetooth",
            description: "Host Bluetooth adapter and device counts captured during the execution.",
            plot_type: PlotType::TimeSeries,
            stream: HOST_BLUETOOTH,
            x: Some("timestamp"),
            y: &[
                "bluetooth_adapter_count",
                "bluetooth_connected_device_count",
                "bluetooth_present_device_count",
            ],
            unit: None,
            style: PlotStyle::Line,
        }))],
    )
}

fn system_section() -> DocumentSection {
    section(
        "hub.host.system",
        "System Performance",
        vec![plot_card(plot(HostPlotSpec {
            identifier: "hub.host.system.resources",
            label: "Host System",
            description: "Host CPU, memory pressure, battery, and power profile context captured during the execution.",
            plot_type: PlotType::TimeSeries,
            stream: HOST_SYSTEM,
            x: Some("timestamp"),
            y: &[
                "cpu_load_percent",
                "memory_pressure_percent",
                "battery_percent",
            ],
            unit: Some("%"),
            style: PlotStyle::Line,
        }))],
    )
}

fn section(identifier_value: &str, label: &str, cards: Vec<DocumentCard>) -> DocumentSection {
    DocumentSection {
        metadata: simple_metadata(identifier_value, label),
        cards,
        repeat: None,
    }
}

fn plot_card(plot: PlotDefinition) -> DocumentCard {
    DocumentCard {
        metadata: plot.metadata.clone(),
        layout: Default::default(),
        filters: Vec::new(),
        body: DocumentCardBody::Plot {
            plot: Box::new(plot),
        },
    }
}

fn plot(spec: HostPlotSpec<'_>) -> PlotDefinition {
    PlotDefinition {
        automatic_id: false,
        metadata: metadata(spec.identifier, spec.label, spec.description),
        plot_type: spec.plot_type,
        stream: Some(spec.stream.to_owned()),
        x: spec.x.map(ToOwned::to_owned),
        y: spec.y.iter().map(|field| (*field).to_owned()).collect(),
        z_stream: None,
        z_field: None,
        series_by: None,
        x_expression: None,
        y_expressions: Vec::new(),
        z_expression: None,
        series_by_expression: None,
        filters: Vec::new(),
        unit: spec.unit.map(ToOwned::to_owned),
        style: spec.style,
        x_scale: Default::default(),
        y_scale: Default::default(),
        analysis: None,
        fft: Default::default(),
        interval: Default::default(),
        histogram: Default::default(),
        marker_record_types: Vec::new(),
        reference_lines: Vec::new(),
        reference_bands: Vec::new(),
        live_window_s: Some(120.0),
        display_points: None,
        downsample: Default::default(),
        x_format: None,
        y_format: None,
        x_axis: None,
    }
}

fn simple_metadata(identifier_value: &str, label: &str) -> Metadata {
    metadata(identifier_value, label, label)
}

fn metadata(identifier_value: &str, label: &str, description: &str) -> Metadata {
    Metadata::definition(
        identifier(identifier_value),
        label,
        Some(description.to_owned()),
        std::iter::empty::<String>(),
    )
}

fn identifier(identifier_value: &str) -> Identifier {
    Identifier::new(identifier_value).expect("host document identifier is valid")
}

/* === Tests === */
