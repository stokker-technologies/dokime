/* ***
 *
 * {{package}}
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
use crate::{Bench, Error};
use dokime::host::ToolConnectionAcquired;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct UnitConnected {
    bench: Arc<Bench>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[dokime::listener(id = "{{owner}}.unit_connected", record = ToolConnectionAcquired)]
impl UnitConnected {
    pub async fn initialize(bench: Arc<Bench>, _: &Supervisor) -> Self {
        Self { bench }
    }

    async fn hear(&mut self, _: &Supervisor, acquired: ToolConnectionAcquired) -> Result<(), Error> {
        self.bench.observed(acquired.resource_id);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
