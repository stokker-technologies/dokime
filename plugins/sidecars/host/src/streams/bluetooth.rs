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
use dokime::plugin::{Collection, Plugin, Rows, Stream};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Bluetooth Collection and Every Stream It Feeds; the Inventory Cache Lives From Its Start to Its Stop.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let collection = Collection::operation(collect)
        .stream(streams::probe(BLUETOOTH_PROBE))
        .stream(Stream::<Bluetooth>::new(Bluetooth::STREAM, Bluetooth::LABEL))
        .stream(Stream::<BluetoothRadios>::new(BluetoothRadios::STREAM, BluetoothRadios::LABEL))
        .stream(Stream::<BluetoothConnections>::new(BluetoothConnections::STREAM, BluetoothConnections::LABEL))
        .stream(Stream::<BluetoothTraffic>::new(BluetoothTraffic::STREAM, BluetoothTraffic::LABEL))
        .stream(Stream::<BluetoothDevice>::new(BluetoothDevice::STREAM, BluetoothDevice::LABEL))
        .interval_setting(settings::BLUETOOTH_INTERVAL)
        .on_stop(|ctx: &Context| {
            ctx.plugin::<Host>()?.bluetooth.stop();
            Ok(())
        });
    Ok(plugin.collection(collection))
}

/// Publish the Inventory Groups Whose Fields Were All Read and One Row per Present Device; the Outcome Row Always.
pub fn publish(reading: readings::Bluetooth, rows: &mut Rows) -> Result<()> {
    streams::outcome(rows, BLUETOOTH_PROBE, reading.probe_status, reading.probe_error)?;
    if let (Some(bluetooth_present_device_count), Some(bluetooth_radio_state)) =
        (reading.bluetooth_present_device_count, reading.bluetooth_radio_state)
    {
        rows.publish(
            Bluetooth::STREAM,
            Bluetooth {
                bluetooth_present_device_count,
                bluetooth_radio_state,
            },
        )?;
    }
    if let Some(bluetooth_adapter_count) = reading.bluetooth_adapter_count {
        rows.publish(BluetoothRadios::STREAM, BluetoothRadios { bluetooth_adapter_count })?;
    }
    if let Some(bluetooth_connected_device_count) = reading.bluetooth_connected_device_count {
        rows.publish(BluetoothConnections::STREAM, BluetoothConnections { bluetooth_connected_device_count })?;
    }
    if let (Some(bluetooth_recv_mb_per_second), Some(bluetooth_sent_mb_per_second)) =
        (reading.bluetooth_recv_mb_per_second, reading.bluetooth_sent_mb_per_second)
    {
        rows.publish(
            BluetoothTraffic::STREAM,
            BluetoothTraffic {
                bluetooth_recv_mb_per_second,
                bluetooth_sent_mb_per_second,
            },
        )?;
    }
    for bluetooth_device_name in reading.bluetooth_device_names {
        rows.publish(BluetoothDevice::STREAM, BluetoothDevice { bluetooth_device_name })?;
    }
    Ok(())
}

/* === Private Functions === */

async fn collect(ctx: &Context, rows: &mut Rows) -> Result<()> {
    let options = Options {
        adapter: settings::selected(ctx, settings::BLUETOOTH_ADAPTER).await?,
        probe_timeout: settings::duration_value(ctx, settings::BLUETOOTH_PROBE_TIMEOUT).await?,
        deep_refresh_interval: settings::duration_value(ctx, settings::BLUETOOTH_DEEP_REFRESH_INTERVAL).await?,
        timing: Timing::read(ctx).await?,
    };
    let reading = ctx.plugin::<Host>()?.bluetooth.collect(options).await?;
    publish(reading, rows)
}

/* === Tests === */
