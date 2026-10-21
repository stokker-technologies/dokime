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
use crate::domain::model::connections::Connections;
use crate::domain::operations::ListConnections;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ConnectionListing {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[dokime::operation]
impl ConnectionListing {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, _: &mut ListConnections) -> Result<Connections, Refusal> {
        Ok(Connections { resources: self.resources.connected() })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
