/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Atomic manual device power changes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::{Error, Testbench};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "obc.device.set_power", version = 1, irreversible)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SetPower {
    pub enabled: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<Testbench> for SetPower {
    type Error = Error;

    fn check(&self, state: &Testbench) -> Result<(), Self::Error> {
        if state.stopped {
            return Err(Error::Stopped);
        }
        if self.enabled && state.device.fault.is_some() {
            return Err(Error::FaultLatched);
        }
        Ok(())
    }

    fn apply(&self, state: &mut Testbench) {
        if self.enabled {
            state.device.start().expect("the transaction checked the latched fault atomically");
        } else {
            state.device.stop();
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
