/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod power;

pub use power::{PowerLoss, Samples};

pub const SUITE: &str = "obc.charger.recovery";
pub const TEMPLATE: &str = "obc.charger.recovery.power";

pub struct Suite;

#[dokime::catalog]
pub enum Templates {
    PowerLoss(power::PowerLoss),
}

#[dokime::suite(id = SUITE, label = "Charger Recovery", tags = ["device", "recovery"], templates = Templates, description = "Recover the Charger from a loss of power: clear the latched fault and verify a controlled restart.")]
impl Suite {}
