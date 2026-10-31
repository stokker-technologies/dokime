/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Replay For Captures That Predate Plot-Specific Checkpoints

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::VecDeque;

// Third Party Imports

// Local Imports
use super::{execution::ExecutionView, records::RecordReader, telemetry::TelemetryReader};
use crate::plot::{
    PlotPayloadRequest,
    preview::{Limits, Preview},
};
use crate::runtime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn rebuild(
    view: &ExecutionView, request: &PlotPayloadRequest, limits: Limits, settings: &harmos::config::SchemaSettings,
) -> Result<Preview> {
    let mut telemetry = TelemetryReader::selected(view, None, None, 0, settings).await?;
    let mut records = RecordReader::open(view, true).await?;
    let mut samples = VecDeque::new();
    let mut events = VecDeque::new();
    let mut telemetry_done = false;
    let mut records_done = false;
    let streams = crate::plot::plot_streams(request);
    let mut preview = Preview::default();
    loop {
        if samples.is_empty() && !telemetry_done {
            match telemetry.next_positioned().await? {
                Some(page) => samples.extend(page.iter().map(|row| (row.position, row.captured.sample.clone()))),
                None => telemetry_done = true,
            }
        }
        if events.is_empty() && !records_done {
            match records.next_positioned().await? {
                Some(page) => events.extend(page.iter().map(|row| (row.position, row.captured.record.clone()))),
                None => records_done = true,
            }
        }
        match (samples.front(), events.front()) {
            (None, None) => break,
            (Some((sample, _)), Some((record, _))) if sample > record => {
                let (_, record) = events.pop_front().expect("record head");
                preview.record(record, request, limits)?;
            }
            (Some(_), _) => {
                let (_, mut sample) = samples.pop_front().expect("sample head");
                if streams.iter().any(|stream| crate::plot::stream_matches_alias(sample.stream.as_str(), stream)) {
                    crate::plot::enrich_plot_samples_with_cycle_context(std::slice::from_mut(&mut sample), view.execution());
                    preview.sample(sample, request, limits)?;
                }
            }
            (None, Some(_)) => {
                let (_, record) = events.pop_front().expect("record head");
                preview.record(record, request, limits)?;
            }
        }
    }
    Ok(preview)
}

/* === Private Functions === */

/* === Tests === */
