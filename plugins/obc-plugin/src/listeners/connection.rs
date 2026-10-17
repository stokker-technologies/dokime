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
use crate::{Resources, error::Error};
use dokime_runtime::host::ToolConnectionAcquired;
use dokime_runtime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Observed {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::listener(id = "obc-plugin.connection", record = ToolConnectionAcquired)]
impl Observed {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn hear(
        &mut self,
        _: &Supervisor,
        acquired: ToolConnectionAcquired,
    ) -> Result<(), Error> {
        self.resources.observed(acquired.resource_id);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
