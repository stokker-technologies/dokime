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
use crate::{Resources, error::Error};
use dokime::host::ToolConnectionAcquired;
use dokime::plugin::ListenerContext;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Observed;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[dokime::listener(id = "obc.bench.connection", record = ToolConnectionAcquired, resources = [Resources])]
impl Observed {
    async fn hear(ctx: &ListenerContext<'_, Self>, acquired: ToolConnectionAcquired) -> Result<(), Error> {
        ctx.resource::<Resources>().observed(acquired.resource_id);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
