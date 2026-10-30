/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "runtime")]
mod kernel;
pub mod load;
#[cfg(feature = "runtime")]
pub(crate) mod sampling;

#[cfg(feature = "runtime")]
mod runtime;

/* === Re-Exports === */

#[cfg(feature = "runtime")]
pub(crate) use runtime::calibrate;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Startup Runs Eight Independent f64 FMA Accumulators On Every Logical CPU.
// Each FMA Counts Two Floating-Point Operations. Aggregate Completed Work Divided
// By Wall Time Is The Calibrated Capacity; Load Is Capacity Times OS Utilization.
// This Is A Kernel-Equivalent Estimate, Not A Hardware Peak Or Instruction Counter.
// The Kernel Stops At Half This Operation's Attempt Budget; Release And Debug
// Builds Calibrate Their Own Compiled Kernel On The Current Machine.
#[dokime::operation(id = "host.cpu.load", response = Load, policy = super::system::POLICY, visibility = Public,
    resources = [tokio::sync::Mutex<super::system::State>])]
#[derive(Clone, Default, PartialEq)]
pub struct CpuLoad {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Load {
    pub capacity_gflops: f64,
    pub utilization_percent: f64,
    pub load_gflops: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Load {
    pub fn measured(capacity_gflops: f64, utilization_percent: f64) -> Self {
        Self {
            capacity_gflops,
            utilization_percent,
            load_gflops: capacity_gflops * utilization_percent / 100.0,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
