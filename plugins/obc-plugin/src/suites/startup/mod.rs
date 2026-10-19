/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod sequence;

/* === Re-Exports === */

pub use sequence::{Samples, Startup};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub const STARTUP_SUITE: &str = "obc.charger.startup";
pub const STARTUP_TEMPLATE: &str = "obc.charger.startup.sequence";
const STARTUP_TEMPLATE_ID: &str = "0197f000-0000-7000-8000-000000000001";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct StartupSuite;

/* === Enums === */

#[dokime_runtime::catalog]
pub enum Templates {
    Startup(sequence::Startup),
}

/* === Traits === */

/* === Implementations === */

#[dokime_runtime::suite(
    id = STARTUP_SUITE,
    label = "Charger Power-Up",
    tags = ["device", "startup", "ac"],
    templates = Templates, description = "Observe AC startup current, inspect the Charger, and capture settled measurements.")]
impl StartupSuite {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
