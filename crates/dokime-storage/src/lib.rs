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
mod backend;
mod client_reads;
mod content;
mod content_name;
mod cycles;
mod document_values;
mod error;
mod evidence_positions;
mod execution;
mod expressions;
mod file_uploads;
mod history;
mod ipc;
mod notes;
mod ordered_merge;
mod outcome;
mod plans;
mod plot_kinds;
mod plot_selection;
mod plot_windows;
mod query;
mod report;
mod resource_values;
mod scalar;
mod store;
mod stream_windows;
mod telemetry;

/* === Re-Exports === */

pub use backend::Backend;
pub use content::{ArchivedContent, ArchivedContentChunk, CONTENT_CHUNK_BYTES, ContentEntry, CycleArchive};
pub use content_name::{validate_artifact_name, validate_execution_path};
pub use error::Error;
pub use execution::{Evidence, Execution, ExecutionArchive, ExecutionSnapshot, ExecutionState, Sample};
pub use outcome::OutcomeCounts;
pub use plot_kinds::{CartesianRead, KindWindow};
pub use plot_selection::{PlotGroup, PlotSelection};
pub use query::{ExecutionCut, StatisticsRead};
pub use report::Report;
pub use resource_values::ResourceValue;
pub use store::{ReadCancellation, Store};
pub use stream_windows::{NumericBin, NumericWindow, SampleWindow};
pub use telemetry::{
    BatchSelection,
    config::{PreviewSettings, StorageSettings, storage_defaults},
};

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
