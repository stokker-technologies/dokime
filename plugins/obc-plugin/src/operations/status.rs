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
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct GetStatus {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "obc-plugin.get_status",
    version = 1,
    description = "Reports bench readiness without advancing the Charger. The settings section selects this operation as its readiness probe.",
    policy = super::POLICY
)]
impl GetStatus {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<StatusResponse, Refusal> {
        let stopped = dokime::plugin::refusal(self.resources.app().stopped().await)?;
        Ok(StatusResponse::new(if stopped {
            "stopped"
        } else {
            "ready"
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
