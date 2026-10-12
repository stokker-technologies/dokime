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
pub struct GetStatus {
    bench: Arc<Bench>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "{{owner}}.get_status",
    version = 1,
    description = "Reports whether the bench is reachable.",
    policy = super::POLICY
)]
impl GetStatus {
    pub fn new(bench: Arc<Bench>) -> Self {
        Self { bench }
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<StatusResponse, Refusal> {
        Ok(StatusResponse::ready()
            .field("sensor", self.bench.sensor.describe())
            .field("units_connected", self.bench.connected().len() as i64)
            .field("upload_configured", self.bench.upload_configured))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
