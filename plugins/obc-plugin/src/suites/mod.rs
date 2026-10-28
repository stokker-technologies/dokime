/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[cfg(feature = "benchmarks")]
mod benchmarks;

pub mod checks;
pub mod common;
pub mod protection;
pub mod recovery;
pub mod startup;
pub mod steady;

// The Catalog Attribute Requires Unconditional Variants.
macro_rules! suites {
    ($($extra:tt)*) => {
        #[dokime::catalog]
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
