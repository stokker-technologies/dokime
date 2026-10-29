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
use crate::domain::model::readiness::Readiness;
use crate::domain::operations::AskReadiness;
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for AskReadiness {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Readiness>, Refusal> {
        let stopped = dokime::plugin::refusal(ctx.resource::<Resources>().app().stopped().await)?;
        Ok(Response::new(Readiness { stopped }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
