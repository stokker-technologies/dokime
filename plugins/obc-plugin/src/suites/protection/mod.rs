/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod overcurrent;

pub use overcurrent::{Overcurrent, Samples};

pub const SUITE: &str = "obc.charger.protection";
pub const TEMPLATE: &str = "obc.charger.protection.overcurrent";

pub struct Suite;

#[dokime::catalog]
pub enum Templates {
    Overcurrent(overcurrent::Overcurrent),
}

#[dokime::suite(id = SUITE, label = "Charger Protection", tags = ["device", "protection"], templates = Templates, description = "Verify that the Charger's protection disconnects it once the load draws too much current.")]
impl Suite {}
