/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Atomic manual device power changes

use dokime::harmos::state::Transaction;
use serde::{Deserialize, Serialize};

use crate::{Error, Testbench};

#[dokime::harmos::transaction(harmos_path = dokime::harmos, id = "obc.device.set_power", irreversible)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SetPower {
    pub enabled: bool,
}

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
