/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::Resources;
use crate::domain::model::readiness::Readiness;
use crate::domain::operations::AskReadiness;
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;

impl Operation for AskReadiness {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Readiness>, Refusal> {
        let stopped = dokime::plugin::refusal(ctx.resource::<Resources>().app().stopped().await)?;
        Ok(Response::new(Readiness { stopped }))
    }
}
