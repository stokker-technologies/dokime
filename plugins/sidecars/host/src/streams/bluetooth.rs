/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Bluetooth Collections: the Link Read Fresh per Tick and the Inventory, One Row per Present Device

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Host;
use crate::collectors::bluetooth::{InventoryOptions, LinkOptions};
use crate::controls::Timing;
use crate::domain::readings;
use crate::domain::telemetry::{
    BLUETOOTH_LINK_PROBE, BLUETOOTH_PROBE, Bluetooth, BluetoothConnections, BluetoothDevice, BluetoothRadios, BluetoothRssi,
    BluetoothThroughput,
};
use crate::settings;
use crate::streams;
use dokime::Result;
use dokime::operations::Context;
use dokime::plugin::{Collection, CollectionOutput, Plugin};
use harmos::plugin::Stream;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Link and Inventory Collections and Every Stream They Feed; Each Collection's State Lives From Its Start to Its Stop.
pub fn catalog(plugin: Plugin, host: Arc<Host>) -> Result<Plugin> {
    let link = Collection::operation({
        let host = host.clone();
        async move |transport, rows: CollectionOutput| {
            let ctx = Context::from_collection(transport, host.clone())?;
            collect_link(&ctx, &rows).await
        }
    })
    .stream(streams::probe(BLUETOOTH_LINK_PROBE))
    .stream(Stream::<BluetoothConnections>::new(BluetoothConnections::STREAM).description(BluetoothConnections::LABEL))
    .stream(Stream::<BluetoothThroughput>::new(BluetoothThroughput::STREAM).description(BluetoothThroughput::LABEL))
    .stream(Stream::<BluetoothRssi>::new(BluetoothRssi::STREAM).description(BluetoothRssi::LABEL))
    .interval_setting(settings::BLUETOOTH_LINK_INTERVAL)
    .on_stop({
        let host = host.clone();
        async move |_| {
            host.bluetooth.stop_link();
            Ok(())
        }
    });
    let inventory = Collection::operation({
        let host = host.clone();
        async move |transport, rows: CollectionOutput| {
            let ctx = Context::from_collection(transport, host.clone())?;
            collect_inventory(&ctx, &rows).await
        }
    })
    .stream(streams::probe(BLUETOOTH_PROBE))
    .stream(Stream::<Bluetooth>::new(Bluetooth::STREAM).description(Bluetooth::LABEL))
    .stream(Stream::<BluetoothRadios>::new(BluetoothRadios::STREAM).description(BluetoothRadios::LABEL))
    .stream(Stream::<BluetoothDevice>::new(BluetoothDevice::STREAM).description(BluetoothDevice::LABEL))
    .interval_setting(settings::BLUETOOTH_INTERVAL)
    .on_stop({
        let host = host.clone();
        async move |_| {
            host.bluetooth.stop_inventory();
            Ok(())
        }
    });
    Ok(plugin.collection(link).collection(inventory))
}

/// Publish the Link Groups Whose Fields Were All Read and One RSSI Row per Device Heard; the Outcome Row Always.
pub fn publish_link(reading: readings::BluetoothLink, rows: &CollectionOutput) {
    streams::outcome(rows, BLUETOOTH_LINK_PROBE, reading.probe_status, reading.probe_error);
    if let Some(bluetooth_connected_device_count) = reading.bluetooth_connected_device_count {
        rows.publish(&Stream::new(BluetoothConnections::STREAM), BluetoothConnections { bluetooth_connected_device_count });
    }
    if let (Some(bluetooth_rx_mbps), Some(bluetooth_tx_mbps)) = (reading.bluetooth_rx_mbps, reading.bluetooth_tx_mbps) {
        rows.publish(&Stream::new(BluetoothThroughput::STREAM), BluetoothThroughput { bluetooth_rx_mbps, bluetooth_tx_mbps });
    }
    for rssi in reading.bluetooth_rssi {
        rows.publish(&Stream::new(BluetoothRssi::STREAM), rssi);
    }
}

/// Publish the Inventory Groups Whose Fields Were All Read and One Row per Present Device; the Outcome Row Always.
pub fn publish_inventory(reading: readings::Bluetooth, rows: &CollectionOutput) {
    streams::outcome(rows, BLUETOOTH_PROBE, reading.probe_status, reading.probe_error);
    if let (Some(bluetooth_present_device_count), Some(bluetooth_radio_state)) =
        (reading.bluetooth_present_device_count, reading.bluetooth_radio_state)
    {
        rows.publish(
            &Stream::new(Bluetooth::STREAM),
            Bluetooth {
                bluetooth_present_device_count,
                bluetooth_radio_state,
            },
        );
    }
    if let Some(bluetooth_adapter_count) = reading.bluetooth_adapter_count {
        rows.publish(&Stream::new(BluetoothRadios::STREAM), BluetoothRadios { bluetooth_adapter_count });
    }
    for bluetooth_device_name in reading.bluetooth_device_names {
        rows.publish(&Stream::new(BluetoothDevice::STREAM), BluetoothDevice { bluetooth_device_name });
    }
}

/* === Private Functions === */

async fn collect_link(ctx: &Context, rows: &CollectionOutput) -> Result<()> {
    let options = LinkOptions {
        adapter: settings::selected(ctx, settings::BLUETOOTH_ADAPTER).await?,
        rssi_devices: settings::selections(ctx, settings::BLUETOOTH_RSSI_DEVICES).await?,
        probe_timeout: ctx.duration_setting(settings::BLUETOOTH_PROBE_TIMEOUT).await?,
        minimum_rate_window: ctx.duration_setting(settings::MINIMUM_RATE_WINDOW).await?,
    };
    let reading = ctx.plugin::<Arc<Host>>()?.bluetooth.link(options).await?;
    publish_link(reading, rows);
    Ok(())
}

async fn collect_inventory(ctx: &Context, rows: &CollectionOutput) -> Result<()> {
    let options = InventoryOptions {
        adapter: settings::selected(ctx, settings::BLUETOOTH_ADAPTER).await?,
        probe_timeout: ctx.duration_setting(settings::BLUETOOTH_PROBE_TIMEOUT).await?,
        deep_refresh_interval: ctx.duration_setting(settings::BLUETOOTH_DEEP_REFRESH_INTERVAL).await?,
        timing: Timing::read(ctx).await?,
    };
    let reading = ctx.plugin::<Arc<Host>>()?.bluetooth.inventory(options).await?;
    publish_inventory(reading, rows);
    Ok(())
}

/* === Tests === */
