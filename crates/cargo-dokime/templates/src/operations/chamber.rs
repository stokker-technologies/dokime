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
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Bench;
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

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct ChamberReading {
        #[row(description = "What the chamber is at, in °C.")]
    pub celsius: f64,
                        #[row(description = "What read it, for the traceability a template states.\nA template never names this plugin, so anything it wants to say about the bench it reads off a reply: what an operation answers is the whole of what a run knows about the process serving it.")]
    pub sensor: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "{{owner}}.get_chamber",
    version = 1,
    description = "Reads the chamber. A template calls it with `ctx.call::<ReadChamber>(())`.",
    policy = super::POLICY
)]
impl ReadChamber {
    pub fn new(bench: Arc<Bench>) -> Self {
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
    policy = super::POLICY
)]
impl DetectPort {
    pub fn new(bench: Arc<Bench>) -> Self {
        Self { bench }
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<Discovered, Refusal> {
        Ok(Discovered::of(self.bench.sensor.port().to_owned()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
