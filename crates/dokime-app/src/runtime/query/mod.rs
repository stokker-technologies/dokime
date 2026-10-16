/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod aggregate;
pub(crate) mod artifacts;
pub mod cases;
pub mod checkpoint_records;
pub mod checkpoint_telemetry;
pub mod cycles;
pub(crate) mod documents;
pub mod execution_history;
pub mod executions;
pub(crate) mod frame;
mod handle_reads;
mod handle_view;
pub(crate) mod installed_plugins;
pub(crate) mod plots;
pub(crate) mod projections;
pub(crate) mod records;
mod reduction;
pub(crate) mod request;
pub(crate) mod runtime;
pub(crate) mod telemetry;

/* === Re-Exports === */
pub use crate::execution_runtime::outputs::{ExecutionFileContent, ExecutionFileEntry};
pub use artifacts::{
    ExecutionFileContentKind, ExecutionFileStreamState, RecordScope, RuntimeArtifactFrame,
    RuntimeExecutionFileFrame, RuntimeExecutionFileRead, RuntimeExecutionFileRecord,
};
pub use cases::{
    CaseActions, CaseProjection, CaseStatus, case_projection, historical_case_projection,
};
pub use cycles::{
    CycleFilter, CycleOrder, CycleOrderKey, CyclePageError, cycle_page, select_cycles,
};
pub use documents::{RuntimeDocumentFrame, RuntimeDocumentRecord, RuntimeResolvedDocumentFrame};
pub use executions::{
    ExecutionControls, ExecutionFilter, ExecutionOrder, ExecutionOrderKey, execution_controls,
};
pub use frame::{RuntimeListFrame, RuntimeOverviewFrame, RuntimeViewFrame, RuntimeViewPayload};
pub use installed_plugins::{RuntimeInstalledPluginFrame, RuntimeInstalledPluginRecord};
pub use plots::{
    BoxSummary, DownsampleTechnique, HistogramBin, HistogramSeries, HistogramStatistics,
    PlotAnalysis, PlotCartesianFrame, PlotCartesianMode, PlotCartesianSeriesFrame, PlotComputation,
    PlotComputedSeries, PlotDomain, PlotEvidence, PlotMarker, PlotPayload, PlotPayloadCursor,
    PlotPayloadDiagnostics, PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind,
    PlotPoint, PlotSeries, PlotSeriesSummary, PlotWindow, RuntimePlotQueryMode,
    RuntimePlotStreamBatch, SigmaBin, SigmaSeries, WifiBandScan, WifiBandWindow, WifiChannelCount,
    WifiScan, matching_plot_payload_request, merge_plot_payload_cursor,
    plot_payload_request_for_descriptor, plot_payload_stream_envelope_visible,
    plot_payload_stream_kind,
};
pub use projections::{
    RuntimeHealthFrame, RuntimeNoteFrame, RuntimePromptFrame, RuntimeResourceFrame,
    RuntimeTraceabilityFrame,
};
pub use records::RuntimeRecordFrame;
pub use request::{
    RuntimeCursor, RuntimeResolution, RuntimeViewQuery, RuntimeViewTopic, RuntimeWindow,
};
pub use runtime::{
    RuntimeChange, RuntimeCycleSummary, RuntimeExecutionSummary, RuntimeOverviewSnapshot,
    RuntimeRetentionDiagnostics, RuntimeStateNotice, RuntimeStateSnapshot,
};
pub use telemetry::{RuntimeTelemetryFrame, RuntimeTelemetryStreamFrame};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
