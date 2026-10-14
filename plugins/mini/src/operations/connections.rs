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
pub struct ListConnections {
    resources: Arc<Resources>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Connections {
    #[row(description = "The resource ids, one per unit the listener heard about.")]
    pub resources: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "mini-sidecar.list_connections",
    version = 1,
    description = "Reports the units this process has been told about. The connection listener accumulates them across executions; a template calls the operation without holding the plugin.",
    policy = super::POLICY
)]
impl ListConnections {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<Connections, Refusal> {
        Ok(Connections {
            resources: self.resources.connected(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
