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
use time::OffsetDateTime;

// Local Imports
use crate::document::frame::{Lookup, Side, Strategy};
use crate::plot::{PlotPayloadRequest, parse_timestamp, sample_matches_scope, stream_matches_alias};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::execution::RecordScope;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct PlotSamples<'a> {
    telemetry: &'a [TelemetrySample],
    request: &'a PlotPayloadRequest,
    cutoff: Option<OffsetDateTime>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IndexedRecordSample<'a> {
    pub(crate) sample: &'a TelemetrySample,
    pub(crate) timestamp: OffsetDateTime,
}

pub(crate) struct RecordSampleLookup<'a> {
    samples: Vec<IndexedRecordSample<'a>>,
    lookup: Lookup<'a>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<'a> PlotSamples<'a> {
    pub(super) fn new(telemetry: &'a [TelemetrySample], request: &'a PlotPayloadRequest) -> Self {
        let mut selected = Self { telemetry, request, cutoff: None };
        if request.window_s.is_some() {
            let latest = selected.iter().filter_map(|sample| parse_timestamp(sample.timestamp.as_str())).max();
            selected.cutoff = latest.and_then(|latest| super::payload::plot_window_cutoff(latest, request.window_s));
        }
        selected
    }

    pub(super) fn len(&self) -> usize {
        self.iter().count()
    }

    pub(super) fn collect(&self) -> Vec<&'a TelemetrySample> {
        let mut samples = Vec::with_capacity(self.len());
        samples.extend(self.iter());
        let time_order = self.request.analysis.is_some()
            || matches!(self.request.plot_type, Some(crate::document::PlotType::Trajectory | crate::document::PlotType::FrequencySeries));
        let x_key = if time_order { "timestamp" } else { self.request.x_key.as_deref().unwrap_or("timestamp") };
        super::payload::sort_plot_samples(&mut samples, x_key);
        samples
    }

    fn iter(&self) -> impl Iterator<Item = &'a TelemetrySample> + Clone {
        self.telemetry.iter().filter(|sample| {
            let selected = stream_matches_alias(sample.stream.as_str(), &self.request.stream)
                && sample_matches_scope(sample, self.request.scope.as_ref())
                && super::payload::sample_matches_filters(sample, &self.request.filters);
            let visible = self.cutoff.is_none_or(|cutoff| parse_timestamp(sample.timestamp.as_str()).is_none_or(|time| time >= cutoff));
            selected && visible
        })
    }
}

impl<'a> RecordSampleLookup<'a> {
    pub(crate) fn try_for_stream_matching(
        telemetry: &'a [TelemetrySample], stream: &str, scope: Option<&RecordScope>, keep: impl Fn(&TelemetrySample) -> bool + Clone,
    ) -> crate::runtime::Result<Self> {
        let selected = indexed_samples(telemetry, stream, scope, keep);
        let mut samples = Vec::new();
        samples.try_reserve_exact(selected.clone().count()).map_err(|_| lookup_memory())?;
        samples.extend(selected);
        let lookup = Lookup::try_new(
            samples.iter().enumerate().map(|(ordinal, sample)| Side {
                ordinal,
                scope: sample.sample.scope.as_ref(),
                time: Some(sample.timestamp.unix_timestamp_nanos()),
                present: true,
            }),
            Strategy::AtOrBefore,
        )?;
        Ok(Self { samples, lookup })
    }

    pub(crate) fn nearest_at_or_before(
        &self, record_scope: Option<&RecordScope>, record_time: OffsetDateTime,
    ) -> Option<IndexedRecordSample<'a>> {
        let side = side(record_scope, record_time);
        let ordinal = self.lookup.find(&side)?;
        self.samples.get(ordinal).copied()
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn lookup_memory() -> crate::runtime::Error {
    harmos::Error::ResourceExhausted { resource: "record sample lookup" }.into()
}

fn side(scope: Option<&RecordScope>, time: OffsetDateTime) -> Side<'_> {
    Side {
        ordinal: 0,
        scope,
        time: Some(time.unix_timestamp_nanos()),
        present: true,
    }
}

fn indexed_samples<'a>(
    telemetry: &'a [TelemetrySample], stream: &str, scope: Option<&RecordScope>, keep: impl Fn(&TelemetrySample) -> bool + Clone,
) -> impl Iterator<Item = IndexedRecordSample<'a>> + Clone {
    telemetry.iter().filter_map(move |sample| {
        let selected = stream_matches_alias(sample.stream.as_str(), stream) && sample_matches_scope(sample, scope);
        if !selected || !keep(sample) {
            return None;
        }
        let timestamp = parse_timestamp(sample.timestamp.as_str())?;
        Some(IndexedRecordSample { sample, timestamp })
    })
}

/* === Tests === */
