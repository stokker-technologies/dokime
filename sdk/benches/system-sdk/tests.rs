/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify the Shared SDK Scenario

/* === Modules === */

pub mod scenario;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use dokime::encoding::contract::Message;
use dokime::{Declarations, StreamDeclaration};
use prost_types::Timestamp;

// Local Imports
use scenario::{Measurement, Scenario, accepted, guard, percentile};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
struct Counter {
    value: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn accepted_count_uses_host_receipt_time_with_an_exclusive_end() {
    let start = Timestamp { seconds: 5, nanos: 0 };
    let end = Timestamp { seconds: 6, nanos: 0 };
    assert!(!accepted(Timestamp { seconds: 4, nanos: 999_999_999 }, start, end));
    assert!(accepted(start, start, end));
    assert!(accepted(Timestamp { seconds: 5, nanos: 999_999_999 }, start, end));
    assert!(!accepted(end, start, end));
}

#[test]
fn scenario_separates_channels_and_ceiling_and_increases_row_rate() {
    let scenario = Scenario::default();
    assert_eq!(scenario.channels.first().unwrap(), "thermal.load.channel_000");
    assert_eq!(scenario.channels.last().unwrap(), "thermal.load.channel_127");
    assert!(!scenario.channels.contains(&scenario.ceiling));
    assert!(scenario.intervals_ns.windows(2).all(|pair| pair[0] > pair[1]));
}

#[test]
fn extra_stream_refuses_before_measurement() {
    let declarations = Declarations {
        types: Some(Counter::types()),
        streams: vec![StreamDeclaration {
            id: "thermal.load.channel_000".into(),
            row: Counter::name(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let reference = scenario::catalog(&declarations).unwrap();
    let mut candidate = declarations;
    candidate.streams.push(StreamDeclaration {
        id: "thermal.load.extra".into(),
        row: Counter::name(),
        ..Default::default()
    });
    let failure = guard(&reference, &scenario::catalog(&candidate).unwrap(), "python").unwrap_err();
    assert!(failure.contains("thermal.load.extra") && failure.contains("rust") && failure.contains("python"));
}

#[test]
fn result_shape_preserves_unavailable_measure_and_percentile_rank() {
    let measurement = Measurement::new(
        "rust",
        "memory/idle",
        "Memory [bytes]",
        "bytes",
        None,
        Some("Read Plugin Memory: Grant Process Inspection".into()),
    );
    assert_eq!(measurement.case, "sdk/rust/memory/idle");
    assert!(measurement.value.is_none() && measurement.reason.is_some());
    assert_eq!(percentile(&[4.0, 1.0, 3.0, 2.0], 50), Some(2.0));
    assert_eq!(percentile(&[], 95), None);
}
