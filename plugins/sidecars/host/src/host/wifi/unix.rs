/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

use super::{Samples, State, Wifi};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;
use dokime::telemetry::TelemetryRow;

/* === Implementations === */

impl Operation for Wifi {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Samples>, Refusal> {
        let state = ctx.resource::<tokio::sync::Mutex<State>>().clone();
        let task = ctx.resource::<crate::Workers>().0.spawn_blocking(move || {
            let mut state = state.blocking_lock();
            let options = state.options.clone();
            let samples = state.collector.collect_link_with_options(&options)?;
            vec![samples]
                .into_iter()
                .map(|sample| dokime::host::Wifi::from_row(&sample.values).map_err(|error| crate::Error::Probe(error.to_string())))
                .collect::<crate::Result<Vec<_>>>()
        });
        let rows = dokime::plugin::refusal(dokime::plugin::refusal(task.await)?)?;
        Ok(Response::new(Samples { rows }))
    }
}
