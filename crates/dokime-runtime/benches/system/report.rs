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
    io::{BufWriter, Write},
    num::NonZeroUsize,
    path::Path,
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, SamplingMode, Throughput};
use futures_util::FutureExt;
use sha2::{Digest, Sha256};

// Local Imports
use dokime_domain::model;
use dokime_domain::model::identity::CommandId;
use dokime_runtime::operations::reports::MaterializeReport;
use dokime_runtime::{Runtime, RuntimeHandle, plugins::PluginBundle, runtime::command::CommandContext, transactions::CycleDefinition};
use dokime_runtime::{TestCycleId, TestExecutionId, Value};

/* === Constants === */

const STEADY_SUITE: &str = "obc.charger.steady";

const STEADY_DURATION_S: f64 = 0.5;
const SETTLE: Duration = Duration::from_secs(600);
const POLL: Duration = Duration::from_millis(20);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct MaterializationFixture {
    _root: tempfile::TempDir,
    owner: Runtime,
    runtime: RuntimeHandle,
    cycle: TestCycleId,
    request: MaterializeReport,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl MaterializationFixture {
    async fn new(concurrency: usize, duration: f64) -> Self {
        let root = tempfile::tempdir().unwrap();
        let (owner, runtime, cycle, _) = started_cycle(root.path(), concurrency, duration).await;
        let execution =
            tokio::time::timeout(SETTLE, inspected_execution(&runtime, &cycle)).await.expect("steady execution reaches inspection");
        let at = runtime.storage().flush().await.unwrap();
        let request = MaterializeReport {
            subject: model::reports::wire::ReportSubject::Execution(execution.to_string()),
            at,
        };
        Self {
            _root: root,
            owner,
            runtime,
            cycle,
            request,
        }
    }

    async fn close(mut self) {
        self.runtime.seal_cycle(self.cycle, CommandContext::default()).await.unwrap();
        drop(self.runtime);
        self.owner.shutdown().await.unwrap();
    }
}

/* === Public Functions === */

pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread().worker_threads(4).enable_all().build().unwrap();
    if std::env::var_os("DOKIME_REPORT_MEASURE_COPIES").is_some() {
        executor.block_on(large_cycle_save_measurement());
        return;
    }
    fresh_materialization(criterion, &executor);
    let root = tempfile::tempdir().unwrap();
    let (mut owner, runtime, executions) = executor.block_on(sealed_cycle(root.path()));
    let mut group = criterion.benchmark_group("report_renders_saved");
    group.throughput(Throughput::Elements(executions.len() as u64));
    group.bench_function(BenchmarkId::from_parameter(executions.len()), |bencher| {
        bencher.iter(|| {
            executor.block_on(async {
                for execution in &executions {
                    let mut report = runtime.open_execution_report(execution).await.unwrap();
                    while let Some(batch) = report.reader.next().await.unwrap() {
                        black_box(batch.bytes());
                    }
                    report.reader.completion().unwrap();
                }
            })
        });
    });
    group.finish();
    drop(runtime);
    executor.block_on(owner.shutdown()).unwrap();
}

/* === Private Functions === */

async fn large_cycle_save_measurement() {
    let mut fixture = MaterializationFixture::new(4, STEADY_DURATION_S).await;
    measure_cycle_save(&fixture.runtime, &fixture.cycle, "current").await;
    fixture.runtime.seal_cycle(fixture.cycle.clone(), CommandContext::default()).await.unwrap();
    measure_cycle_save(&fixture.runtime, &fixture.cycle, "sealed").await;
    drop(fixture.runtime);
    fixture.owner.shutdown().await.unwrap();
}

async fn measure_cycle_save(runtime: &RuntimeHandle, cycle: &TestCycleId, dataset: &str) {
    let output = tempfile::tempfile().unwrap();
    let mut writer = BufWriter::with_capacity(1024 * 1024, output);
    let started = Instant::now();
    let mut report = runtime.open_cycle_report(cycle).await.unwrap();
    let opened = started.elapsed();
    let mut first_byte = None;
    let mut bytes = 0_u64;
    let mut peak_batch = 0_usize;
    let mut digest = Sha256::new();

    while let Some(batch) = report.reader.next().await.unwrap() {
        first_byte.get_or_insert_with(|| started.elapsed());
        assert_eq!(batch.offset(), bytes);
        peak_batch = peak_batch.max(batch.bytes().len());
        writer.write_all(batch.bytes()).unwrap();
        digest.update(batch.bytes());
        bytes += batch.bytes().len() as u64;
    }
    report.reader.completion().unwrap();
    writer.flush().unwrap();
    writer.get_ref().sync_all().unwrap();
    let elapsed = started.elapsed();

    assert_eq!(bytes, report.file.length);
    assert_eq!(digest.finalize().as_slice(), report.file.sha256.as_slice());
    let result = serde_json::json!({
        "kind": "large_cycle_html_save", "dataset": dataset,
        "copies": std::env::var("DOKIME_REPORT_MEASURE_COPIES").unwrap(),
        "runtime_ns": elapsed.as_nanos(), "open_ns": opened.as_nanos(),
        "first_byte_ns": first_byte.map(|duration| duration.as_nanos()),
        "delivery_peak_retained_bytes": peak_batch + 1024 * 1024,
        "file_bytes": bytes, "delivery_file_passes": 1,
        "materialization_peak_retained_bytes": null, "rows_scanned": null,
        "materialization_file_passes": null, "ingest_latency_ns": null,
        "instrumentation_note": "Null fields require source-owned counters; delivery peak excludes materialization.",
    });
    println!("{result}");
}

fn fresh_materialization(criterion: &mut Criterion, executor: &tokio::runtime::Runtime) {
    let mut group = criterion.benchmark_group("report_materializes_execution");
    // Every iteration materializes the same durable boundary of an unsealed execution.
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(1));
    for duration in [0.5, 2.0] {
        for concurrency in [1, 2, 4] {
            let mut fixture = None;
            group.bench_function(BenchmarkId::new(format!("{duration}s"), concurrency), |bencher| {
                let fixture = fixture.get_or_insert_with(|| executor.block_on(MaterializationFixture::new(concurrency, duration)));

                bencher.iter_custom(|iterations| {
                    executor.block_on(async {
                        let mut elapsed = Duration::ZERO;
                        for _ in 0..iterations {
                            elapsed += materialization(&fixture.runtime, &fixture.request).await;
                        }
                        elapsed
                    })
                });
            });
            if let Some(fixture) = fixture {
                executor.block_on(fixture.close());
            }
        }
    }
    group.finish();
}

async fn materialization(runtime: &RuntimeHandle, request: &MaterializeReport) -> Duration {
    let materialize = MaterializeReport {
        subject: request.subject.clone(),
        at: request.at.clone(),
    };
    let started = Instant::now();
    let report = runtime.operations().call::<MaterializeReport>(materialize).await.unwrap();
    let elapsed = started.elapsed();
    assert!(report.report.length > 0);
    let bytes = dokime_runtime::storage::read_file_bytes(runtime.storage(), &report.report).await.unwrap();
    let html = std::str::from_utf8(bytes.as_slice()).unwrap();
    let json = html.split_once(r#"<script id="hub-report-data" type="application/json">"#).unwrap().1.split_once("</script>").unwrap().0;
    let payload: serde_json::Value = serde_json::from_str(json).unwrap();
    assert!(plot_payload_count(&payload) >= 5, "steady report must materialize its five measurement plots");
    elapsed
}

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
            .map(|(key, value)| if key == "plots" { value.as_object().map_or(0, |plots| plots.len()) } else { plot_payload_count(value) })
            .sum(),
        serde_json::Value::Array(values) => values.iter().map(plot_payload_count).sum(),
        _ => 0,
    }
}

async fn sealed_cycle(root: &Path) -> (Runtime, RuntimeHandle, Vec<TestExecutionId>) {
    let (owner, runtime, cycle, request) = started_cycle(root, 4, STEADY_DURATION_S).await;
    let settled = settle(&runtime, &cycle, &request).boxed_local();
    let executions = tokio::time::timeout(SETTLE, settled).await.expect("the steady cycle seals");
    (owner, runtime, executions)
}

async fn started_cycle(root: &Path, concurrency: usize, duration: f64) -> (Runtime, RuntimeHandle, TestCycleId, CommandId) {
    let bundle = root.join("obc-plugin.dokime");
    std::fs::copy(super::plugin_bundle(), &bundle).unwrap();
    let mut config = dokime_runtime::RuntimeConfig::new(
        root,
        dokime_runtime::Metadata::new(dokime_runtime::Identifier::new("bench.report").unwrap(), "Report Bench", None),
    )
    .with_plugins([PluginBundle::new(bundle)]);
    config.storage.request.max_active_queries = NonZeroUsize::new(concurrency).unwrap();
    let owner = Runtime::start(config).boxed_local().await.unwrap();
    let runtime = owner.handle();
    let registration = runtime
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.suite_id == STEADY_SUITE)
        .expect("the plugin's steady suite is registered");
    let options = dokime_runtime::definition::CaseMaterializationOptions {
        force: true,
        max_cases: usize::try_from(1_000).unwrap(),
        registration_id: dokime_runtime::model::identity::new_id(),
        first_label_number: 1,
    };
    let mut cases = Vec::new();
    for template in &registration.suite.test_case_templates {
        for mut case in
            dokime_runtime::definition::materialize_template_cases(&registration.suite_id, template, Default::default(), options.clone())
                .unwrap()
                .test_cases
        {
            case.suite = Some(registration.suite.metadata.clone());
            case.parameters.insert("duration_s".into(), Value::from(duration));
            cases.push(case);
        }
    }
    if let Ok(copies) = std::env::var("DOKIME_REPORT_MEASURE_COPIES") {
        let copies: usize = copies.parse().expect("measurement copies is an integer");
        assert!((1..=128).contains(&copies));
        let originals = cases.clone();
        for _ in 1..copies {
            for original in &originals {
                let mut case = original.clone();
                case.test_case_id = dokime_runtime::model::identity::new_id();
                cases.push(case);
            }
        }
    }
    let test_cycle_id = runtime
        .create_cycle(
            CycleDefinition {
                metadata: registration.suite.metadata.clone(),
                continue_automatically: true,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let executions =
        cases.iter().map(|case| (dokime_runtime::model::identity::new_id::<TestExecutionId>(), case.test_case_id.clone())).collect();
    let registration = dokime_runtime::model::identity::new_id::<CommandId>();
    runtime.register_cases(registration, cases).await.unwrap();
    runtime.append_executions(test_cycle_id.clone(), executions).await.unwrap();
    let execution_id = runtime.watch_state().latest().state().cycles[&test_cycle_id].execution_ids[0].clone();
    let receipt =
        runtime.start_execution(execution_id, dokime_domain::model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    (owner, runtime, test_cycle_id, receipt.command_id)
}

async fn settle(runtime: &RuntimeHandle, test_cycle_id: &TestCycleId, request: &CommandId) -> Vec<TestExecutionId> {
    loop {
        for prompt in runtime.pending_prompt_projection().await {
            let _ = runtime
                .submit_prompt_response(
                    prompt.prompt_id.clone(),
                    "submit".to_owned(),
                    "operator".to_owned(),
                    BTreeMap::from([("operator_note".to_owned(), Value::from("steady")), ("accepted".to_owned(), Value::from(true))]),
                    true,
                )
                .boxed_local()
                .await;
        }
        let cycle =
            runtime.cycles().await.into_iter().find(|cycle| &cycle.test_cycle_id == test_cycle_id).expect("the steady cycle exists");
        let settled = runtime.run_request(request).await.unwrap().is_some_and(|receipt| receipt.summary.phase.terminal());
        if settled && cycle.test_executions.iter().all(|execution| execution.saved.is_some()) {
            runtime.seal_cycle(test_cycle_id.clone(), CommandContext::default()).await.unwrap();
            return cycle.test_executions.into_iter().map(|execution| execution.test_execution_id).collect();
        }
        tokio::time::sleep(POLL).await;
    }
}

/* === Tests === */
