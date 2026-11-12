/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Execution Evidence

/* === Modules === */

mod client_reads;
mod error;
mod execution;
mod notes;
mod outcome;
mod plot_kinds;
mod plot_windows;
mod query;
mod report;
mod store;
mod stream_windows;
mod streams;

/* === Re-Exports === */

pub use error::Error;
pub use execution::{Evidence, Execution, ExecutionArchive, ExecutionSnapshot, ExecutionState, Sample, SampleBatch};
pub use outcome::OutcomeCounts;
pub use plot_kinds::{CartesianRead, KindWindow};
pub use query::{ExecutionCut, StatisticsRead};
pub use report::Report;
pub use store::Store;
pub use stream_windows::{NumericBin, NumericWindow, SampleWindow};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
