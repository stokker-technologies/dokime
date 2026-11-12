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

mod error;
mod execution;
mod outcome;
mod plans;
mod query;
mod report;
mod store;
mod streams;

/* === Re-Exports === */

pub use error::Error;
pub use execution::{Evidence, Execution, ExecutionArchive, ExecutionSnapshot, ExecutionState, Sample};
pub use outcome::OutcomeCounts;
pub use query::{ExecutionCut, StatisticsRead};
pub use report::Report;
pub use store::Store;

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
