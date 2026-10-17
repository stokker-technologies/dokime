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
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::Resources;
use dokime_runtime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct AskReadiness {
    resources: Arc<Resources>,
}

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
pub struct Readiness {
    #[row(description = "Whether the embedded runtime has stopped taking new work.")]
    pub stopped: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::operation(
    id = "obc-plugin.check_readiness",
    version = 1,
    description = "Reports whether the testbench will still take work. A template calls it with `ctx.call::<AskReadiness>(())`.",
    policy = super::POLICY
)]
impl AskReadiness {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<Readiness, Refusal> {
        Ok(Readiness {
            stopped: dokime_runtime::plugin::refusal(self.resources.app().stopped().await)?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
