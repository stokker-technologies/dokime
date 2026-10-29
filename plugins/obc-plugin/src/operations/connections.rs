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
use crate::domain::model::connections::Connections;
use crate::domain::operations::ListConnections;
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ListConnections {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Connections>, Refusal> {
        Ok(Response::new(Connections {
            resources: ctx.resource::<Resources>().connected(),
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
