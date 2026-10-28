/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


pub mod temperature;


pub use temperature::Temperature;


#[dokime::catalog]
pub enum Observers {
    Temperature(Temperature),
}

#[dokime::catalog]
pub enum Streams {
    TemperatureRow(crate::domain::model::chamber::TemperatureRow),
}
