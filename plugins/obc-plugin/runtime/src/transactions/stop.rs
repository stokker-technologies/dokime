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

// Third Party Imports
use dokime::harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::{Error, Testbench};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::harmos::transaction(harmos_path = dokime::harmos, id = "obc.bench.stop", version = 1, irreversible)]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct Stop;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<Testbench> for Stop {
    type Error = Error;
    fn check(&self, _: &Testbench) -> Result<(), Self::Error> {
        Ok(())
    }
    fn apply(&self, state: &mut Testbench) {
        state.device.stop();
        state.stopped = true;
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
