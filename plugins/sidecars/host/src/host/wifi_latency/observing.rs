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
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::WifiLatency;
use crate::Error;
use dokime::harmos::operations::Host;
use dokime::harmos::plugin;
use dokime::harmos::sidecar::inventory::observers::Declared;
use dokime::harmos::sidecar::{Activation, Observer, ObserverContext, Publisher};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct State {
    pub(crate) collector: crate::collectors::wifi::WifiLatencyCollector,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<P: plugin::Declared> Observer<Publisher<P>> for WifiLatency {
    type Error = Error;
    const INTERVAL: Duration = Duration::from_millis(crate::settings::WIFI_LATENCY_INTERVAL_MS);
    const ID: &'static str = <Self as Declared>::ENTRY.id;
    const STREAMS: &'static [dokime::harmos::sidecar::inventory::publications::Entry] = <Self as Declared>::ENTRY.streams;
    const ACTIVATION: Activation = <Self as Declared>::ENTRY.activation;
    fn interval(resources: &dokime::harmos::ExecutionContext) -> Result<Duration, Error> {
        let options = resources.resource::<crate::HostOptions>().map_err(|error| Error::Probe(error.to_string()))?;
        Ok(Duration::from_millis(options.cadence.wifi_latency_interval_ms))
    }

    async fn initialize(_: &dokime::harmos::ExecutionContext) -> Result<Self, Error> {
        Ok(Self {})
    }

    async fn observe(&mut self, ctx: &ObserverContext<'_, Publisher<P>>) -> Result<(), Error> {
        let response = ctx.host().supervisor().call::<Self>(Self {}).await.map_err(|error| Error::Probe(error.to_string()))?;
        let mut batch = ctx.host().batch::<Self>();
        for sample in &response.rows {
            batch.push_at(sample.row.clone(), sample.timestamp_micros);
        }
        ctx.host().publish_batch_on::<Self>(&batch).await.map_err(|error| Error::Probe(error.to_string()))?;
        Ok(())
    }
}

impl Declared for WifiLatency {
    const ENTRY: dokime::harmos::sidecar::inventory::observers::Entry = dokime::harmos::sidecar::inventory::observers::Entry {
        id: "host.wifi.latency",
        description: "",
        activation: Activation::Demand,
        streams: &[dokime::harmos::sidecar::inventory::publications::Entry::of::<Self>()],
        demand_keys: &[],
    };
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
