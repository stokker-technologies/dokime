/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report Benchmarks: Materializing And Rendering Reports Of Plugin Cycles

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
use criterion::{BenchmarkId, Criterion, SamplingMode, Throughput};

// Local Imports
use dokime_runtime::core::{CommandId, TestCycleId, TestExecutionId, Value};
use dokime_runtime::{
    Runtime, RuntimeHandle, plugins::PluginBundle, runtime::command::CommandContext,
    transactions::CycleDefinition,
};

/* === Constants === */
const STEADY_SUITE: &str = "obc.charger.steady";
/// Half a second at the steady suite's 1 ms step: about 500 rows per case.
const STEADY_DURATION_S: f64 = 0.5;
const SETTLE: Duration = Duration::from_secs(600);
const POLL: Duration = Duration::from_millis(20);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// Measures fresh materialization at admission limits 1/2/4 separately from
/// rendering immutable saved reports. Fixture acquisition and cleanup are untimed.
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    fresh_materialization(criterion, &executor);
    let root = tempfile::tempdir().unwrap();
    let (mut owner, runtime, executions) = executor.block_on(sealed_cycle(root.path()));
    let mut group = criterion.benchmark_group("report_renders_saved");
    group.throughput(Throughput::Elements(executions.len() as u64));
    group.bench_function(BenchmarkId::from_parameter(executions.len()), |bencher| {
        bencher.iter(|| {
            executor.block_on(async {
                for execution in &executions {
                    let html = runtime
                        .render_saved_execution_report(execution)
                        .await
                        .unwrap();
                    black_box(html);
                }
            })
        });
    });
    group.finish();
    drop(runtime);
    executor.block_on(owner.shutdown()).unwrap();
}

/* === Private Functions === */
fn fresh_materialization(criterion: &mut Criterion, executor: &tokio::runtime::Runtime) {
    let mut group = criterion.benchmark_group("report_materializes_execution");
    // Every iteration runs a cycle to inspection: equal iteration counts per sample.
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(1));
    for duration in [0.5, 2.0] {
        for concurrency in [1, 2, 4] {
            group.bench_function(
                BenchmarkId::new(format!("{duration}s"), concurrency),
                |bencher| {
                    bencher.iter_custom(|iterations| {
                        executor.block_on(async {
                            let mut elapsed = Duration::ZERO;
                            for _ in 0..iterations {
                                elapsed += materialization(concurrency, duration).await;
                            }
                            elapsed
                        })
                    });
                },
            );
        }
    }
    group.finish();
}

/// Runs one steady execution to its inspection prompt and times only the
/// report materialization of the unsealed execution.
async fn materialization(concurrency: usize, duration: f64) -> Duration {
    use dokime_runtime::operations::reports::{MaterializeReport, MaterializeReportRequest, ReportSubject};
    let root = tempfile::tempdir().unwrap();
    let (mut owner, runtime, cycle, request) =
        started_cycle(root.path(), concurrency, duration).await;
    let execution = tokio::time::timeout(SETTLE, inspected_execution(&runtime, &cycle))
        .await
        .expect("steady execution reaches inspection");
    let at = runtime.storage().flush().await.unwrap();
    let materialize = MaterializeReportRequest {
        subject: ReportSubject::Execution(execution.to_string()),
        at,
    };
    let started = Instant::now();
    let report = runtime
        .operations()
        .call_as::<MaterializeReport>(materialize)
        .await
        .unwrap();
    let elapsed = started.elapsed();
    assert!(report.report.length > 0);
    let bytes = dokime_runtime::storage::read_file_bytes(runtime.storage(), &report.report)
        .await
        .unwrap();
    let html = String::from_utf8(bytes).unwrap();
    let json = html
        .split_once(r#"<script id="hub-report-data" type="application/json">"#)
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    let payload: serde_json::Value = serde_json::from_str(json).unwrap();
    assert!(
        plot_payload_count(&payload) >= 5,
        "steady report must materialize its five measurement plots"
    );
    drop(report);
    tokio::time::timeout(SETTLE, settle(&runtime, &cycle, &request))
        .await
        .expect("benchmark cycle seals after measurement");
    drop(runtime);
    owner.shutdown().await.unwrap();
    elapsed
}

/// Polls the prompt projection until the cycle's steady execution holds its
/// inspection prompt; the sidecar exposes no inspection event.
async fn inspected_execution(runtime: &RuntimeHandle, cycle: &TestCycleId) -> TestExecutionId {
    loop {
        for prompt in runtime.pending_prompt_projection().await {
            if prompt.scope.test_cycle_id.as_ref() != Some(cycle) {
                continue;
            }
            if let Some(id) = prompt.scope.test_execution_id {
                return id;
            }
        }
        tokio::time::sleep(POLL).await;
    }
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
    let bundle = root.join("obc-plugin.dokime");
    std::fs::copy(super::plugin_bundle(), &bundle).unwrap();
    let mut config = dokime_runtime::RuntimeConfig::new(
        root,
        dokime_runtime::core::Metadata::new(
            dokime_runtime::core::Identifier::new("bench.report").unwrap(),
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
        .find(|suite| suite.suite_id == STEADY_SUITE)
        .expect("the plugin's steady suite is registered");
    let options = dokime_runtime::definition::CaseMaterializationOptions {
        force: true,
        max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
        registration_id: dokime_runtime::model::identity::new_id(),
        first_label_number: 1,
    };
    let mut cases = Vec::new();
    for template in &registration.suite.test_case_templates {
        for mut case in dokime_runtime::definition::materialize_template_cases(
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
        .create_cycle(
            CycleDefinition {
                metadata: registration.suite.metadata.clone(),
                continue_on_failure: true,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    runtime
        .append_cycle_cases(test_cycle_id.clone(), cases, CommandContext::default())
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
        tokio::time::sleep(POLL).await;
    }
}
