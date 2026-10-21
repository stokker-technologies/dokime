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
pub mod common;
pub mod protection;
pub mod recovery;
pub mod startup;
pub mod steady;

/* === Re-Exports === */

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
    ObcChecks(checks::ObcChecks),
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
