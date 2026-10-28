/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::{Resources, error::Error};
use dokime::host::ToolConnectionAcquired;
use dokime::plugin::ListenerContext;

pub(crate) struct Observed;

#[dokime::listener(id = "obc.bench.connection", record = ToolConnectionAcquired, resources = [Resources])]
impl Observed {
    async fn hear(ctx: &ListenerContext<'_, Self>, acquired: ToolConnectionAcquired) -> Result<(), Error> {
        ctx.resource::<Resources>().observed(acquired.resource_id);
        Ok(())
    }
}
