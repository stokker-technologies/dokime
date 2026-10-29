/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::{Collected, Samples, State, WifiLink};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;
use dokime::telemetry::TelemetryRow;

impl Operation for WifiLink {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Samples>, Refusal> {
        let state = ctx.resource::<tokio::sync::Mutex<State>>().clone();
        let task = ctx.resource::<crate::Workers>().0.spawn_blocking(move || {
            let mut state = state.blocking_lock();
            let samples = state.collector.collect()?;
            vec![samples]
                .into_iter()
                .map(|sample| {
                    let timestamp = sample.timestamp.to_datetime().map_err(|error| crate::Error::Probe(error.to_string()))?;
                    let timestamp_micros =
                        i64::try_from(timestamp.unix_timestamp_nanos() / 1_000).map_err(|error| crate::Error::Probe(error.to_string()))?;
                    let row = crate::WifiLink::from_row(&sample.values).map_err(|error| crate::Error::Probe(error.to_string()))?;
                    Ok(Collected { row, timestamp_micros })
                })
                .collect::<crate::Result<Vec<_>>>()
        });
        let rows = dokime::plugin::refusal(dokime::plugin::refusal(task.await)?)?;
        Ok(Response::new(Samples { rows }))
    }
}
