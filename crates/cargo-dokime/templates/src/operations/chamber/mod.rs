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
use crate::domain::operations::{DetectPort, ReadChamber};
use dokime_runtime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct ChamberReader {
    bench: Arc<Bench>,
}

pub struct PortDetector {
    bench: Arc<Bench>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::operation]
impl ChamberReader {
    pub async fn initialize(bench: Arc<Bench>, _: &Supervisor) -> Self {
        Self { bench }
    }

    async fn run(&mut self, ctx: &Supervisor, _: &mut ReadChamber) -> Result<ChamberReading, Refusal> {
        if let Some(scope) = ExecutionScope::serving::<Self>(ctx) {
            tracing::debug!(key = %scope.key(), "reading the chamber for a run");
        }
        Ok(ChamberReading {
            celsius: dokime_runtime::plugin::refusal(self.bench.sensor.read_celsius())?,
            sensor: self.bench.sensor.describe(),
        })
    }
}

#[dokime_runtime::operation]
impl PortDetector {
    pub async fn initialize(bench: Arc<Bench>, _: &Supervisor) -> Self {
        Self { bench }
    }

    async fn run(&mut self, _: &Supervisor, _: &mut DetectPort) -> Result<Discovered, Refusal> {
        Ok(Discovered::of(self.bench.sensor.port().to_owned()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
