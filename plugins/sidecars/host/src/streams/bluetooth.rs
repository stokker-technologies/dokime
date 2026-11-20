/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Streams and Their Observers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::Host;
use crate::collectors::bluetooth::Options;
use crate::domain::telemetry::Bluetooth;
use crate::settings;
use dokime::plugin::{Observer, Plugin, Stream};
use dokime::{Result, operations};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Bluetooth Streams and Observers.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let stream = Stream::<Bluetooth>::new(Bluetooth::STREAM, Bluetooth::LABEL);
    let observer = Observer::operation(&stream, sample).interval_setting(settings::BLUETOOTH_INTERVAL);
    Ok(plugin.stream(stream).observer(observer))
}

/* === Private Functions === */

async fn sample(ctx: &operations::Context) -> Result<Bluetooth> {
    let options = Options {
        adapter: settings::selected(ctx, settings::BLUETOOTH_ADAPTER).await?,
        probe_timeout: settings::duration_value(ctx, settings::BLUETOOTH_PROBE_TIMEOUT).await?,
        deep_refresh_interval: settings::duration_value(ctx, settings::BLUETOOTH_DEEP_REFRESH_INTERVAL).await?,
    };
    ctx.plugin::<Host>()?.bluetooth.collect(options).await
}

/* === Tests === */
