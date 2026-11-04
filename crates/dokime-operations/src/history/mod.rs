/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod browse_cycles;
mod browse_executions;
mod delete_execution;
mod download_checkpoint_artifact;
mod get_checkpoint_context;
mod get_checkpoint_document;
mod get_checkpoint_evidence;
mod get_execution_checkpoint;
mod list_checkpoint_records;
mod list_checkpoint_telemetry;
mod read_checkpoint_output;

/* === Re-Exports === */

pub use browse_cycles::BrowseCycles;
pub use browse_executions::BrowseExecutions;
pub use delete_execution::DeleteExecution;
pub use download_checkpoint_artifact::DownloadCheckpointArtifact;
pub use get_checkpoint_context::GetCheckpointContext;
pub use get_checkpoint_document::GetCheckpointDocument;
pub use get_checkpoint_evidence::GetCheckpointEvidence;
pub use get_execution_checkpoint::GetExecutionCheckpoint;
pub use list_checkpoint_records::ListCheckpointRecords;
pub use list_checkpoint_telemetry::ListCheckpointTelemetry;
pub use read_checkpoint_output::ReadCheckpointOutput;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::TRANSFER;
use super::{MUTATION, ORDINARY};

use dokime_domain::model::content::wire::*;

use dokime_domain::model::execution::wire::*;

use dokime_domain::model::execution::wire_checkpoint::*;

use dokime_domain::model::history::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    BrowseCycles(BrowseCycles),
    BrowseExecutions(BrowseExecutions),
    GetExecutionCheckpoint(GetExecutionCheckpoint),
    DeleteExecution(DeleteExecution),
    GetCheckpointDocument(GetCheckpointDocument),
    GetCheckpointEvidence(GetCheckpointEvidence),
    ReadCheckpointOutput(ReadCheckpointOutput),
    DownloadCheckpointArtifact(DownloadCheckpointArtifact),
    ListCheckpointRecords(ListCheckpointRecords),
    ListCheckpointTelemetry(ListCheckpointTelemetry),
    GetCheckpointContext(GetCheckpointContext),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
