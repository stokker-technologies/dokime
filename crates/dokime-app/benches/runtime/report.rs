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
use std::{
    collections::BTreeMap,
    hint::black_box,
    num::NonZeroUsize,
    path::Path,
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, Throughput};

// Local Imports
use dokime::core::{CommandId, TestCycleId, TestExecutionId, Value};
use dokime_app::{
    Runtime, RuntimeHandle, plugins::PluginBundle, runtime::command::CommandContext,
    transactions::CycleDefinition,
};

/* === Constants === */
const STEADY_SUITE: &str = "mini.charger.steady";
/// Half a second at the steady suite's 1 ms step: about 500 rows per case.
const STEADY_DURATION_S: f64 = 0.5;
const SETTLE: Duration = Duration::from_secs(600);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// Measures Fresh Materialization At Admission Limits 1/2/4 Separately From
/// Rendering Immutable Saved Reports. Fixture Acquisition And Cleanup Are Untimed.
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    fresh_materialization(criterion, &executor);
    let root = tempfile::tempdir().unwrap();
    let (mut owner, runtime, executions) = executor.block_on(sealed_cycle(root.path()));
    let mut group = criterion.benchmark_group("saved_report_rendering");
    group.throughput(Throughput::Elements(executions.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("mini_steady_cycle", executions.len()),
        &executions,
        |bencher, executions| {
            bencher.iter(|| {
                executor.block_on(async {
                    for execution in executions {
                        let html = runtime
                            .render_saved_execution_report(execution)
                            .await
                            .unwrap();
                        black_box(html);
                    }
                })
            });
        },
    );
    group.finish();
    drop(runtime);
    executor.block_on(owner.shutdown()).unwrap();
}

/* === Private Functions === */
fn fresh_materialization(criterion: &mut Criterion, executor: &tokio::runtime::Runtime) {
    use dokime::operations::reports::{MaterializeReport, MaterializeReportRequest, ReportSubject};
    let mut group = criterion.benchmark_group("report_materialization");
    group.sample_size(10);
    group.throughput(Throughput::Elements(1));
    for duration in [0.5, 2.0] {
        for concurrency in [1, 2, 4] {
            group.bench_function(BenchmarkId::new(format!("mini_steady/{duration}s"), concurrency), |bencher| {
                bencher.iter_custom(|iterations| {
                    executor.block_on(async {
                        let mut elapsed = Duration::ZERO;
                        let mut latencies = Vec::new();
                        let mut peak_read = 0;
                        let mut peak_pending = 0;
                        let mut peak_rss = 0;
                        let mut system = sysinfo::System::new();
                        let pid = sysinfo::get_current_pid().unwrap();
                        for _ in 0..iterations {
                            let root = tempfile::tempdir().unwrap();
                            let (mut owner, runtime, cycle, request) = started_cycle(root.path(), concurrency, duration).await;
                            let execution = tokio::time::timeout(SETTLE, async {
                                'inspection: loop {
                                    // The Inspection Prompt Holds Acquisition Before Sealing.
                                    for prompt in runtime.pending_prompt_projection().await {
                                        if prompt.scope.test_cycle_id.as_ref() != Some(&cycle) {
                                            continue;
                                        }
                                        if let Some(id) = prompt.scope.test_execution_id {
                                            let current = runtime.cycles().await.into_iter().find(|item| item.test_cycle_id == cycle).unwrap();
                                            let execution = current.test_executions.iter().find(|execution| execution.test_execution_id == id).unwrap();
                                            assert!(execution.saved.is_none());
                                            assert!(execution.active_step_attempt.is_some());
                                            break 'inspection id;
                                        }
                                    }
                                    tokio::time::sleep(Duration::from_millis(20)).await;
                                }
                            }).await.expect("steady execution reaches inspection");
                            let at = runtime.storage().flush().await.unwrap();
                            let materialize = MaterializeReportRequest { subject: ReportSubject::Execution(execution.to_string()), at };
                            let started = Instant::now();
                            let report = {
                            let call = runtime.operations().call_as::<MaterializeReport>(materialize);
                            tokio::pin!(call);
                            let mut sample = tokio::time::interval(Duration::from_millis(5));
                            loop {
                                tokio::select! {
                                    result = &mut call => break result.unwrap(),
                                    _ = sample.tick() => {
                                        let status = runtime.storage().status();
                                        peak_read = peak_read.max(status.read_bytes);
                                        peak_pending = peak_pending.max(status.pending_bytes);
                                        system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::Some(&[pid]), false, sysinfo::ProcessRefreshKind::nothing().with_memory());
                                        peak_rss = peak_rss.max(system.process(pid).map_or(0, |process| process.memory()));
                                    }
                                }
                            }
                            };
                            let latency = started.elapsed();
                            elapsed += latency;
                            latencies.push(latency);
                            assert!(report.report.length > 0);
                            assert!(report.materialized_input.is_none(), "inspection must remain unsealed");
                            // Validate The Published Payload Outside The Timed Region.
                            let mut reader = runtime.storage().read_file(&report.report).await.unwrap();
                            let mut bytes = Vec::new();
                            while let Some(batch) = reader.next_chunk(NonZeroUsize::new(64 * 1024).unwrap()).await.unwrap() {
                                bytes.extend_from_slice(batch.as_bytes());
                            }
                            let html = String::from_utf8(bytes).unwrap();
                            let json = html.split_once(r#"<script id="hub-report-data" type="application/json">"#).unwrap().1.split_once("</script>").unwrap().0;
                            let payload: serde_json::Value = serde_json::from_str(json).unwrap();
                            assert!(plot_payload_count(&payload) >= 5, "steady report must materialize its five measurement plots");
                            drop(reader);
                            black_box(&report.report);
                            drop(report);
                            tokio::time::timeout(SETTLE, settle(&runtime, &cycle, &request)).await.expect("benchmark cycle seals after measurement");
                            drop(runtime);
                            owner.shutdown().await.unwrap();
                        }
                        latencies.sort_unstable();
                        if !latencies.is_empty() {
                            eprintln!("report materialization sqlite active={concurrency} plot_plans=4 acquisition_seconds={duration} samples={iterations} p50_us={} p95_us={} sampled_read_peak={peak_read} sampled_pending_peak={peak_pending} sampled_process_rss_peak={peak_rss}; 5ms sampling overhead included; subinterval peaks may be missed",
                                latencies[latencies.len() / 2].as_micros(), latencies[(latencies.len() - 1) * 95 / 100].as_micros());
                        }
                        elapsed
                    })
                });
            });
        }
    }
    group.finish();
}

fn plot_payload_count(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(fields) => fields
            .iter()
            .map(|(key, value)| {
                if key == "plots" {
                    value.as_object().map_or(0, |plots| plots.len())
                } else {
                    plot_payload_count(value)
                }
            })
            .sum(),
        serde_json::Value::Array(values) => values.iter().map(plot_payload_count).sum(),
        _ => 0,
    }
}

async fn sealed_cycle(root: &Path) -> (Runtime, RuntimeHandle, Vec<TestExecutionId>) {
    let (owner, runtime, cycle, request) = started_cycle(root, 4, STEADY_DURATION_S).await;
    let executions = tokio::time::timeout(SETTLE, settle(&runtime, &cycle, &request))
        .await
        .expect("the steady cycle seals");
    (owner, runtime, executions)
}

async fn started_cycle(
    root: &Path,
    concurrency: usize,
    duration: f64,
) -> (Runtime, RuntimeHandle, TestCycleId, CommandId) {
    let bundle = root.join("mini-sidecar.dokime");
    let prepared = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/fixtures/mini-sidecar/mini-sidecar.dokime");
    std::fs::copy(&prepared, &bundle).unwrap_or_else(|error| {
        panic!(
            "run mise run fixtures:prepare to prepare {}: {error}",
            prepared.display()
        )
    });
    let mut config = dokime_app::RuntimeConfig::new(
        root,
        dokime::core::Metadata::new(
            dokime::core::Identifier::new("bench.report").unwrap(),
            "Report Bench",
            None,
        ),
    )
    .with_plugins([PluginBundle::new(bundle)]);
    config.storage.request.max_active_queries = NonZeroUsize::new(concurrency).unwrap();
    let owner = Runtime::start(config).await.unwrap();
    let runtime = owner.handle();
    let registration = runtime
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "mini-sidecar" && suite.suite_id == STEADY_SUITE)
        .expect("Mini's steady suite is registered");
    let options = dokime::definition::CaseMaterializationOptions {
        force: true,
        ..Default::default()
    };
    let mut cases = Vec::new();
    for template in &registration.suite.test_case_templates {
        for mut case in dokime::definition::materialize_template_cases(
            &registration.suite_id,
            template,
            Default::default(),
            options.clone(),
        )
        .unwrap()
        .test_cases
        {
            case.suite = Some(registration.suite.metadata.clone());
            case.parameters
                .insert("duration_s".into(), Value::from(duration));
            cases.push(case);
        }
    }
    let test_cycle_id = runtime
        .start_cycle(
            CycleDefinition {
                metadata: registration.suite.metadata.clone(),
                suite: registration.suite,
                cases,
                continue_on_failure: true,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    (owner, runtime, test_cycle_id, receipt.command_id)
}

/// Accepts every steady inspection until the cycle report is sealed and its
/// run has settled, so no finalization still holds storage reads.
async fn settle(
    runtime: &RuntimeHandle,
    test_cycle_id: &TestCycleId,
    request: &CommandId,
) -> Vec<TestExecutionId> {
    loop {
        for prompt in runtime.pending_prompt_projection().await {
            let _ = runtime
                .submit_prompt_response(
                    prompt.prompt_id.clone(),
                    "submit".to_owned(),
                    "operator".to_owned(),
                    BTreeMap::from([
                        ("operator_note".to_owned(), Value::from("steady")),
                        ("accepted".to_owned(), Value::from(true)),
                    ]),
                    true,
                )
                .await;
        }
        let cycle = runtime
            .cycles()
            .await
            .into_iter()
            .find(|cycle| &cycle.test_cycle_id == test_cycle_id)
            .expect("the steady cycle exists");
        let settled = runtime
            .run_request(request)
            .await
            .unwrap()
            .is_some_and(|receipt| receipt.summary.phase.terminal());
        if cycle.saved.is_some() && settled {
            return cycle
                .test_executions
                .into_iter()
                .map(|execution| execution.test_execution_id)
                .collect();
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
