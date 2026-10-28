/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::Resources;
use crate::domain::operations::GetStatus;
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::{Refusal, StatusResponse};

impl Operation for GetStatus {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<StatusResponse>, Refusal> {
        let stopped = dokime::plugin::refusal(ctx.resource::<Resources>().app().stopped().await)?;
        Ok(Response::new(StatusResponse::new(if stopped { "disconnected" } else { "connected" })))
    }
}
