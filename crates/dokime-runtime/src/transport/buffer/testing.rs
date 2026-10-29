/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Transport Receipt And Evidence Volume Fixtures

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::telemetry::TelemetrySample;
use dokime_domain::model::{execution::RecordScope, identity::Identifier, time::Timestamp, values::Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn records(count: u64) {
    let (cancel, _) = tokio::sync::watch::channel(false);
    let mut state = super::ExecutionState::new(Default::default(), cancel);
    let scope = RecordScope::default();
    let record = |ordinal| {
        dokime_domain::model::records::evidence::Record::new(
            dokime_domain::model::identity::RecordId::from_seed(format!("record:{ordinal}")),
            Timestamp::new("2000-01-01T00:00:00Z"),
            Identifier::new("reading").unwrap(),
            1,
            "fixture",
            dokime_domain::model::records::RecordSeverity::Info,
        )
    };
    for ordinal in 1..=count {
        let row = record(ordinal);
        assert!(state.deliver_captured(ordinal, &row, "host", Some(&scope)));
        assert!(!state.deliver_captured(ordinal, &row, "host", Some(&scope)));
        assert_eq!(state.record_ordinal, ordinal);
        assert!(state.recent_records.len() <= super::RECENT_STREAM_SAMPLES);
    }
    assert_eq!(state.captured_records.len(), count as usize);
    assert_eq!(state.captured_records.retained_rows(), 0);
    assert_eq!(state.captured_records.iter().count(), count as usize);
    let captured = state.captured_records.clone();
    assert!(!state.deliver_captured(1, &record(1), "host", Some(&scope)));
    assert!(state.cancelled());
    assert_eq!(captured.len(), count as usize);
}

pub fn stream(count: u64) {
    let (cancel, _) = tokio::sync::watch::channel(false);
    let mut state = super::ExecutionState::new(Default::default(), cancel);
    let stream = Identifier::new("reading").unwrap();
    let timestamp = Timestamp::new("2000-01-01T00:00:00Z");
    let first = TelemetrySample::new(
        dokime_domain::model::identity::TelemetrySampleId::from_seed("minute:0"),
        stream.clone(),
        0,
        timestamp.clone(),
        "fixture",
    );
    for minute in 0..count {
        let sample = TelemetrySample::new(
            dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("minute:{minute}")),
            stream.clone(),
            minute,
            timestamp.clone(),
            "fixture",
        );
        assert!(state.receive(sample.clone()).unwrap());
        assert!(!state.receive(sample).unwrap());
        let held = &state.sample_streams["reading"];
        assert!(held.recent.len() <= super::RECENT_STREAM_SAMPLES);
        assert_eq!(held.high_water, Some(minute));
    }

    assert_eq!(state.sample_streams.len(), 1);
    let receipts = state.sample_receipts();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].stream, "reading");
    let last = dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("minute:{}", count - 1));
    assert_eq!(receipts[0].sample_id, last.as_str());
    assert_eq!(state.stream_rows.retained_rows(), 0);
    assert_eq!(state.stream_rows.len(), count as usize);
    assert!(state.receive(first.clone()).is_err());
    assert_eq!(state.produced("reading"), count);
    let mut reused_sequence = first;
    reused_sequence.sample_id = dokime_domain::model::identity::TelemetrySampleId::from_seed("other");
    reused_sequence.sequence = count - 1;
    assert!(state.receive(reused_sequence).is_err());
    assert_eq!(state.produced("reading"), count);
    let oldest_recent = count - super::RECENT_STREAM_SAMPLES as u64;
    let recent = TelemetrySample::new(
        dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("minute:{oldest_recent}")),
        stream,
        oldest_recent,
        timestamp,
        "fixture",
    );
    assert!(!state.receive(recent.clone()).unwrap());
    let mut conflicting = recent.clone();
    conflicting.values.insert("value".into(), Value::from(2.0));
    assert!(state.receive(conflicting).is_err());
    assert_eq!(state.produced("reading"), count);

    let progress = state.settlement_progress();
    state.subscriptions_settling = true;
    assert!(!state.receive(recent.clone()).unwrap());
    assert_eq!(state.settlement_progress(), progress + 1);
    assert!(!state.receive(recent).unwrap());
    assert_eq!(state.settlement_progress(), progress + 1);

    let staged = state.take_staged();
    assert_eq!(staged.retained_rows(), 0);
    for (minute, sample) in staged.iter().enumerate() {
        assert_eq!(sample.sequence, minute as u64);
    }
    drop(staged);
    for receipt in &state.sample_streams["reading"].recent {
        assert_eq!(receipt.row.read().unwrap().sample_id.as_str(), receipt.sample_id);
    }
}
/* === Private Functions === */

/* === Tests === */
