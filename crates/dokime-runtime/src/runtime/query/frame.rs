/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::{
    queries::RuntimeOverviewSnapshot,
    queries::{
        RuntimeArtifactFrame, RuntimeCursor, RuntimeDocumentFrame, RuntimeExecutionFileRead, RuntimeHealthFrame,
        RuntimeInstalledPluginFrame, RuntimeNoteFrame, RuntimePromptFrame, RuntimeRecordFrame, RuntimeResolvedDocumentRead,
        RuntimeResourceFrame, RuntimeTelemetryRead, RuntimeTraceabilityRead, RuntimeWindow,
    },
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RuntimeViewFrame {
    pub cursor: RuntimeCursor,
    pub window: RuntimeWindow,
    pub payload: RuntimeViewPayload,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeOverviewFrame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<RuntimeOverviewSnapshot>,

    pub suites: usize,
    pub cycles: usize,
    pub executions: usize,
    pub installed_plugins: usize,
    pub records: usize,
    pub telemetry_streams: usize,
    pub telemetry_samples: usize,
    pub resources: usize,
    pub health: usize,
    pub traceability: usize,
    pub notes: usize,
    pub documents: usize,
    pub record_definitions: usize,
    pub telemetry_definitions: usize,
    pub artifacts: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeListFrame {
    pub count: usize,
    pub items: Vec<String>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RuntimeViewPayload {
    Empty,
    Overview { frame: Box<RuntimeOverviewFrame> },
    InstalledPlugins { frame: Box<RuntimeInstalledPluginFrame> },
    Suites { frame: RuntimeListFrame },
    Cycles { frame: RuntimeListFrame },
    Execution { frame: RuntimeListFrame },
    ExecutionFiles { frame: RuntimeExecutionFileRead },
    RecordDefinitions { frame: RuntimeListFrame },
    Records { frame: Box<RuntimeRecordFrame> },
    TelemetryDefinitions { frame: RuntimeListFrame },
    Telemetry { frame: RuntimeTelemetryRead },
    Document { frame: RuntimeResolvedDocumentRead },
    Documents { frame: Box<RuntimeDocumentFrame> },
    Artifacts { frame: Box<RuntimeArtifactFrame> },
    Resources { frame: Box<RuntimeResourceFrame> },
    Prompts { frame: Box<RuntimePromptFrame> },
    Health { frame: Box<RuntimeHealthFrame> },
    Traceability { frame: RuntimeTraceabilityRead },
    Notes { frame: Box<RuntimeNoteFrame> },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
