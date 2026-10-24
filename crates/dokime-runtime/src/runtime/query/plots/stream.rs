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

// Local Imports
use super::request::{request_requires_context_snapshot, request_requires_stateful_expression_snapshot};
use crate::document::PlotType;
use crate::plot::{PlotPayloadCursor, PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePlotStreamBatch {
    pub mode: RuntimePlotQueryMode,
    pub envelope: PlotPayloadEnvelope,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePlotQueryMode {
    Snapshot,
    Append,
    Replace,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn plot_payload_stream_kind(mode: RuntimePlotQueryMode) -> PlotPayloadStreamKind {
    match mode {
        RuntimePlotQueryMode::Snapshot => PlotPayloadStreamKind::Snapshot,
        RuntimePlotQueryMode::Append => PlotPayloadStreamKind::Append,
        RuntimePlotQueryMode::Replace => PlotPayloadStreamKind::Replace,
    }
}

pub fn plot_payload_stream_envelope_visible(
    _mode: RuntimePlotQueryMode, envelope: &PlotPayloadEnvelope, current: &PlotPayloadRequest,
) -> bool {
    envelope.revision > current.cursor.as_ref().map(cursor_revision).unwrap_or(0)
}

pub fn matching_plot_payload_request<'a>(requests: &'a [PlotPayloadRequest], plot_id: &str) -> Option<&'a PlotPayloadRequest> {
    requests.iter().find(|request| request.plot_id.as_deref().unwrap_or(&request.stream) == plot_id)
}

pub fn merge_plot_payload_cursor(requests: &mut [PlotPayloadRequest], envelope: &PlotPayloadEnvelope) {
    if let Some(request) = requests.iter_mut().find(|request| request.plot_id.as_deref().unwrap_or(&request.stream) == envelope.plot_id) {
        request.cursor = Some(envelope.cursor.clone());
    }
}

pub(super) fn stream_requests_by_mode(requests: &[PlotPayloadRequest]) -> (Vec<PlotPayloadRequest>, Vec<PlotPayloadRequest>) {
    let mut append_requests = Vec::new();
    let mut replace_requests = Vec::new();

    for request in requests {
        if requires_stream_snapshot(request) {
            let mut request = request.clone();
            request.cursor = None;
            replace_requests.push(request);
        } else {
            append_requests.push(request.clone());
        }
    }

    (append_requests, replace_requests)
}

pub(super) fn response_cursor(
    telemetry: &[TelemetrySample], records: &[RecordEntry], previous: Option<&PlotPayloadCursor>,
) -> PlotPayloadCursor {
    PlotPayloadCursor {
        telemetry_streams: telemetry.iter().fold(
            previous.map(|cursor| cursor.telemetry_streams.clone()).unwrap_or_default(),
            |mut cursors, sample| {
                let id = crate::telemetry_runtime::telemetry_stream_id(sample.scope.as_ref(), &sample.stream);
                cursors.insert(id, sample.sequence);
                cursors
            },
        ),
        record_sequence: records.iter().map(|record| record.sequence).max().or_else(|| previous.and_then(|cursor| cursor.record_sequence)),
    }
}

pub(super) fn cursor_revision(cursor: &PlotPayloadCursor) -> u64 {
    cursor.telemetry_streams.values().copied().fold(cursor.record_sequence.unwrap_or(0), u64::saturating_add)
}

/* === Private Functions === */

fn requires_stream_snapshot(request: &PlotPayloadRequest) -> bool {
    request.cursor.is_none()
        || request.analysis.is_some()
        || request.plot_type.as_ref().is_some_and(non_appendable_plot_type)
        || request.x_key.as_deref().is_some_and(|x_key| x_key != "timestamp")
        || request_requires_context_snapshot(request)
        || request_requires_stateful_expression_snapshot(request)
}

fn non_appendable_plot_type(plot_type: &PlotType) -> bool {
    matches!(
        plot_type,
        PlotType::XyLine
            | PlotType::XyScatter
            | PlotType::XyInterval
            | PlotType::Trajectory
            | PlotType::FrequencySeries
            | PlotType::WifiChannelOccupancy
            | PlotType::Histogram
            | PlotType::BoxSummary
    )
}

/* === Tests === */
