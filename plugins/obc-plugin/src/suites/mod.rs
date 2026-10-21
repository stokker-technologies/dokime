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
    ($($extra:tt)*) => {
        #[dokime_runtime::catalog]
        pub enum Suites {
            ObcChecks(checks::ObcChecks),
            StartupSuite(startup::StartupSuite),
            SteadySuite(steady::SteadySuite),
            ProtectionSuite(protection::Suite),
            RecoverySuite(recovery::Suite),
            $($extra)*
        }
    };
}

#[cfg(feature = "benchmarks")]
suites!(BenchmarkSuite(benchmarks::Suite),);

#[cfg(not(feature = "benchmarks"))]
suites!();

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
