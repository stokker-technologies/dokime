/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use crate::queries::{
    RuntimeCursor, RuntimeResolution, RuntimeViewFrame, RuntimeViewPayload, RuntimeWindow, reduction::reduce_timestamped_items,
};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::scope_visible_in;
use dokime_domain::model::identity::Identifier;

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

#[derive(Clone)]
pub struct RuntimeTelemetryRead(harmos::storage::ReadBatch<RuntimeTelemetryFrame>);

impl std::ops::Deref for RuntimeTelemetryRead {
    type Target = RuntimeTelemetryFrame;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Debug for RuntimeTelemetryRead {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&*self.0, formatter)
    }
}
impl PartialEq for RuntimeTelemetryRead {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}
impl Serialize for RuntimeTelemetryRead {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

pub(crate) fn telemetry_sample_matches(sample: &TelemetrySample, stream_filter: Option<&Identifier>, scope: Option<&RecordScope>) -> bool {
    if stream_filter.is_some_and(|filter| &sample.stream != filter) {
        return false;
    }
    if scope.is_some_and(|scope| !scope_visible_in(sample.scope.as_ref(), scope)) {
        return false;
    }
    true
}

pub(crate) fn telemetry_view_frame(
    samples: harmos::storage::ReadBatch<Vec<TelemetrySample>>, stream_filter: Option<&Identifier>, scope: Option<&RecordScope>,
    window: &RuntimeWindow, resolution: &RuntimeResolution, request_cursor: Option<RuntimeCursor>,
) -> crate::runtime::Result<RuntimeViewFrame> {
    // Stored Positions Provide Stable Ordering Within Each Stream.
    let samples = samples.try_map(|mut samples| -> crate::runtime::Result<_> {
        samples.retain(|sample| telemetry_sample_matches(sample, stream_filter, scope));
        samples.sort_unstable_by(|left, right| left.stream.cmp(&right.stream).then(left.sequence.cmp(&right.sequence)));
        Ok(samples)
    })?;
    let mut groups = 0usize;
    let mut previous = None;
    for sample in samples.iter() {
        if previous != Some(&sample.stream) {
            groups += 1;
            previous = Some(&sample.stream);
        }
    }
    let projected = samples.try_map(|samples| -> crate::runtime::Result<_> {
        let mut streams = Vec::new();
        streams
            .try_reserve_exact(groups)
            .map_err(|error| crate::runtime::Error::Operation(format!("telemetry frame allocation failed: {error}")))?;
        let (mut total_samples, mut returned_samples, mut downsampled, mut high_water) = (0, 0, false, None);
        let mut remaining = samples.into_iter();
        while let Some(first) = remaining.as_slice().first() {
            let stream = first.stream.clone();
            let count = remaining.as_slice().iter().take_while(|sample| sample.stream == stream).count();
            let mut samples = Vec::new();
            samples
                .try_reserve_exact(count)
                .map_err(|error| crate::runtime::Error::Operation(format!("telemetry group allocation failed: {error}")))?;
            samples.extend(remaining.by_ref().take(count));
            let reduced = reduce_timestamped_items(samples, window, resolution, |sample| &sample.timestamp, |sample| Some(sample.sequence));
            total_samples += reduced.total;
            returned_samples += reduced.returned.len();
            downsampled |= reduced.returned.len() < reduced.total;
            high_water = high_water.max(reduced.high_water);
            if !reduced.returned.is_empty() {
                streams.push(RuntimeTelemetryStreamFrame { stream, samples: reduced.returned });
            }
        }
        Ok((
            RuntimeTelemetryFrame {
                streams,
                total_samples,
                returned_samples,
                downsampled,
            },
            high_water,
        ))
    })?;
    let cursor = RuntimeCursor(request_cursor.map_or(0, |cursor| cursor.0).max(projected.1.unwrap_or(0)));
    let frame = projected.try_map(|(frame, _)| -> crate::runtime::Result<_> { Ok(frame) })?;
    Ok(RuntimeViewFrame {
        cursor,
        window: window.clone(),
        payload: RuntimeViewPayload::Telemetry { frame: RuntimeTelemetryRead(frame) },
    })
}
