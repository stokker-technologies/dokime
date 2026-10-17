/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Bench;
use crate::domain::model::chamber::ChamberReading;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct ReadChamber {
    bench: Arc<Bench>,
}

pub struct DetectPort {
    bench: Arc<Bench>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "{{owner}}.get_chamber",
    version = 1,
    description = "Reads the chamber. A template calls it with `ctx.call::<ReadChamber>(())`.",
    policy = crate::domain::operations::POLICY
)]
impl ReadChamber {
    pub async fn initialize(bench: Arc<Bench>, _: &Supervisor) -> Self {
        Self { bench }
    }

    async fn run(&mut self, ctx: &Supervisor, _: &()) -> Result<ChamberReading, Refusal> {
        if let Some(scope) = ExecutionScope::serving::<Self>(ctx) {
            tracing::debug!(key = %scope.key(), "reading the chamber for a run");
        }
        Ok(ChamberReading {
            celsius: dokime::plugin::refusal(self.bench.sensor.read_celsius())?,
            sensor: self.bench.sensor.describe(),
        })
    }
}

#[dokime::operation(
    id = "{{owner}}.detect_port",
    version = 1,
    description = "Finds the sensor port the settings card offers before anybody saved one.",
    policy = crate::domain::operations::POLICY
)]
impl DetectPort {
    pub async fn initialize(bench: Arc<Bench>, _: &Supervisor) -> Self {
        Self { bench }
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<Discovered, Refusal> {
        Ok(Discovered::of(self.bench.sensor.port().to_owned()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
