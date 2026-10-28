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
use crate::{Bench, Error};
use dokime::host::ToolConnectionAcquired;
use dokime::plugin::ListenerContext;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct UnitConnected;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[dokime::listener(id = "{{crate}}.unit_connected", record = ToolConnectionAcquired, resources = [Bench])]
impl UnitConnected {
    async fn hear(ctx: &ListenerContext<'_, Self>, acquired: ToolConnectionAcquired) -> Result<(), Error> {
        ctx.resource::<Bench>().observed(acquired.resource_id);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
