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
use std::{collections::BTreeMap, hint::black_box, path::Path, time::Duration};

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
/// Materializes Mini's full report for each sealed execution of its nine-case
/// steady cycle: every record, every telemetry page, every document and plot,
/// and the rendered HTML, as cycle finalization does for each member.
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    let (mut owner, runtime, executions) = executor.block_on(sealed_cycle(root.path()));
    let mut group = criterion.benchmark_group("report_materialization");
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
async fn sealed_cycle(root: &Path) -> (Runtime, RuntimeHandle, Vec<TestExecutionId>) {
    let bundle = root.join("mini-sidecar.dokime");
    let prepared = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/fixtures/mini-sidecar/mini-sidecar.dokime");
    std::fs::copy(&prepared, &bundle).unwrap_or_else(|error| {
        panic!(
            "run mise run fixtures:prepare to prepare {}: {error}",
            prepared.display()
        )
    });
    let config = dokime_app::RuntimeConfig::new(
        root,
        dokime::core::Metadata::new(
            dokime::core::Identifier::new("bench.report").unwrap(),
            "Report Bench",
            None,
        ),
    )
    .with_plugins([PluginBundle::new(bundle)]);
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
                .insert("duration_s".into(), Value::from(STEADY_DURATION_S));
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
    let executions = tokio::time::timeout(
        SETTLE,
        settle(&runtime, &test_cycle_id, &receipt.command_id),
    )
    .await
    .expect("the steady cycle seals");
    (owner, runtime, executions)
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
