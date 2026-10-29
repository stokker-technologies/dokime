/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime::harmos::state::Transaction;
use serde::{Deserialize, Serialize};

use crate::{Error, Testbench};

#[dokime::harmos::transaction(harmos_path = dokime::harmos, id = "obc.bench.stop")]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct Stop;

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
