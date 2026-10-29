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

// Third Party Imports

// Local Imports
use crate::Resources;
use crate::domain::operations::GetStatus;
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::{Refusal, StatusResponse};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for GetStatus {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<StatusResponse>, Refusal> {
        let stopped = dokime::plugin::refusal(ctx.resource::<Resources>().app().stopped().await)?;
        Ok(Response::new(StatusResponse::new(if stopped { "disconnected" } else { "connected" })))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
