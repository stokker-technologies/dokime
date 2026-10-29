/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod sequence;

pub use sequence::{Samples, Startup};

pub const STARTUP_SUITE: &str = "obc.charger.startup";
pub const STARTUP_TEMPLATE: &str = "obc.charger.startup.sequence";
const STARTUP_TEMPLATE_ID: &str = "0197f000-0000-7000-8000-000000000001";

pub struct StartupSuite;

#[dokime::catalog]
pub enum Templates {
    Startup(sequence::Startup),
}

#[dokime::suite(
    id = STARTUP_SUITE,
    label = "Charger Start-Up",
    tags = ["device", "startup", "ac"],
    templates = Templates, description = "Start the Charger on AC, inspect it, and capture its settled measurements.")]
impl StartupSuite {}
