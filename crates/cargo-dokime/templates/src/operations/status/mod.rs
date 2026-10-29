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

// Third Party Imports

// Local Imports
use crate::Bench;
use crate::domain::operations::GetStatus;
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

// The request is the operation: this body runs one attempt with the resources
// the definition declares, and holds no state of its own.
impl Operation for GetStatus {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<StatusResponse>, Refusal> {
        let bench = ctx.resource::<Bench>();
        Ok(Response::new(
            StatusResponse::ready()
                .field("sensor", bench.sensor.describe())
                .field("units_connected", bench.connected().len() as i64)
                .field("upload_configured", bench.upload_configured),
        ))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
