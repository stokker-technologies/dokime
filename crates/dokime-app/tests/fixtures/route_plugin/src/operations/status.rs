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
use std::sync::atomic::Ordering;
// Third Party Imports

// Local Imports
use crate::Resources;
use dokime::domains::StatusResponse;
use dokime::{Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct GetStatus {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "route-fixture.get_status",
    version = 1,
    description = "Reports whether the fixture holds a token, and how many commands it ran.",
    policy = crate::POLICY
)]
impl GetStatus {
    pub(crate) fn new(resources: Arc<Resources>) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<StatusResponse, Refusal> {
        Ok(StatusResponse {
            state: if self.resources.connected {
                "connected"
            } else {
                "disconnected"
            }
            .into(),
            fields: std::collections::BTreeMap::from([(
                "Calls".into(),
                format!("calls-{}", self.resources.calls.load(Ordering::Relaxed)).into(),
            )]),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
