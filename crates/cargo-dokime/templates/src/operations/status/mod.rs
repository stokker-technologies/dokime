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
use crate::domain::operations::GetStatus;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct StatusProbe {
    bench: Arc<Bench>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

// The handler declares nothing: its request type is the operation.
#[dokime::operation]
impl StatusProbe {
    pub async fn initialize(bench: Arc<Bench>, _: &Supervisor) -> Self {
        Self { bench }
    }

    async fn run(&mut self, _: &Supervisor, _: &mut GetStatus) -> Result<StatusResponse, Refusal> {
        Ok(StatusResponse::ready()
            .field("sensor", self.bench.sensor.describe())
            .field("units_connected", self.bench.connected().len() as i64)
            .field("upload_configured", self.bench.upload_configured))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
