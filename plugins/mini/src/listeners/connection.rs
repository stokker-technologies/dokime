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
use dokime::host::ToolConnectionAcquired;
use dokime::prelude::*;

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
#[dokime::listener(id = "mini-sidecar.connection", record = ToolConnectionAcquired)]
impl Observed {
    pub fn new(resources: Arc<Resources>) -> Self {
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
