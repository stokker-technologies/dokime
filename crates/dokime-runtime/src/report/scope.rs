/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// What Report Materialization Reads Of The Application, Held By The Host
// Because An Authored Operation Context Carries No Runtime Scope.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::app::{Dokime, Resources};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Scope {
    state: harmos::state::StateManager<Dokime>,
    resources: Resources,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Scope {
    pub(crate) fn new(state: harmos::state::StateManager<Dokime>, resources: Resources) -> Self {
        Self { state, resources }
    }

    pub(crate) fn state(&self) -> &harmos::state::StateManager<Dokime> {
        &self.state
    }

    pub(crate) fn resources(&self) -> &Resources {
        &self.resources
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
