/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::Resources;
use crate::domain::model::connections::Connections;
use crate::domain::operations::ListConnections;
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;

impl Operation for ListConnections {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Connections>, Refusal> {
        Ok(Response::new(Connections {
            resources: ctx.resource::<Resources>().connected(),
        }))
    }
}
