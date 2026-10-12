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
use time::OffsetDateTime;
// Local Imports
use crate::document::frame::{Lookup, Side, Strategy};
use crate::plot::{parse_timestamp, sample_matches_scope, stream_matches_alias};
use dokime::core::RecordScope;
use dokime::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy, Debug)]
pub(crate) struct IndexedRecordSample<'a> {
    pub(crate) sample: &'a TelemetrySample,
    pub(crate) timestamp: OffsetDateTime,
}

pub(crate) struct RecordSampleLookup<'a> {
    samples: Vec<IndexedRecordSample<'a>>,
    lookup: Lookup,
    prepared: BTreeMap<Probe, Option<usize>>,
}

type Probe = (Option<String>, Option<String>, i128);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<'a> RecordSampleLookup<'a> {
    pub(crate) fn for_stream(
        telemetry: &'a [TelemetrySample],
        stream: &str,
        scope: Option<&RecordScope>,
    ) -> Self {
        Self::for_stream_matching(telemetry, stream, scope, |_| true)
    }

    pub(crate) fn for_stream_matching(
        telemetry: &'a [TelemetrySample],
        stream: &str,
        scope: Option<&RecordScope>,
        mut keep: impl FnMut(&TelemetrySample) -> bool,
    ) -> Self {
        let samples = telemetry
            .iter()
            .filter(|sample| stream_matches_alias(sample.stream.as_str(), stream))
            .filter(|sample| sample_matches_scope(sample.scope.as_ref(), scope))
            .filter(|sample| keep(sample))
            .filter_map(|sample| {
                Some(IndexedRecordSample {
                    sample,
                    timestamp: parse_timestamp(sample.timestamp.as_str())?,
                })
            })
            .collect::<Vec<_>>();
        let lookup = Lookup::new(
            samples.iter().enumerate().map(|(ordinal, sample)| Side {
                ordinal,
                scope: sample.sample.scope.as_ref(),
                time: Some(sample.timestamp.unix_timestamp_nanos()),
                present: true,
            }),
            Strategy::AtOrBefore,
        );
        Self {
            samples,
            lookup,
            prepared: BTreeMap::new(),
        }
    }

    pub(crate) fn prepare<'p>(
        &mut self,
        probes: impl IntoIterator<Item = (Option<&'p RecordScope>, OffsetDateTime)>,
    ) {
        let sides = probes
            .into_iter()
            .map(|(scope, time)| side(scope, time))
            .filter(|side| !self.prepared.contains_key(&probe(side)))
            .collect::<Vec<_>>();
        for (side, found) in sides.iter().zip(self.lookup.find_many(&sides)) {
            self.prepared.insert(probe(side), found);
        }
    }

    pub(crate) fn nearest_at_or_before(
        &self,
        record_scope: Option<&RecordScope>,
        record_time: OffsetDateTime,
    ) -> Option<IndexedRecordSample<'a>> {
        let side = side(record_scope, record_time);
        let ordinal = match self.prepared.get(&probe(&side)) {
            Some(found) => *found,
            None => self.lookup.find(&side),
        }?;
        self.samples.get(ordinal).copied()
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn side(scope: Option<&RecordScope>, time: OffsetDateTime) -> Side<'_> {
    Side {
        ordinal: 0,
        scope,
        time: Some(time.unix_timestamp_nanos()),
        present: true,
    }
}

fn probe(side: &Side<'_>) -> Probe {
    let scope = side.scope;
    (
        scope
            .and_then(|scope| scope.test_execution_id.as_ref())
            .map(|id| id.as_str().to_owned()),
        scope.and_then(|scope| match &scope.origin {
            dokime::core::EvidenceOrigin::Step { attempt_id } => {
                Some(attempt_id.as_str().to_owned())
            }
            _ => None,
        }),
        side.time.unwrap_or_default(),
    )
}
/* === Tests === */
