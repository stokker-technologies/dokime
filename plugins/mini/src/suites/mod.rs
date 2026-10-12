/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod checks;
mod measurement;
pub mod parameters;
pub mod protection;
pub mod recovery;
pub mod startup;
pub mod steady;
mod support;

/* === Re-Exports === */
pub use checks::{
    DEFAULT_SAMPLE_COUNT, DEFAULT_TOLERANCE, MiniChecks, Pulse, PulseRow, SAMPLE_INTERVAL_MS,
    pulse_value,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[dokime::catalog]
pub enum Suites {
    MiniChecks(MiniChecks),
    StartupSuite(startup::StartupSuite),
    SteadySuite(steady::SteadySuite),
    ProtectionSuite(protection::Suite),
    RecoverySuite(recovery::Suite),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
