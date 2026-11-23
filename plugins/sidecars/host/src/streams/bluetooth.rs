/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Bluetooth Collection: the Inventory per Tick, One Row per Present Device

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Host;
use crate::collectors::bluetooth::Options;
use crate::controls::Timing;
use crate::domain::readings;
use crate::domain::telemetry::{BLUETOOTH_PROBE, Bluetooth, BluetoothConnections, BluetoothDevice, BluetoothRadios, BluetoothTraffic};
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

/// Declare the Bluetooth Collection and Every Stream It Feeds; the Inventory Cache Lives From Its Start to Its Stop.
pub fn catalog(plugin: Plugin, host: Arc<Host>) -> Result<Plugin> {
    let collection = Collection::operation({
        let host = host.clone();
        async move |transport, rows: CollectionOutput| {
            let ctx = Context::from_collection(transport, host.clone())?;
            collect(&ctx, &rows).await
        }
    })
    .stream(streams::probe(BLUETOOTH_PROBE))
    .stream(Stream::<Bluetooth>::new(Bluetooth::STREAM).description(Bluetooth::LABEL))
    .stream(Stream::<BluetoothRadios>::new(BluetoothRadios::STREAM).description(BluetoothRadios::LABEL))
    .stream(Stream::<BluetoothConnections>::new(BluetoothConnections::STREAM).description(BluetoothConnections::LABEL))
    .stream(Stream::<BluetoothTraffic>::new(BluetoothTraffic::STREAM).description(BluetoothTraffic::LABEL))
    .stream(Stream::<BluetoothDevice>::new(BluetoothDevice::STREAM).description(BluetoothDevice::LABEL))
    .interval_setting(settings::BLUETOOTH_INTERVAL)
    .on_stop({
        let host = host.clone();
        async move |_| {
            host.bluetooth.stop();
            Ok(())
        }
    });
    Ok(plugin.collection(collection))
}

/// Publish the Inventory Groups Whose Fields Were All Read and One Row per Present Device; the Outcome Row Always.
pub fn publish(reading: readings::Bluetooth, rows: &CollectionOutput) {
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
    if let Some(bluetooth_connected_device_count) = reading.bluetooth_connected_device_count {
        rows.publish(&Stream::new(BluetoothConnections::STREAM), BluetoothConnections { bluetooth_connected_device_count });
    }
    if let (Some(bluetooth_recv_mb_per_second), Some(bluetooth_sent_mb_per_second)) =
        (reading.bluetooth_recv_mb_per_second, reading.bluetooth_sent_mb_per_second)
    {
        rows.publish(
            &Stream::new(BluetoothTraffic::STREAM),
            BluetoothTraffic {
                bluetooth_recv_mb_per_second,
                bluetooth_sent_mb_per_second,
            },
        );
    }
    for bluetooth_device_name in reading.bluetooth_device_names {
        rows.publish(&Stream::new(BluetoothDevice::STREAM), BluetoothDevice { bluetooth_device_name });
    }
}

/* === Private Functions === */

async fn collect(ctx: &Context, rows: &CollectionOutput) -> Result<()> {
    let options = Options {
        adapter: settings::selected(ctx, settings::BLUETOOTH_ADAPTER).await?,
        probe_timeout: ctx.duration_setting(settings::BLUETOOTH_PROBE_TIMEOUT).await?,
        deep_refresh_interval: ctx.duration_setting(settings::BLUETOOTH_DEEP_REFRESH_INTERVAL).await?,
        timing: Timing::read(ctx).await?,
    };
    let reading = ctx.plugin::<Arc<Host>>()?.bluetooth.collect(options).await?;
    publish(reading, rows);
    Ok(())
}

/* === Tests === */
