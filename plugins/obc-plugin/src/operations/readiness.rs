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
use crate::domain::model::readiness::Readiness;
use crate::domain::operations::AskReadiness;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ReadinessProbe {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[dokime::operation]
impl ReadinessProbe {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, _: &mut AskReadiness) -> Result<Readiness, Refusal> {
        Ok(Readiness {
            stopped: dokime::plugin::refusal(self.resources.app().stopped().await)?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
