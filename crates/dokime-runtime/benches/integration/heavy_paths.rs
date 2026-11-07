/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycle And File Read Measurements

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{sync::Arc, time::Instant};

// Third Party Imports
use criterion::Criterion;

// Local Imports
use crate::service_support::{self as support, prelude::*};

/* === Constants === */

const MEMBERS: usize = 128;
const HISTORICAL_MEMBERS: usize = 32;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn benchmark(_: &mut Criterion) {
    let Ok(selection) = std::env::var("DOKIME_HEAVY_PATHS") else { return };
    let _trace = dokime_tracing::test("heavy_paths", false);
    let executor = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    executor.block_on(measure(&selection));
}

/* === Private Functions === */

async fn measure(selection: &str) {
    let cycles = matches!(selection, "cycles" | "all" | "1");
    let files = matches!(selection, "files" | "all" | "1");
    let finalizer = matches!(selection, "finalizer" | "all" | "1");
    assert!(cycles || files || finalizer, "unknown heavy-path measurement selection");
    let members = if matches!(selection, "finalizer" | "files") { 1 } else { MEMBERS };
    let root = tempfile::tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let mut owner =
        dokime_runtime::Runtime::with_execution_fixture(support::config(root.path()), "testdb", executor.clone()).await.unwrap();
    let runtime = owner.handle();
    let suite = fixture_suite(&vec!["passed"; members]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite.clone()).await;
    let cycle = cycle_snapshot(&runtime, &cycle_id).await;
    let execution_id = &cycle.test_executions[0].test_execution_id;
    let receipt = runtime.start_execution(execution_id.clone(), Default::default(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let upload = runtime
        .begin_artifact_upload(dokime_runtime::artifacts::ArtifactUploadInput {
            scope: cycle.test_executions[0].scope.clone(),
            filename: "sample.bin".into(),
            content_type: Some("application/octet-stream".into()),
            size_bytes: 3,
            sha256: None,
        })
        .await
        .unwrap();
    upload.send(b"abc".to_vec()).await.unwrap();
    upload.finish().await.unwrap();
    drop(upload);
    let source = root.path().join("device.log");
    let bytes = vec![b'x'; 2 * 1024 * 1024];
    std::fs::write(&source, &bytes).unwrap();
    let tail = runtime.start_execution_log_tail(execution_id.clone(), "logs/device.log", source).await.unwrap();
    loop {
        let files = runtime.execution_files(execution_id).await.unwrap();
        if files.iter().any(|file| file.relative_path == "logs/device.log" && file.size_bytes == bytes.len() as u64) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    runtime.stop_execution_log_tail(&tail).await.unwrap();
    executor.release();
    support::waits::wait_for_run_settled(&runtime, &receipt.command_id).await;
    runtime.wait_execution_report(execution_id).await.unwrap();
    for execution in cycle.test_executions.iter().skip(1) {
        let started = Instant::now();
        let receipt = runtime.start_execution(execution.test_execution_id.clone(), Default::default(), CommandContext::default()).await;
        support::waits::wait_for_run_settled(&runtime, &receipt.command_id).await;
        println!("heavy member_settle_ms={}", started.elapsed().as_secs_f64() * 1000.0);
    }
    for execution in &cycle.test_executions {
        runtime.wait_execution_report(&execution.test_execution_id).await.unwrap();
    }
    if cycles {
        for _ in 0..7 {
            let started = Instant::now();
            let snapshot = runtime.inspect_cycle(&cycle_id).await.unwrap();
            assert_eq!(snapshot.state().cycles[&cycle_id].execution_ids.len(), MEMBERS);
            println!("heavy live_cycle_ms={}", started.elapsed().as_secs_f64() * 1000.0);
        }
        for _ in 0..7 {
            let started = Instant::now();
            let snapshot = runtime.watch_state().latest();
            let header = &snapshot.state().cycles[&cycle_id];
            assert_eq!(snapshot.state().cases_for_cycle(header).count(), MEMBERS);
            let _tally = dokime_runtime::runtime::query::case_tally(snapshot.state(), header);
            println!("heavy live_header_ms={}", started.elapsed().as_secs_f64() * 1000.0);
            let started = Instant::now();
            let view = snapshot.state().cycle_view(&cycle_id).unwrap();
            let cases = view
                .cases()
                .map(|case| dokime_runtime::runtime::query::case_projection(snapshot.state(), &view, &case.test_case_id))
                .collect::<Vec<_>>();
            assert_eq!(cases.len(), MEMBERS);
            println!("heavy live_case_headers_ms={}", started.elapsed().as_secs_f64() * 1000.0);
        }
    }
    let started = Instant::now();
    if finalizer {
        support::system_reports::finalize(&runtime).await;
        println!("heavy cycle_finalize_ms={}", started.elapsed().as_secs_f64() * 1000.0);
        println!("heavy finalizer_sample_members={members}");
        let reports = tempfile::tempdir().unwrap();
        for _ in 0..7 {
            let started = Instant::now();
            for execution in &cycle.test_executions {
                support::system_reports::save_execution(&runtime, &execution.test_execution_id, reports.path()).await;
            }
            println!("heavy finalizer_members_ms={}", started.elapsed().as_secs_f64() * 1000.0);
            if selection == "finalizer" {
                let checkpoint = runtime.execution_saved_checkpoint(execution_id.clone()).await.unwrap();
                let started = Instant::now();
                let view = dokime_runtime::runtime::ExecutionView::open_checkpoint(runtime.storage(), &checkpoint).await.unwrap().unwrap();
                println!("heavy duplicate_checkpoint_ms={}", started.elapsed().as_secs_f64() * 1000.0);
                let report = &view.execution().saved.as_ref().unwrap().report;
                let expected = report.sha256.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                let path = reports.path().join(execution_id.as_str()).join("report.html");
                let started = Instant::now();
                support::system_reports::verify_file(&path, report.length, &expected);
                println!("heavy duplicate_report_hash_ms={} report_bytes={}", started.elapsed().as_secs_f64() * 1000.0, report.length);
            }
        }
    } else {
        runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
        println!("heavy cycle_seal_ms={}", started.elapsed().as_secs_f64() * 1000.0);
    }
    owner.shutdown().await.unwrap();
    drop(runtime);
    if !cycles && !files {
        return;
    }
    let mut owner = dokime_runtime::Runtime::with_execution_fixture(support::config(root.path()), "testdb", executor).await.unwrap();
    let runtime = owner.handle();
    if files {
        let database = dokime_runtime::storage::sqlite::rusqlite::Connection::open(runtime.storage_root().join("local.sqlite")).unwrap();
        let object_bytes: i64 = database
            .query_row(
                "SELECT length(body) FROM storage_data_entries WHERE kind='object' AND item=?1 ORDER BY position DESC LIMIT 1",
                [execution_id.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        println!("heavy members={members} execution_object_bytes={object_bytes}");
        drop(database);
        for _ in 0..7 {
            let started = Instant::now();
            let mut length = 0;
            for offset in (0..bytes.len()).step_by(64 * 1024) {
                let window = runtime
                    .execution_file_content(execution_id, "logs/device.log".into(), offset as u64, 64 * 1024)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(window.bytes, bytes[offset..offset + 64 * 1024]);
                length += window.bytes.len();
            }
            assert_eq!(length, bytes.len());
            println!("heavy windows_32_ms={}", started.elapsed().as_secs_f64() * 1000.0);
        }
        for _ in 0..7 {
            let started = Instant::now();
            let mut stream = runtime.open_execution_file_stream(execution_id, "logs/device.log".into()).await.unwrap().unwrap();
            let mut offset = 0;
            while let Some(window) = stream.next_chunk(std::num::NonZeroUsize::new(64 * 1024).unwrap()).await.unwrap() {
                assert_eq!(window.bytes, bytes[offset..offset + window.bytes.len()]);
                offset += window.bytes.len();
            }
            assert_eq!(offset, bytes.len());
            println!("heavy resolved_windows_32_ms={}", started.elapsed().as_secs_f64() * 1000.0);
        }
    }
    if cycles {
        assert!(!runtime.watch_state().latest().state().cycles.contains_key(&cycle_id));
        for _ in 0..7 {
            let started = Instant::now();
            let header = runtime.saved_cycle_header(&cycle_id).await.unwrap().unwrap();
            assert_eq!(header.case_count as usize, MEMBERS);
            println!("heavy historical_header_ms={}", started.elapsed().as_secs_f64() * 1000.0);
            let started = Instant::now();
            let headers = runtime.saved_cycle_cases(&cycle_id).await.unwrap().unwrap();
            assert_eq!(headers.len(), MEMBERS);
            assert!(headers.iter().all(|case| case.projection.attempt_count == 1));
            println!("heavy historical_case_headers_ms={}", started.elapsed().as_secs_f64() * 1000.0);
        }
        let historical_suite = fixture_suite(&["passed"; HISTORICAL_MEMBERS]);
        register_python_suite(&runtime, historical_suite.clone()).await;
        let historical_id = start_cycle(&runtime, historical_suite).await;
        let historical = cycle_snapshot(&runtime, &historical_id).await;
        for execution in &historical.test_executions {
            let receipt = runtime.start_execution(execution.test_execution_id.clone(), Default::default(), CommandContext::default()).await;
            support::waits::wait_for_run_settled(&runtime, &receipt.command_id).await;
        }
        runtime.seal_cycle(historical_id.clone(), CommandContext::default()).await.unwrap();
        runtime
            .create_cycle(
                CycleDefinition {
                    metadata: metadata("bench.next", "Next Cycle"),
                    continue_automatically: false,
                },
                CommandContext::default(),
            )
            .await
            .unwrap();
        assert!(!runtime.watch_state().latest().state().cycles.contains_key(&historical_id));
        for _ in 0..7 {
            let started = Instant::now();
            let snapshot = runtime.inspect_cycle(&historical_id).await.unwrap();
            assert_eq!(snapshot.state().cycles[&historical_id].execution_ids.len(), HISTORICAL_MEMBERS);
            println!("heavy historical_32_cycle_ms={}", started.elapsed().as_secs_f64() * 1000.0);
            let started = Instant::now();
            let header = runtime.saved_cycle_header(&historical_id).await.unwrap().unwrap();
            assert_eq!(header.case_count as usize, HISTORICAL_MEMBERS);
            println!("heavy historical_32_header_ms={}", started.elapsed().as_secs_f64() * 1000.0);
            let started = Instant::now();
            let headers = runtime.saved_cycle_cases(&historical_id).await.unwrap().unwrap();
            assert_eq!(headers.len(), HISTORICAL_MEMBERS);
            println!("heavy historical_32_case_headers_ms={}", started.elapsed().as_secs_f64() * 1000.0);
        }
    }
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Tests === */
