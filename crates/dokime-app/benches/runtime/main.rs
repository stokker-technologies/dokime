/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod capture;
mod report;
mod storage;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{env, hint::black_box, time::Duration};
// Third Party Imports
use criterion::{Criterion, criterion_group, criterion_main};
// Local Imports
use dokime::core::{Identifier, Metadata, Timestamp, Value};
use dokime::records::{Record, RecordEntry, RecordSeverity};
use dokime::telemetry::TelemetrySample;
use dokime_app::RuntimeConfig;
use dokime_app::domains::evidence::{RecordSelection, TelemetrySelection};

/* === Constants === */

/* === Macros === */
criterion_group! {
    name = benches;
    config = benchmark_config();
    targets =
        capture::benchmark,
        report::benchmark,
        storage::benchmark,
        bench_evidence_selection_and_materialization
}

criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn bench_evidence_selection_and_materialization(criterion: &mut Criterion) {
    let counts = record_counts();
    let mut selection_group = criterion.benchmark_group("runtime_evidence_selection");

    for count in &counts {
        let count = *count;
        let records = records(count);
        let telemetry = telemetry_samples(count, "bench.telemetry");
        let record_selection = RecordSelection {
            after_sequence: Some(count / 10),
            limit: Some(1_000),
            sources: vec!["bench".to_owned()],
            record_type_prefixes: vec!["bench.".to_owned()],
            ..RecordSelection::default()
        };
        let telemetry_selection = TelemetrySelection {
            limit: Some(1_000),
            streams: vec![identifier("bench.telemetry")],
            sources: vec!["bench".to_owned()],
            signals: vec!["value".to_owned()],
            ..TelemetrySelection::default()
        };

        selection_group.bench_function(format!("events_{count}_records"), |bencher| {
            bencher.iter(|| black_box(record_selection.apply(black_box(&records))));
        });
        selection_group.bench_function(format!("telemetry_{count}_samples"), |bencher| {
            bencher.iter(|| black_box(telemetry_selection.apply(black_box(&telemetry))));
        });
    }
    selection_group.finish();
}

/* === Private Functions === */
fn records(count: u64) -> Vec<RecordEntry> {
    (0..count)
        .map(|index| {
            let mut record = RecordEntry::from_record(
                index + 1,
                Record::new(
                    Timestamp::now_utc(),
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
            let sequence = index + 1;
            let mut sample =
                TelemetrySample::new(identifier(stream), sequence, Timestamp::now_utc(), "bench");
            // `TelemetrySample::new`'s own belt already stamped
            // `values["timestamp"]` as a `Value::Datetime`, agreeing with the
            // stream's automatic `Datetime`-typed timestamp column.
            sample
                .values
                .insert("value".to_owned(), Value::Number(index as f64));
            sample
        })
        .collect()
}

fn config(root: impl Into<std::path::PathBuf>) -> RuntimeConfig {
    let root = root.into();
    RuntimeConfig::new(&root, metadata("runtime.hub", "Runtime Hub"))
}

fn record_counts() -> Vec<u64> {
    let mut counts = vec![1_000, 10_000];
    if env::var("DOKIME_EVIDENCE_BENCH_EXPENSIVE").as_deref() == Ok("1") {
        counts.push(100_000);
    }

    counts
}

fn metadata(identifier: &str, label: &str) -> Metadata {
    Metadata::new(self::identifier(identifier), label, None)
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn benchmark_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1))
}

/* === Tests === */
