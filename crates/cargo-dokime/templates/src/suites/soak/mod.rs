/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


mod hold;


pub use hold::{Hold, HoldReached, SoakRow};


pub struct SoakSuite;


#[dokime::catalog]
pub enum Templates {
    Hold(hold::Hold),
}


// A suite names a record in `records = Records` only when every template it owns
// publishes it. `HoldReached` is published by the hold and by nothing else, so
// it is declared in `hold.rs` beside the step that files it and named on that
// template's own attribute — and reaches the host through the template list
// below, bound to the template that publishes it.
#[dokime::suite(
    id = "{{crate}}.soak",
    label = "Thermal Soak",
    tags = ["device", "thermal"],
    templates = Templates, description = "Hold the unit at temperature and watch it stay in band.")]
impl SoakSuite {}
