/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod power;
mod stop;

pub use power::SetPower;
pub use stop::Stop;

use serde::{Deserialize, Serialize};

use crate::{Error, Testbench};

#[dokime::harmos::transactions(harmos_path = dokime::harmos, state = Testbench, error = Error)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Transactions {
    SetPower(SetPower),
    Stop(Stop),
}
