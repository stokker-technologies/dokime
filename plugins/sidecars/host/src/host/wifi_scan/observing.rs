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
use super::WifiScan;
use crate::{Error, HostTelemetryStream};
use dokime::harmos::operations::Host;
use dokime::harmos::plugin;
use dokime::harmos::sidecar::inventory::observers::Declared;
use dokime::harmos::sidecar::{Activation, Observer, ObserverContext, Publisher};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct State {
    pub(crate) collector: crate::collectors::wifi::WifiScanCollector,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<P: plugin::Declared> Observer<Publisher<P>> for WifiScan {
    type Error = Error;
    const ID: &'static str = <Self as Declared>::ENTRY.id;
    const STREAMS: &'static [dokime::harmos::sidecar::inventory::publications::Entry] = <Self as Declared>::ENTRY.streams;
    const ACTIVATION: Activation = <Self as Declared>::ENTRY.activation;
    const INTERVAL: Duration = <Self as Declared>::ENTRY.interval;

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

impl Declared for WifiScan {
    const ENTRY: dokime::harmos::sidecar::inventory::observers::Entry = dokime::harmos::sidecar::inventory::observers::Entry {
        id: "host.wifi.scan",
        description: "",
        interval: Duration::from_millis(HostTelemetryStream::WifiScan.interval().0),
        activation: Activation::Demand,
        streams: &[dokime::harmos::sidecar::inventory::publications::Entry::of::<Self>()],
        demand_keys: &[],
    };
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
