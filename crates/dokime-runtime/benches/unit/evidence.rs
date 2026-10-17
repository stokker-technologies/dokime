/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Selection Benchmarks: Record And Telemetry Filters Over Memory

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::hint::black_box;

// Third Party Imports
use criterion::{BenchmarkId, Criterion, Throughput};

// Local Imports
use dokime_runtime::{Identifier, Timestamp, Value};
use dokime_runtime::records::{Record, RecordEntry, RecordSeverity};
use dokime_runtime::telemetry::TelemetrySample;
use dokime_runtime::domains::evidence::{RecordSelection, TelemetrySelection};

/* === Constants === */
const COUNTS: [u64; 2] = [1_000, 10_000];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("records_select_entries");
    for count in COUNTS {
        let records = records(count);
        let selection = RecordSelection {
            after_sequence: Some(count / 10),
            limit: Some(1_000),
            sources: vec!["bench".to_owned()],
            record_type_prefixes: vec!["bench.".to_owned()],
            ..RecordSelection::default()
        };
        group.throughput(Throughput::Elements(count));
        group.bench_function(BenchmarkId::from_parameter(count), |bencher| {
            bencher.iter(|| black_box(selection.apply(black_box(&records))));
        });
    }
    group.finish();

    let mut group = criterion.benchmark_group("telemetry_selects_samples");
    for count in COUNTS {
        let samples = telemetry_samples(count, "bench.telemetry");
        let selection = TelemetrySelection {
            limit: Some(1_000),
            streams: vec![identifier("bench.telemetry")],
            sources: vec!["bench".to_owned()],
            signals: vec!["value".to_owned()],
            ..TelemetrySelection::default()
        };
        group.throughput(Throughput::Elements(count));
        group.bench_function(BenchmarkId::from_parameter(count), |bencher| {
            bencher.iter(|| black_box(selection.apply(black_box(&samples))));
        });
    }
    group.finish();
}

/* === Private Functions === */
fn records(count: u64) -> Vec<RecordEntry> {
    (0..count)
        .map(|index| {
            let mut record = RecordEntry::from_record(
                index + 1,
                Record::new(
                    dokime_runtime::model::time::now_utc(),
                    identifier("bench.event"),
                    1,
                    if index % 2 == 0 { "bench" } else { "other" },
                    RecordSeverity::Info,
                ),
            );
            record
                .details
                .insert("value".to_owned(), Value::Number(index as f64));
            record
        })
        .collect()
}

fn telemetry_samples(count: u64, stream: &str) -> Vec<TelemetrySample> {
    (0..count)
        .map(|index| {
            let mut sample =
                TelemetrySample::new(dokime_runtime::model::identity::new_id(), identifier(stream), index + 1, dokime_runtime::model::time::now_utc(), "bench");
            sample
                .values
                .insert("value".to_owned(), Value::Number(index as f64));
            sample
        })
        .collect()
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

/* === Tests === */
