/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Replay for Captures That Predate Plot-Specific Checkpoints

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

/// One replay of the execution's evidence feeding every requested preview, so
/// a document's uncheckpointed plots cost one read rather than one each.
pub(crate) async fn rebuild(
    view: &ExecutionView, local: &crate::storage::sqlite::State, requests: &[PlotPayloadRequest], limits: Limits,
    settings: &harmos::config::SchemaSettings,
) -> Result<Vec<Preview>> {
    let mut previews: Vec<_> = requests.iter().map(|_| Preview::default()).collect();
    if requests.is_empty() {
        return Ok(previews);
    }
    let mut telemetry = TelemetryReader::selected(view, local, None, None, 0, settings).await?;
    let mut records = RecordReader::selected(view, local)?;
    let mut samples = VecDeque::new();
    let mut events = VecDeque::new();
    let mut telemetry_done = false;
    let mut records_done = false;
    let streams: Vec<_> = requests.iter().map(crate::plot::plot_streams).collect();
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
        let record = match (samples.front(), events.front()) {
            (None, None) => break,
            (Some((sample, _)), Some((record, _))) => sample > record,
            (Some(_), None) => false,
            (None, Some(_)) => true,
        };
        if record {
            let (_, record) = events.pop_front().expect("record head");
            for (preview, request) in previews.iter_mut().zip(requests) {
                preview.record(record.clone(), request, limits)?;
            }
            continue;
        }
        let (_, mut sample) = samples.pop_front().expect("sample head");
        crate::plot::enrich_plot_samples_with_cycle_context(std::slice::from_mut(&mut sample), view.execution());
        for ((preview, request), streams) in previews.iter_mut().zip(requests).zip(&streams) {
            if streams.iter().any(|stream| crate::plot::stream_matches_alias(sample.stream.as_str(), stream)) {
                preview.sample(sample.clone(), request, limits)?;
            }
        }
    }
    Ok(previews)
}

/* === Private Functions === */

/* === Tests === */
