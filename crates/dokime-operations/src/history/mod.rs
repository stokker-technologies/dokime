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
use super::Endpoint;
use super::TRANSFER;
use super::{MUTATION, ORDINARY};

#[allow(unused_imports)]
use dokime_domain::model::appearance::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::catalog::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::content::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::definition::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::documents::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::evidence::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::execution::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::execution::wire_checkpoint::*;

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::notes::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::plots::wire::*;

#[allow(unused_imports)]
use dokime_domain::model::prompts::wire::*;

use dokime_domain::model::history::wire::*;
#[allow(unused_imports)]
use dokime_domain::model::telemetry::wire::*;

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

impl Endpoint for BrowseCycles {
    const PATH: &'static str = "/dokime.v1.History/BrowseCycles";
}

impl Endpoint for BrowseExecutions {
    const PATH: &'static str = "/dokime.v1.History/BrowseExecutions";
}

impl Endpoint for GetExecutionCheckpoint {
    const PATH: &'static str = "/dokime.v1.History/GetExecutionCheckpoint";
}

impl Endpoint for DeleteExecution {
    const PATH: &'static str = "/dokime.v1.History/DeleteExecution";
}

impl Endpoint for GetCheckpointDocument {
    const PATH: &'static str = "/dokime.v1.History/GetCheckpointDocument";
}

impl Endpoint for GetCheckpointEvidence {
    const PATH: &'static str = "/dokime.v1.History/GetCheckpointEvidence";
}

impl Endpoint for ReadCheckpointOutput {
    const PATH: &'static str = "/dokime.v1.History/ReadCheckpointOutput";
}

impl Endpoint for DownloadCheckpointArtifact {
    const PATH: &'static str = "/dokime.v1.History/DownloadCheckpointArtifact";
}

impl Endpoint for ListCheckpointRecords {
    const PATH: &'static str = "/dokime.v1.History/ListCheckpointRecords";
}

impl Endpoint for ListCheckpointTelemetry {
    const PATH: &'static str = "/dokime.v1.History/ListCheckpointTelemetry";
}

impl Endpoint for GetCheckpointContext {
    const PATH: &'static str = "/dokime.v1.History/GetCheckpointContext";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
