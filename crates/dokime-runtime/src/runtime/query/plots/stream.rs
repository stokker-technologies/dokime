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
use crate::plot::{PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct RuntimePlotStreamBatch {
    pub mode: RuntimePlotQueryMode,
    pub envelope: harmos::storage::ReadBatch<PlotPayloadEnvelope>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePlotQueryMode {
    Snapshot,
    Replace,
}

/* === Traits === */

/* === Implementations === */

impl PartialEq for RuntimePlotStreamBatch {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode && *self.envelope == *other.envelope
    }
}

/* === Public Functions === */

pub fn plot_payload_stream_kind(mode: RuntimePlotQueryMode) -> PlotPayloadStreamKind {
    match mode {
        RuntimePlotQueryMode::Snapshot => PlotPayloadStreamKind::Snapshot,
        RuntimePlotQueryMode::Replace => PlotPayloadStreamKind::Replace,
    }
}

pub fn matching_plot_payload_request<'a>(requests: &'a [PlotPayloadRequest], plot_id: &str) -> Option<&'a PlotPayloadRequest> {
    requests.iter().find(|request| request.plot_id.as_deref().unwrap_or(&request.stream) == plot_id)
}

/* === Private Functions === */

/* === Tests === */
