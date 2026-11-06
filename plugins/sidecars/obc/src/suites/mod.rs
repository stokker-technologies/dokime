/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "benchmarks")]
mod benchmarks;

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

// The Catalog Attribute Requires Unconditional Variants.
macro_rules! suites {
    ($($entry:tt)*) => {
        #[dokime::catalog]
        pub enum Suites {
            $($entry)*
        }
    };
}

#[cfg(feature = "benchmarks")]
suites!(SteadySuite(steady::SteadySuite), RecoverySuite(recovery::Suite), BenchmarkSuite(benchmarks::Suite),);

#[cfg(not(feature = "benchmarks"))]
suites!(
    ObcChecks(checks::ObcChecks),
    StartupSuite(startup::StartupSuite),
    SteadySuite(steady::SteadySuite),
    ProtectionSuite(protection::Suite),
    RecoverySuite(recovery::Suite),
);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
