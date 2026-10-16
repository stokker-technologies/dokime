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
use std::time::Duration;

// Third Party Imports
use tempfile::tempdir;

// Local Imports
use crate::support::plugins::fixture_bundle;
use crate::support::prelude::*;
use dokime::core::{Identifier, Metadata};
use dokime::execution::ExecutionStatus;
use dokime_app::domains::execution::requests::RunPhase;
use dokime_app::execution::CriterionStatus;
use dokime_app::runtime::command::{CommandContext, CommandStatus};
use dokime_app::runtime::progress::{LiveOperation, OperationOutcome};
use dokime_app::runtime::query::RuntimeChange;
use dokime_app::transactions::CycleDefinition;

/* === Constants === */
/// Rows each case pushes onto its own stream: many storage pages per case.
const PULSE_ROWS: u32 = 640;

/// Cases run back to back in one store. Before the pager kept its cursor in
/// the index range, every page of a later case rescanned the earlier cases'
/// rows and the terminal capture exhausted its query time by the third case.
const CASES: usize = 6;

/// Rows one large case pushes: more than the report could hold as samples
/// under its 32 MiB materialization budget. The stories proving that run as
/// their own entries in the serial `whole-read-budget` group, at the shipped
/// read ledger.
const LARGE_PULSE_ROWS: u32 = 20_000;

/// Points the fixture's pulse plot declares it shows.
const PULSE_DISPLAY_POINTS: u64 = 400;

/// Points an operator sets on the live pulse plot, above what it declares.
const LIVE_DISPLAY_POINTS: u32 = 1_500;

/// Rows of the stream the fixture's frequency plot transforms whole: many
/// storage pages, yet a fraction of the 32 MiB report budget.
const SPECTRUM_ROWS: u32 = 1_024;

/// Rows of the whole-capture stream that alone exceed the 32 MiB report budget.
const OVERSIZED_SPECTRUM_ROWS: u32 = 8_192;

/// How a report too large for its whole budget names what it ran out of.
const EXHAUSTED: &str = "memory";

/// Members of the cycle the refusing pool cannot hold at once.
const POOLED_MEMBERS: usize = 9;

/// Rows of each member's whole-capture stream: within one report's budget.
const POOLED_SPECTRUM_ROWS: u32 = 1_536;

/// Ordinary Query Deadline Also Applies To Report Reads.
const SHARED_QUERY_BUDGET: Duration = Duration::from_secs(5);

/// Artificial Backend Delay Within The Ordinary Query Deadline.
const OUTPUT_READ_STALL: Duration = Duration::from_secs(1);

/// Cases of one cycle whose terminal captures seal concurrently.
const LOGGED_CASES: usize = 5;

/// Lines each logged case writes into its sidecar stream log.
const LOGGED_LINES: u32 = 3;

/// Replay page reads the storage backend refuses at its deadline, once.
const FORCED_REPLAY_DEADLINES: u32 = 1;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
// A runtime with the suite fixture pushing the given rows.
fn sampling_config(root: &Path, settings: serde_json::Value) -> RuntimeConfig {
    config(root).with_plugins([fixture_bundle(root, "fixture-suites")
        .configured("fixture-suites", settings)
        .unwrap()])
}

// The budget-edge stories prove the shipped report budget at the shipped ledger.
fn budget_edge_config(root: &Path, settings: serde_json::Value) -> RuntimeConfig {
    let mut config = sampling_config(root, settings);
    crate::support::budgets::whole_read_budget(&mut config.storage);
    config
}

async fn run_case(handle: &RuntimeHandle, label: &str) {
    run_case_execution(handle, label, Duration::from_secs(60)).await;
}

async fn run_case_execution(
    handle: &RuntimeHandle,
    label: &str,
    bound: Duration,
) -> dokime::core::TestExecutionId {
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(Identifier::new(label).unwrap(), label, None),
                cases: materialize_cases("fixture.checks", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    // A terminal capture refused by its query budget leaves the receipt
    // nonterminal for good, so the bound here is the failure, not a wait.
    let phase = tokio::time::timeout(bound, async {
        loop {
            let receipt = handle
                .run_request(&receipt.command_id)
                .await
                .unwrap()
                .unwrap();
            if receipt.summary.phase.terminal() {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("{label} records its terminal provider state"));
    assert_eq!(
        phase.summary.phase,
        RunPhase::Completed,
        "{label}: {:?}",
        phase.interruption
    );
    let execution = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap()
        .test_executions
        .remove(0);
    assert_eq!(execution.status, ExecutionStatus::Passed, "{label}");
    let criteria = execution
        .steps
        .iter()
        .flat_map(|step| step.criteria.iter())
        .collect::<Vec<_>>();
    assert_eq!(criteria.len(), 1, "{label}");
    assert_eq!(
        criteria[0].status,
        CriterionStatus::Passed,
        "{label}: the run was handed back every pulse row exactly once"
    );
    execution.test_execution_id
}

/* === Tests === */
// cases whose own stream spans many pages replay and seal independent of history.
pub(crate) async fn paged_evidence_settles() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "pulse_rows": PULSE_ROWS }),
            )
            .unwrap()]),
    )
    .await;
    for case in 0..CASES {
        run_case(&handle, &format!("pulse.case{case}")).await;
    }
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// a multi-case cycle whose every case logs seals its outputs and goes terminal.
pub(crate) async fn logged_cycle_terminal() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "log_lines": LOGGED_LINES }),
            )
            .unwrap()]),
    )
    .await;
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let case = materialize_cases("fixture.checks", &suite).remove(0);
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite,
                metadata: Metadata::new(Identifier::new("logged.cycle").unwrap(), "Logged", None),
                cases: vec![case; LOGGED_CASES],
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let mut watch = handle.watch_state();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");

    // Each case's sealed log is produced while sibling captures flush, so a
    // checkpoint captured before a seal persists after it. That must not fail
    // terminal publication and leave the cycle running with every case passed.
    let terminal = tokio::time::timeout(
        Duration::from_secs(120),
        watch.wait_until(|snapshot| {
            snapshot.state().cycles.get(&cycle_id).is_some_and(|cycle| {
                !matches!(
                    cycle.lifecycle,
                    CycleStatus::Ready
                        | CycleStatus::Running
                        | CycleStatus::Paused
                        | CycleStatus::Stopping
                )
            })
        }),
    )
    .await
    .expect("the state watch reports the cycle terminal")
    .expect("the state watch stays open");
    let cycle = &terminal.state().cycles[&cycle_id];
    assert_eq!(cycle.lifecycle, CycleStatus::Completed);
    let phase = tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            let receipt = handle
                .run_request(&receipt.command_id)
                .await
                .unwrap()
                .unwrap();
            if receipt.summary.phase.terminal() {
                break receipt.summary.phase;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the run request records its terminal provider state");
    assert_eq!(phase, RunPhase::Completed);
    let executions = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap()
        .test_executions;
    assert_eq!(executions.len(), LOGGED_CASES);
    for execution in &executions {
        assert_eq!(execution.status, ExecutionStatus::Passed);
        crate::support::native::checkpoint(&handle, &execution.test_execution_id).await;
        let files = handle
            .execution_files(&execution.test_execution_id)
            .await
            .unwrap();
        assert!(
            files
                .iter()
                .any(|file| file.relative_path.ends_with("stream.log") && file.size_bytes > 0),
            "every logged case seals its sidecar stream log: {files:?}"
        );
    }
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// one case far larger than the report budget seals a reduced plot over every row.
pub(crate) async fn large_case_seals() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = started(budget_edge_config(
        root.path(),
        serde_json::json!({ "pulse_rows": LARGE_PULSE_ROWS }),
    ))
    .await;
    let execution_id = run_case_execution(&handle, "pulse.large", Duration::from_secs(300)).await;
    let html = crate::support::native::report(&handle, &execution_id).await;
    let payload = crate::support::reports::parse(&html);
    let series = &payload["case_runs"][0]["plots"]["fixture.checks.smoke.pulse"]["series"][0];
    let points = series["points"]
        .as_array()
        .expect("the sealed plot carries its points");
    assert!(
        !points.is_empty() && points.len() as u64 <= PULSE_DISPLAY_POINTS,
        "the sealed plot shows {} points against its declared {PULSE_DISPLAY_POINTS}",
        points.len()
    );
    assert_eq!(
        series["raw_points"],
        serde_json::json!(LARGE_PULSE_ROWS),
        "the plot was reduced from every captured row, none truncated"
    );
    assert_eq!(series["downsampled"], serde_json::json!(true));
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// a plot that needs its whole capture keeps only its own stream's rows, so a
// case whose other stream would overflow the report budget still seals both.
pub(crate) async fn whole_capture_plot_seals() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = started(budget_edge_config(
        root.path(),
        serde_json::json!({
            "pulse_rows": LARGE_PULSE_ROWS,
            "spectrum_rows": SPECTRUM_ROWS,
        }),
    ))
    .await;
    let execution_id =
        run_case_execution(&handle, "spectrum.whole", Duration::from_secs(300)).await;
    let html = crate::support::native::report(&handle, &execution_id).await;
    let payload = crate::support::reports::parse(&html);
    let plots = &payload["case_runs"][0]["plots"];
    assert_eq!(
        plots["fixture.checks.smoke.pulse"]["series"][0]["raw_points"],
        serde_json::json!(LARGE_PULSE_ROWS),
        "the streamed plot was reduced from every pulse row"
    );
    assert_eq!(
        plots["fixture.checks.smoke.spectrum"]["series"][0]["raw_points"],
        serde_json::json!(SPECTRUM_ROWS),
        "the whole-capture plot read every spectrum row across its pages"
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// a running execution's report follows the plot settings the operator chose on
// the live plot, renders again and again from the same evidence, and the
// sealed report keeps those settings.
pub(crate) async fn running_report_follows_live_plot() {
    let root = tempdir().unwrap();
    let release = root.path().join("release");
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({
                    "pulse_rows": LARGE_PULSE_ROWS,
                    "block_execution": true,
                    "execution_release": release,
                }),
            )
            .unwrap()]),
    )
    .await;
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(Identifier::new("pulse.live").unwrap(), "Live", None),
                cases: materialize_cases("fixture.checks", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    let execution =
        handle.watch_state().latest().state().cycles[&cycle_id].execution_ids[0].clone();
    let pulse = "fixture.checks.smoke.pulse";
    // The step holds once every pulse row is published.
    tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            if let Ok(opened) = handle.open_execution_report(&execution).await {
                let payload =
                    crate::support::reports::parse(std::str::from_utf8(&opened.html).unwrap());
                if payload["case_runs"][0]["plots"][pulse]["series"][0]["raw_points"]
                    == serde_json::json!(LARGE_PULSE_ROWS)
                {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("the running step publishes every pulse row");

    handle
        .update_plot_settings(
            &cycle_id,
            &execution,
            &dokime::core::DocumentId::from_identifier("default").unwrap(),
            &Identifier::new(pulse).unwrap(),
            Some(LIVE_DISPLAY_POINTS),
            dokime::document::DownsampleTechnique::Lttb,
        )
        .await
        .unwrap();
    for attempt in 1..=3 {
        let opened = handle
            .open_execution_report(&execution)
            .await
            .unwrap_or_else(|error| panic!("render {attempt} of the running report: {error}"));
        let payload = crate::support::reports::parse(std::str::from_utf8(&opened.html).unwrap());
        assert_live_plot(&payload, pulse, attempt);
        let steps = payload["test_executions"][0]["steps"].as_array().unwrap();
        assert!(
            steps
                .iter()
                .any(|row| row["record_type"] == "step" && row["status"] == "running"),
            "render {attempt} carries the running step: {steps:?}"
        );
    }

    std::fs::write(&release, b"").unwrap();
    let settled = tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            let receipt = handle
                .run_request(&receipt.command_id)
                .await
                .unwrap()
                .unwrap();
            if receipt.summary.phase.terminal() {
                break receipt.summary.phase;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(settled, RunPhase::Completed);
    let html = crate::support::native::report(&handle, &execution).await;
    assert_live_plot(&crate::support::reports::parse(&html), pulse, 4);
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// opening the report of a sizeable running cycle shows on the runtime state
// watch as one operation: it starts, moves through its phases and finishes.
pub(crate) async fn report_open_shows_progress() {
    let root = tempdir().unwrap();
    let release = root.path().join("release");
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({
                    "pulse_rows": LARGE_PULSE_ROWS,
                    "block_execution": true,
                    "execution_release": release,
                }),
            )
            .unwrap()]),
    )
    .await;
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(
                    Identifier::new("pulse.progress").unwrap(),
                    "Progress",
                    None,
                ),
                cases: materialize_cases("fixture.checks", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    let execution =
        handle.watch_state().latest().state().cycles[&cycle_id].execution_ids[0].clone();
    let pulse = "fixture.checks.smoke.pulse";
    // The step holds once every pulse row is published.
    tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            if let Ok(opened) = handle.open_execution_report(&execution).await {
                let payload =
                    crate::support::reports::parse(std::str::from_utf8(&opened.html).unwrap());
                if payload["case_runs"][0]["plots"][pulse]["series"][0]["raw_points"]
                    == serde_json::json!(LARGE_PULSE_ROWS)
                {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("the running step publishes every pulse row");

    let mut watch = handle.watch_state();
    let earlier: std::collections::BTreeSet<String> = watch
        .operations()
        .operations()
        .iter()
        .map(|operation| operation.operation_id.clone())
        .collect();
    let observer = tokio::spawn(async move {
        let mut seen = Vec::<LiveOperation>::new();
        loop {
            let snapshot = match watch.next_change().await {
                Some(RuntimeChange::Operations(snapshot)) => snapshot,
                Some(RuntimeChange::State(_)) => watch.operations(),
                None => break seen,
            };
            let Some(operation) = snapshot.operations().iter().find(|operation| {
                operation.label == "Opening Cycle Report"
                    && !earlier.contains(&operation.operation_id)
            }) else {
                continue;
            };
            if seen.last() != Some(operation) {
                seen.push(operation.clone());
            }
            if operation.outcome != OperationOutcome::Running {
                break seen;
            }
        }
    });
    let opened = handle.open_cycle_report(&cycle_id).await.unwrap();
    assert!(!opened.html.is_empty());
    let seen = tokio::time::timeout(Duration::from_secs(30), observer)
        .await
        .expect("the watch shows the report operation settle")
        .unwrap();

    let identities: std::collections::BTreeSet<_> = seen
        .iter()
        .map(|operation| &operation.operation_id)
        .collect();
    assert_eq!(
        identities.len(),
        1,
        "one operation opened the report: {seen:?}"
    );
    assert_eq!(
        seen.first().map(|operation| &operation.outcome),
        Some(&OperationOutcome::Running),
        "the watch shows the operation start: {seen:?}"
    );
    let settled = seen.last().expect("the watch shows the operation");
    assert_eq!(
        settled.phases.last(),
        Some(&settled.phase),
        "the history ends at the current phase: {settled:?}"
    );
    assert!(
        settled.phases.len() >= 2,
        "the operation changed phase: {:?}",
        settled.phases
    );
    for phase in ["Capturing Evidence", "Materializing Evidence", "Rendering"] {
        assert!(
            settled.phases.iter().any(|passed| passed == phase),
            "the history keeps {phase}: {:?}",
            settled.phases
        );
    }
    assert!(
        matches!(
            seen.last().map(|operation| &operation.outcome),
            Some(OperationOutcome::Finished { .. })
        ),
        "the watch shows the operation finish: {seen:?}"
    );
    tokio::time::sleep(dokime_app::runtime::progress::SETTLED_RETENTION * 2).await;
    assert!(
        handle
            .watch_state()
            .operations()
            .operations()
            .iter()
            .all(|operation| !identities.contains(&operation.operation_id)),
        "a finished operation retires after its retention"
    );

    std::fs::write(&release, b"").unwrap();
    tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            let receipt = handle
                .run_request(&receipt.command_id)
                .await
                .unwrap()
                .unwrap();
            if receipt.summary.phase.terminal() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    drop(handle);
    runtime.shutdown().await.unwrap();
}

fn assert_live_plot(payload: &serde_json::Value, plot: &str, attempt: usize) {
    let plot = &payload["case_runs"][0]["plots"][plot];
    assert_eq!(
        plot["max_points"],
        serde_json::json!(LIVE_DISPLAY_POINTS),
        "render {attempt} uses the live plot's points: {}",
        payload["warnings"]
    );
    let series = &plot["series"][0];
    assert_eq!(series["raw_points"], serde_json::json!(LARGE_PULSE_ROWS));
    let points = series["points"].as_array().unwrap().len();
    assert!(
        points <= LIVE_DISPLAY_POINTS as usize && points > PULSE_DISPLAY_POINTS as usize,
        "render {attempt} draws the live plot's {LIVE_DISPLAY_POINTS} points, not the authored {PULSE_DISPLAY_POINTS}: {points}"
    );
    // The frozen summary covers every pulse row, 0 to n - 1, not the drawn
    // points: the numbers the live table shows over the same window.
    let summaries = plot["summaries"].as_array().unwrap();
    assert_eq!(summaries.len(), 1, "one summary per series: {summaries:?}");
    let summary = &summaries[0];
    assert_eq!(summary["series_id"], series["key"]);
    assert_eq!(summary["count"], serde_json::json!(LARGE_PULSE_ROWS));
    let rows = f64::from(LARGE_PULSE_ROWS);
    for (statistic, expected) in [
        ("min", 0.0),
        ("max", rows - 1.0),
        ("mean", (rows - 1.0) / 2.0),
        ("median", (rows - 1.0) / 2.0),
        ("std", ((rows * rows - 1.0) / 12.0).sqrt()),
        ("rms", ((rows - 1.0) * (2.0 * rows - 1.0) / 6.0).sqrt()),
    ] {
        let actual = summary[statistic].as_f64().unwrap();
        assert!(
            (actual - expected).abs() <= 1e-9 * expected.abs().max(1.0),
            "render {attempt} {statistic}: {actual} != {expected}"
        );
    }
    let window = &summary["window"];
    assert!(window["from"].as_f64().unwrap() <= window["to"].as_f64().unwrap());
}

// A Permanent Materialization Refusal Leaves Evidence And The Execution Unsealed.
pub(crate) async fn oversized_report_remains_unsealed() {
    let root = tempdir().unwrap();
    let release = root.path().join("release");
    let mut constrained =
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({
                    "spectrum_rows": OVERSIZED_SPECTRUM_ROWS,
                    "block_execution": true,
                    "execution_release": release,
                }),
            )
            .unwrap()]);
    constrain_report_memory(&mut constrained);
    let (mut runtime, handle) = started(constrained).await;
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(
                    Identifier::new("spectrum.oversized").unwrap(),
                    "Oversized",
                    None,
                ),
                cases: materialize_cases("fixture.checks", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    let execution =
        handle.watch_state().latest().state().cycles[&cycle_id].execution_ids[0].clone();

    let failure = tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            if let Err(error) = handle.open_execution_report(&execution).await {
                let message = error.to_string();
                assert!(
                    message.contains(EXHAUSTED) || message.contains("exhaust"),
                    "{message}"
                );
                break message;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("oversized materialization returns an explicit failure");
    assert!(!failure.is_empty());
    std::fs::write(&release, b"").unwrap();
    let cycle = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap();
    assert!(cycle.saved.is_none());
    assert!(
        cycle
            .test_executions
            .iter()
            .all(|execution| execution.saved.is_none())
    );
    assert!(handle.open_execution_report(&execution).await.is_err());
    drop(handle);
    assert!(
        runtime.shutdown().await.is_err(),
        "shutdown reports unfinished materialization"
    );
}

// A Refused Pool Never Publishes A Degraded Cycle Report.
pub(crate) async fn pool_refusal_settles() {
    let root = tempdir().unwrap();
    let mut refusing =
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "spectrum_rows": POOLED_SPECTRUM_ROWS }),
            )
            .unwrap()]);
    constrain_report_memory(&mut refusing);
    let (mut runtime, handle) = started(refusing).await;
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(
                    Identifier::new("spectrum.refused").unwrap(),
                    "Refused",
                    None,
                ),
                cases: vec![materialize_cases("fixture.checks", &suite).remove(0); POOLED_MEMBERS],
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    // The full pool may refuse this polling read too while finalization holds it.
    let settled = tokio::time::timeout(Duration::from_secs(120), async {
        loop {
            if let Ok(Some(receipt)) = handle.run_request(&receipt.command_id).await
                && receipt.summary.phase.terminal()
            {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("a cycle the pool refuses settles its run in bounded time");
    assert_ne!(
        settled.summary.phase,
        RunPhase::Completed,
        "permanent refusal cannot complete a report"
    );
    let cycle = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap();
    assert!(cycle.saved.is_none(), "refused report remains unsealed");
    assert!(handle.open_cycle_report(&cycle_id).await.is_err());
    drop(handle);
    assert!(runtime.shutdown().await.is_err());
}

// Report Output Reads Use The Same Configured Deadline As Ordinary Reads.
pub(crate) async fn slow_output_read_uses_shared_deadline() {
    let root = tempdir().unwrap();
    let mut slow =
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "log_lines": LOGGED_LINES }),
            )
            .unwrap()]);
    slow.storage.interactive.timeout = SHARED_QUERY_BUDGET;
    slow.storage.finalization_read_stall = OUTPUT_READ_STALL;
    let (mut runtime, handle) = started(slow).await;
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let case = materialize_cases("fixture.checks", &suite).remove(0);
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite,
                metadata: Metadata::new(Identifier::new("logged.slow").unwrap(), "Slow", None),
                cases: vec![case],
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    let settled = tokio::time::timeout(Duration::from_secs(120), async {
        loop {
            let receipt = handle
                .run_request(&receipt.command_id)
                .await
                .unwrap()
                .unwrap();
            if receipt.summary.phase.terminal() {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("a stalled output read settles its run");
    assert_eq!(
        settled.summary.phase,
        RunPhase::Completed,
        "{:?}",
        settled.interruption
    );
    let cycle = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap();
    assert!(cycle.saved.is_some(), "the cycle report is sealed");
    assert!(
        cycle
            .test_executions
            .iter()
            .all(|execution| execution.saved.is_some()),
        "the logged execution seals although its output read stalled"
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// A replay page read the storage backend refuses at its deadline is read again
// from the position the replay kept, so the run is still handed every pulse
// row exactly once and completes.
pub(crate) async fn replay_deadline_retries() {
    let root = tempdir().unwrap();
    let mut refusing =
        sampling_config(root.path(), serde_json::json!({ "pulse_rows": PULSE_ROWS }));
    refusing.storage.replay_read_deadlines = FORCED_REPLAY_DEADLINES;
    let (mut runtime, handle) = started(refusing).await;
    run_case(&handle, "pulse.deadline").await;
    drop(handle);
    runtime.shutdown().await.unwrap();
}

fn constrain_report_memory(config: &mut dokime_app::RuntimeConfig) {
    let memory = std::num::NonZeroUsize::new(32 * 1024 * 1024).unwrap();
    config.storage.budget.read_memory_bytes = memory;
    config.storage.interactive.memory_bytes = memory;
    config.storage.page_bytes = std::num::NonZeroUsize::new(64 * 1024).unwrap();
    config.storage.response.batch_bytes = std::num::NonZeroUsize::new(512 * 1024).unwrap();
}
