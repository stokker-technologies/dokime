/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::document::DocumentFilter;
use crate::document::frame::{Lookup, Side, Strategy, nanos};
use crate::expressions::DataSource;
use crate::plot::{PlotPayloadRequest, parse_timestamp, record_samples::RecordSampleLookup};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::records::evidence::RecordEntry;

pub(crate) const RECORD_CONTEXT_STREAM: &str = "dokime.plot.record_context";

pub(super) struct RecordFilterLookup<'a> {
    filters: Vec<(&'a DocumentFilter, RecordSampleLookup<'a>)>,
}

pub(super) struct RecordPairLookup<'a> {
    records: Vec<(usize, &'a RecordEntry)>,
    starts: Lookup<'a>,
}

#[derive(Clone, Copy)]
pub(super) struct RecordPair<'a> {
    pub(super) start: &'a RecordEntry,
    pub(super) end: &'a RecordEntry,
    pub(super) seconds: f64,
}

impl<'a> RecordFilterLookup<'a> {
    pub(super) fn try_new(telemetry: &'a [TelemetrySample], request: &'a PlotPayloadRequest) -> crate::runtime::Result<Self> {
        let mut filters = Vec::new();
        filters.try_reserve_exact(request.filters.len()).map_err(|_| record_memory())?;
        for filter in &request.filters {
            let stream = filter_stream(telemetry, request, filter);
            let lookup = RecordSampleLookup::try_for_stream_matching(telemetry, stream, request.scope.as_ref(), |_| true)?;
            filters.push((filter, lookup));
        }
        Ok(Self { filters })
    }

    pub(super) fn matches(&self, record: &RecordEntry) -> bool {
        if self.filters.is_empty() {
            return true;
        }
        let Some(timestamp) = parse_timestamp(record.timestamp.as_str()) else {
            return false;
        };

        self.filters.iter().all(|(filter, lookup)| {
            lookup
                .nearest_at_or_before(record.scope.as_ref(), timestamp)
                .is_some_and(|sample| super::payload::sample_matches_filters(sample.sample, std::slice::from_ref(*filter)))
        })
    }
}

impl<'a> RecordPairLookup<'a> {
    pub(super) fn try_new(records: impl Iterator<Item = &'a RecordEntry> + Clone, previous: &str) -> crate::runtime::Result<Self> {
        let mut ordered = Vec::new();
        ordered.try_reserve_exact(records.clone().count()).map_err(|_| record_memory())?;
        ordered.extend(records.enumerate());
        ordered.sort_unstable_by(|(left_order, left), (right_order, right)| {
            left.sequence.cmp(&right.sequence).then(left_order.cmp(right_order))
        });
        let starts = Lookup::try_new(
            ordered.iter().enumerate().filter(|(_, (_, record))| record.record_type.as_str() == previous).map(|(ordinal, (_, record))| {
                Side {
                    ordinal,
                    scope: record.scope.as_ref(),
                    time: nanos(&record.timestamp),
                    present: true,
                }
            }),
            Strategy::Previous,
        )?;
        Ok(Self { records: ordered, starts })
    }

    pub(super) fn pairs<'b>(&'b self, end: &'b str) -> impl Iterator<Item = RecordPair<'a>> + Clone + 'b {
        self.records.iter().enumerate().filter_map(move |(ordinal, (_, record))| {
            if record.record_type.as_str() != end {
                return None;
            }
            let side = Side {
                ordinal,
                scope: record.scope.as_ref(),
                time: nanos(&record.timestamp),
                present: true,
            };
            let start = self.records.get(self.starts.find(&side)?)?.1;
            let seconds = start.timestamp.millis_until(&record.timestamp)?.0 as f64 / 1_000.0;
            Some(RecordPair { start, end: record, seconds })
        })
    }
}

fn record_memory() -> crate::runtime::Error {
    harmos::Error::ResourceExhausted { resource: "record correlation" }.into()
}

fn filter_stream<'a>(telemetry: &[TelemetrySample], request: &'a PlotPayloadRequest, filter: &'a DocumentFilter) -> &'a str {
    match &filter.expression.source {
        DataSource::Telemetry { stream, .. } => stream.as_str(),
        DataSource::Dimension { .. } | DataSource::Parameter { .. }
            if telemetry.iter().any(|sample| sample.stream.as_str() == RECORD_CONTEXT_STREAM) =>
        {
            RECORD_CONTEXT_STREAM
        }
        _ => request.stream.as_str(),
    }
}
