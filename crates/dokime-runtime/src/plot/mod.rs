/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod payload;
mod record_samples;
mod record_timing;
#[cfg(feature = "host")]
pub(crate) mod stored;

pub(crate) use payload::builtins;
#[cfg(feature = "storage")]
pub(crate) use payload::spilled;
pub use payload::{
    BoxSummary, DownsampleTechnique, HistogramBin, HistogramSeries, HistogramStatistics, PlotAnalysis, PlotCartesianFrame,
    PlotCartesianMode, PlotCartesianSeriesFrame, PlotComputation, PlotComputedSeries, PlotDomain, PlotEvidence, PlotMarker, PlotPayload,
    PlotPayloadDiagnostics, PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind, PlotPoint, PlotSeries, PlotSeriesSummary,
    PlotWindow, SigmaBin, SigmaSeries, WifiBandScan, WifiBandWindow, WifiChannelCount, WifiScan, declared_plot_request,
    plot_payload_request_for_descriptor,
};
pub(crate) use payload::{
    PlotPayloadContext, canonical_plot_query, enrich_plot_samples_with_cycle_context, enrich_plot_samples_with_traceability_context,
    parse_timestamp, plot_payload_diagnostics, plot_streams, plot_window_duration, resolved_format, sample_matches_scope,
    stream_matches_alias,
};
pub(crate) use record_timing::RECORD_CONTEXT_STREAM;

#[cfg(feature = "storage")]
pub(crate) use payload::{finish_plot_frame, finish_retained_plot_payload, project_retained_plot_payload};
