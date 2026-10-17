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
use crate::domain::operations::GetStatus;
use dokime_runtime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct StatusProbe {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::operation]
impl StatusProbe {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, _: &mut GetStatus) -> Result<StatusResponse, Refusal> {
        let stopped = dokime_runtime::plugin::refusal(self.resources.app().stopped().await)?;
        Ok(StatusResponse::new(if stopped { "disconnected" } else { "connected" }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
