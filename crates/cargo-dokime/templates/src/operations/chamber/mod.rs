/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


use crate::Bench;
use crate::domain::model::chamber::ChamberReading;
use crate::domain::operations::{DetectPort, ReadChamber};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::*;


impl Operation for ReadChamber {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<ChamberReading>, Refusal> {
        if let Some(scope) = ExecutionScope::serving::<Self>(ctx.call()) {
            tracing::debug!(key = %scope.key(), "reading the chamber for a run");
        }
        let bench = ctx.resource::<Bench>();
        Ok(Response::new(ChamberReading {
            celsius: dokime::plugin::refusal(bench.sensor.read_celsius())?,
            sensor: bench.sensor.describe(),
        }))
    }
}

impl Operation for DetectPort {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Discovered>, Refusal> {
        Ok(Response::new(Discovered::of(ctx.resource::<Bench>().sensor.port().to_owned())))
    }
}
