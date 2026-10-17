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
use std::collections::BTreeMap;
// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::queries::{
    RuntimeCursor, RuntimeResolution, RuntimeViewFrame, RuntimeViewPayload, RuntimeWindow, reduction::reduce_timestamped_items,
};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::scope_visible_in;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeTelemetryFrame {
    pub streams: Vec<RuntimeTelemetryStreamFrame>,
    pub total_samples: usize,
    pub returned_samples: usize,
    pub downsampled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeTelemetryStreamFrame {
    pub stream: Identifier,
    pub samples: Vec<TelemetrySample>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn telemetry_sample_matches(sample: &TelemetrySample, stream_filter: Option<&Identifier>, scope: Option<&RecordScope>) -> bool {
    if stream_filter.is_some_and(|filter| &sample.stream != filter) {
        return false;
    }
    if scope.is_some_and(|scope| !scope_visible_in(sample.scope.as_ref(), scope)) {
        return false;
    }
    true
}

pub(crate) fn telemetry_frame_from_samples<'a>(
    samples: impl Iterator<Item = &'a TelemetrySample>, stream_filter: Option<&Identifier>, scope: Option<&RecordScope>,
    window: &RuntimeWindow, resolution: &RuntimeResolution,
) -> (RuntimeTelemetryFrame, Option<u64>) {
    let mut streams = BTreeMap::<Identifier, Vec<TelemetrySample>>::new();
    for sample in samples {
        if !telemetry_sample_matches(sample, stream_filter, scope) {
            continue;
        }
        streams.entry(sample.stream.clone()).or_default().push(sample.clone());
    }

    let mut stream_frames = Vec::new();
    let mut total_samples = 0;
    let mut returned_samples = 0;
    let mut downsampled = false;
    let mut high_water = None;

    for (stream, samples) in streams {
        let samples = reduce_timestamped_items(samples, window, resolution, |sample| &sample.timestamp, |sample| Some(sample.sequence));
        total_samples += samples.total;
        returned_samples += samples.returned.len();
        downsampled |= samples.returned.len() < samples.total;
        high_water = high_water.max(samples.high_water);

        if !samples.returned.is_empty() {
            stream_frames.push(RuntimeTelemetryStreamFrame { stream, samples: samples.returned });
        }
    }

    (RuntimeTelemetryFrame { streams: stream_frames, total_samples, returned_samples, downsampled }, high_water)
}

pub(crate) fn telemetry_view_frame(
    samples: &[TelemetrySample], stream_filter: Option<&Identifier>, scope: Option<&RecordScope>, window: &RuntimeWindow,
    resolution: &RuntimeResolution, request_cursor: Option<RuntimeCursor>,
) -> RuntimeViewFrame {
    let (frame, high_water) = telemetry_frame_from_samples(samples.iter(), stream_filter, scope, window, resolution);
    let cursor = RuntimeCursor(request_cursor.map_or(0, |cursor| cursor.0).max(high_water.unwrap_or(0)));
    RuntimeViewFrame { cursor, window: window.clone(), payload: RuntimeViewPayload::Telemetry { frame: Box::new(frame) } }
}

/* === Private Functions === */

/* === Tests === */
