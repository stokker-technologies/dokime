/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Network Collection: Traffic per Tick and One Row per Native Change Since the Previous Tick

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Host;
use crate::domain::telemetry::{NETWORK_PROBE, NetworkAddressChange, NetworkChange, NetworkInternet, NetworkPrimary};
use crate::domain::{readings, telemetry};
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

/// Declare the Network Collection and Every Stream It Feeds; Native Change Watching Lives From Its Start to Its Stop.
pub fn catalog(plugin: Plugin, host: Arc<Host>) -> Result<Plugin> {
    let collection = Collection::operation({
        let host = host.clone();
        async move |transport, rows: CollectionOutput| {
            let ctx = Context::from_collection(transport, host.clone())?;
            collect(&ctx, &rows).await.map_err(Into::into)
        }
    })
    .stream(streams::probe(NETWORK_PROBE))
    .stream(Stream::<telemetry::Network>::new(telemetry::Network::STREAM).description(telemetry::Network::LABEL))
    .stream(Stream::<NetworkPrimary>::new(NetworkPrimary::STREAM).description(NetworkPrimary::LABEL))
    .stream(Stream::<NetworkInternet>::new(NetworkInternet::STREAM).description(NetworkInternet::LABEL))
    .stream(Stream::<NetworkChange>::new(NetworkChange::STREAM).description(NetworkChange::LABEL))
    .stream(Stream::<NetworkAddressChange>::new(NetworkAddressChange::STREAM).description(NetworkAddressChange::LABEL))
    .interval_setting(settings::NETWORK_INTERVAL)
    .on_start({
        let host = host.clone();
        async move |_| host.network.start().map_err(Into::into)
    })
    .on_stop({
        let host = host.clone();
        async move |_| {
            host.network.stop();
            Ok(())
        }
    });
    Ok(plugin.collection(collection))
}

/// Publish Each Group of a Reading Whose Fields Were All Read, One Row per Change; the Outcome Row Always.
pub fn publish(reading: readings::Network, rows: &CollectionOutput) {
    streams::outcome(rows, NETWORK_PROBE, reading.probe_status, reading.probe_error);
    let traffic = (
        reading.network_interface_count,
        reading.network_recv_mb_per_second,
        reading.network_sent_mb_per_second,
        reading.network_total_recv_bytes,
        reading.network_total_sent_bytes,
        reading.network_rx_errors,
        reading.network_tx_errors,
    );
    if let (
        Some(network_interface_count),
        Some(network_recv_mb_per_second),
        Some(network_sent_mb_per_second),
        Some(network_total_recv_bytes),
        Some(network_total_sent_bytes),
        Some(network_rx_errors),
        Some(network_tx_errors),
    ) = traffic
    {
        let row = telemetry::Network {
            network_interface_count,
            network_recv_mb_per_second,
            network_sent_mb_per_second,
            network_total_recv_bytes,
            network_total_sent_bytes,
            network_rx_errors,
            network_tx_errors,
        };
        rows.publish(&Stream::new(telemetry::Network::STREAM), row);
    }
    if let Some(primary_interface) = reading.primary_interface {
        rows.publish(&Stream::new(NetworkPrimary::STREAM), NetworkPrimary { primary_interface });
    }
    if let Some(internet) = reading.internet {
        rows.publish(&Stream::new(NetworkInternet::STREAM), NetworkInternet { internet });
    }
    for change in reading.changes {
        match change.address {
            Some(address) => {
                let row = NetworkAddressChange {
                    change: change.kind,
                    adapter_id: change.adapter_id,
                    address,
                };
                rows.publish(&Stream::new(NetworkAddressChange::STREAM), row);
            }
            None => rows.publish(
                &Stream::new(NetworkChange::STREAM),
                NetworkChange {
                    change: change.kind,
                    adapter_id: change.adapter_id,
                },
            ),
        }
    }
}

/* === Private Functions === */

async fn collect(ctx: &Context, rows: &CollectionOutput) -> Result<()> {
    let reading = ctx.plugin::<Arc<Host>>()?.network.collect(ctx).await?;
    publish(reading, rows);
    Ok(())
}

/* === Tests === */
