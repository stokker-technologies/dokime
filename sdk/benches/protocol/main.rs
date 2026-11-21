/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Measure Product Protocol Encoding and Decoding

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{env, error::Error, fs, hint::black_box, path::PathBuf};

// Third Party Imports
use criterion::Criterion;
use prost::Message;
use prost_types::Timestamp;

// Local Imports
use dokime_protocol::{host, model};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<(), Box<dyn Error>> {
    let mut criterion = Criterion::default().configure_from_args();
    let timestamp = Timestamp { seconds: 1_700_000_000, nanos: 0 };
    let scope = model::EvidenceScope {
        execution_id: Some("00000000-0000-7000-8000-000000000001".into()),
        ..Default::default()
    };
    let records = host::evidence::RecordsListResponse {
        records: (0..128)
            .map(|index| host::evidence::CapturedRecord {
                record_id: format!("00000000-0000-7000-8000-{index:012x}"),
                record_type: "thermal.cycle.door".into(),
                sequence: index,
                schema_version: 1,
                timestamp: Some(timestamp),
                received_at: Some(timestamp),
                scope: Some(scope.clone()),
                payload: vec![8, 1],
                ..Default::default()
            })
            .collect(),
        count: 128,
        ..Default::default()
    };
    let samples = host::evidence::SamplesListResponse {
        samples: (0..1024)
            .map(|index| host::evidence::SampleRow {
                kind: Some(host::evidence::sample_row::Kind::Raw(host::evidence::TelemetrySample {
                    sample_id: format!("00000000-0000-7000-8000-{index:012x}"),
                    stream: "thermal.cycle.counter".into(),
                    schema_version: 1,
                    sequence: index,
                    timestamp: Some(timestamp),
                    received_at: Some(timestamp),
                    scope: Some(scope.clone()),
                    payload: vec![8, 1],
                    ..Default::default()
                })),
            })
            .collect(),
        ..Default::default()
    };
    let frame = host::plots::PlotFrame {
        time: (0..1024).map(|index| index as f64 / 1000.0).collect(),
        series: (0..4)
            .map(|_| host::plots::PlotColumn {
                y: (0..1024).map(|index| host::plots::NullableNumber { value: Some(index as f64) }).collect(),
                ..Default::default()
            })
            .collect(),
        point_count: 1024,
        signature: "thermal.cycle.counter".into(),
        ..Default::default()
    };
    let report = host::ReportsGetResponse {
        report: Some(host::ReportMetadata {
            test_cycle_id: "00000000-0000-7000-8000-000000000002".into(),
            test_execution_id: Some("00000000-0000-7000-8000-000000000001".into()),
            identifier: "thermal.report".into(),
            metadata: Some(model::Metadata {
                identifier: "thermal.report".into(),
                label: "Thermal Report".into(),
                ..Default::default()
            }),
            generated_at: Some(timestamp),
            origin: host::ReportOrigin::Sealed as i32,
            ..Default::default()
        }),
        ..Default::default()
    };
    let sizes = [
        measure(&mut criterion, "protocol/evidence/records_128", records),
        measure(&mut criterion, "protocol/samples/rows_1024", samples),
        measure(&mut criterion, "protocol/plots/series_4_points_1024", frame),
        measure(&mut criterion, "protocol/reports/response", report),
    ];
    let home = PathBuf::from(env::var("CRITERION_HOME").map_err(|_| "Record Protocol Sizes: Run the Project Bench Task")?);
    fs::create_dir_all(&home)?;
    fs::write(home.join("encoded-sizes.json"), format!("{{{}}}", sizes.join(",")))?;
    criterion.final_summary();
    Ok(())
}

fn measure<T: Message + Default>(criterion: &mut Criterion, name: &str, message: T) -> String {
    let encoded = message.encode_to_vec();
    criterion.bench_function(&format!("{name}/encode"), |bench| bench.iter(|| black_box(&message).encode_to_vec()));
    criterion.bench_function(&format!("{name}/decode"), |bench| {
        bench.iter(|| T::decode(black_box(encoded.as_slice())).expect("Decode the Encoded Protocol Fixture"))
    });
    format!("\"{name}/size\":{}", encoded.len())
}

/* === Tests === */
