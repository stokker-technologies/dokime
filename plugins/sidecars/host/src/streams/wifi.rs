/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Wi-Fi Streams and Their Observers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::Host;
use crate::collectors::wifi::{LatencyOptions, LinkOptions, ScanOptions};
use crate::domain::telemetry::{WifiLatency, WifiLink, WifiScan};
use crate::settings;
use dokime::plugin::{Observer, Plugin, Publisher, Stream};
use dokime::{Error, Result, SettingValue, operations, setting_value, templates};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Wi-Fi Streams and Observers.
///
/// A scan yields one row per observed network, so its stream publishes each collection itself at the scan interval instead of
/// answering one row per observer sample.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let link = Stream::<WifiLink>::new(WifiLink::STREAM, WifiLink::LABEL);
    let link_observer = Observer::operation(&link, link_sample).interval_setting(settings::WIFI_LINK_INTERVAL);
    let latency = Stream::<WifiLatency>::new(WifiLatency::STREAM, WifiLatency::LABEL);
    let latency_observer = Observer::operation(&latency, latency_sample).interval_setting(settings::WIFI_LATENCY_INTERVAL);
    let scan = Stream::<WifiScan>::new(WifiScan::STREAM, WifiScan::LABEL).publish_on(scan_publication);
    Ok(plugin.stream(link).observer(link_observer).stream(latency).observer(latency_observer).stream(scan))
}

/// Read a Saved Host Setting in a Publication, Refusing a Value Outside Its Declared Range.
pub async fn published_setting(ctx: &templates::Context, name: &str) -> Result<SettingValue> {
    let value = ctx.setting(name).await?;
    settings::setting(name)?.check(&value)?;
    Ok(value)
}

/// Read a Saved Host Duration Setting in a Publication.
pub async fn published_duration(ctx: &templates::Context, name: &str) -> Result<Duration> {
    published_setting(ctx, name).await?;
    ctx.duration_setting(name).await
}

/// Read a Saved Host Text Selection in a Publication; Empty Means Any Available Choice.
pub async fn published_selection(ctx: &templates::Context, name: &str) -> Result<Option<String>> {
    match published_setting(ctx, name).await?.value {
        Some(setting_value::Value::Text(value)) => Ok(Some(value.trim().to_owned()).filter(|value| !value.is_empty())),
        _ => Err(Error::Invalid(format!("Read Setting {name}: Configure a Text Value in the Plugin Settings"))),
    }
}

/* === Private Functions === */

async fn link_sample(ctx: &operations::Context) -> Result<WifiLink> {
    let options = LinkOptions {
        adapter: settings::selected(ctx, settings::WIFI_ADAPTER).await?,
        probe_timeout: settings::duration_value(ctx, settings::WIFI_LINK_PROBE_TIMEOUT).await?,
        minimum_rate_window: settings::duration_value(ctx, settings::MINIMUM_RATE_WINDOW).await?,
    };
    ctx.plugin::<Host>()?.wifi.link(options).await
}

async fn latency_sample(ctx: &operations::Context) -> Result<WifiLatency> {
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
    ctx.plugin::<Host>()?.wifi.latency(options).await
}

/// Publish One Scan per Interval Until Demand Closes; All Rows of One Scan Share Its Timestamp.
async fn scan_publication(ctx: &templates::Context, publisher: Publisher<WifiScan>) -> Result<()> {
    let wifi = &ctx.plugin::<Host>()?.wifi;
    loop {
        let congestion_weight = match published_setting(ctx, settings::CONGESTION_NETWORK_WEIGHT).await?.value {
            Some(setting_value::Value::Number(weight)) => weight,
            _ => return Err(Error::Invalid("Read Setting Congestion Weight: Configure a Number in the Plugin Settings".into())),
        };
        let options = ScanOptions {
            adapter: published_selection(ctx, settings::WIFI_ADAPTER).await?,
            probe_timeout: published_duration(ctx, settings::WIFI_SCAN_PROBE_TIMEOUT).await?,
            window: published_duration(ctx, settings::WIFI_SCAN_WINDOW).await?,
            minimum_interval: published_duration(ctx, settings::MINIMUM_SCAN_INTERVAL).await?,
            congestion_weight,
        };
        let rows = wifi.scan(options).await?;
        let at = ctx.now().await?;
        for row in rows {
            // Publication Fails Only When Demand Closed, Which Ends the Stream.
            if publisher.publish_at(row, Some(at)).await.is_err() {
                return Ok(());
            }
        }
        ctx.sleep(published_duration(ctx, settings::WIFI_SCAN_INTERVAL).await?).await?;
    }
}

/* === Tests === */
