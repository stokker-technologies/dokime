/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

use super::System;
use crate::{Error, HostPlugin, HostTelemetryStream};
use dokime::harmos::operations::Host;
use dokime::harmos::sidecar::{Activation, Observer, ObserverContext, Publisher};
use std::time::Duration;

/* === Structs === */

pub struct State {
    pub(crate) collector: crate::collectors::system::SystemCollector,
}

/* === Implementations === */

impl Observer<Publisher<HostPlugin>> for System {
    type Error = Error;
    const ID: &'static str = "host.system";
    const STREAMS: &'static [dokime::harmos::sidecar::inventory::publications::Entry] =
        &[dokime::harmos::sidecar::inventory::publications::Entry::of::<Self>()];
    const ACTIVATION: Activation = Activation::Demand;
    const INTERVAL: Duration = Duration::from_millis(HostTelemetryStream::System.interval().0);

    async fn initialize(_: &dokime::harmos::ExecutionContext) -> Result<Self, Error> {
        Ok(Self {})
    }

    async fn observe(&mut self, ctx: &ObserverContext<'_, Publisher<HostPlugin>>) -> Result<(), Error> {
        let response = ctx.host().supervisor().call::<Self>(Self {}).await.map_err(|error| Error::Probe(error.to_string()))?;
        let mut batch = ctx.host().batch::<Self>();
        for sample in &response.rows {
            batch.push_at(sample.row.clone(), sample.timestamp_micros);
        }
        ctx.host().publish_batch_on::<Self>(&batch).await.map_err(|error| Error::Probe(error.to_string()))?;
        Ok(())
    }
}

impl dokime::harmos::sidecar::inventory::observers::Declared for System {
    const ENTRY: dokime::harmos::sidecar::inventory::observers::Entry = dokime::harmos::sidecar::inventory::observers::Entry {
        id: <Self as Observer<Publisher<HostPlugin>>>::ID,
        description: <Self as Observer<Publisher<HostPlugin>>>::DESCRIPTION,
        interval: <Self as Observer<Publisher<HostPlugin>>>::INTERVAL,
        activation: <Self as Observer<Publisher<HostPlugin>>>::ACTIVATION,
        streams: <Self as Observer<Publisher<HostPlugin>>>::STREAMS,
        demand_keys: <Self as Observer<Publisher<HostPlugin>>>::DEMAND_KEYS,
    };
}
