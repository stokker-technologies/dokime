/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


use crate::{Bench, Error};
use dokime::host::ToolConnectionAcquired;
use dokime::plugin::ListenerContext;


pub struct UnitConnected;


#[dokime::listener(id = "{{crate}}.unit_connected", record = ToolConnectionAcquired, resources = [Bench])]
impl UnitConnected {
    async fn hear(ctx: &ListenerContext<'_, Self>, acquired: ToolConnectionAcquired) -> Result<(), Error> {
        ctx.resource::<Bench>().observed(acquired.resource_id);
        Ok(())
    }
}
