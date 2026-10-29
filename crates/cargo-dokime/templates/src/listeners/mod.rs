/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


pub mod unit_connected;


pub use unit_connected::UnitConnected;


#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Listeners {
    UnitConnected(UnitConnected),
}
