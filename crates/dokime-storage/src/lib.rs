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

mod administration;
mod client_reads;
mod content;
mod content_name;
mod cycles;
mod document_values;
mod error;
mod execution;
mod history;
mod notes;
mod outcome;
mod plans;
mod plot_kinds;
mod plot_selection;
mod plot_windows;
mod query;
mod report;
mod store;
mod stream_windows;
mod streams;

/* === Re-Exports === */

pub use content::{ArchivedContent, ArchivedContentChunk, CONTENT_CHUNK_BYTES, ContentEntry, CycleArchive};
pub use content_name::{validate_artifact_name, validate_execution_path};
pub use error::Error;
pub use execution::{Evidence, Execution, ExecutionArchive, ExecutionSnapshot, ExecutionState, Sample, SampleBatch};
pub use outcome::OutcomeCounts;
pub use plot_kinds::{CartesianRead, KindWindow};
pub use plot_selection::{PlotGroup, PlotSelection};
pub use query::{ExecutionCut, StatisticsRead};
pub use report::Report;
pub use store::{ReadCancellation, Store};
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
