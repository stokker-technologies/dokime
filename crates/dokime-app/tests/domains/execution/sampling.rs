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
use std::time::{Duration, Instant};

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
use dokime_app::transactions::CycleDefinition;

/* === Constants === */
/// Rows each case pushes onto its own stream: many storage pages per case.
const PULSE_ROWS: u32 = 2_000;

/// Cases run back to back in one store. Before the pager kept its cursor in
/// the index range, every page of a later case rescanned the earlier cases'
/// rows and the terminal capture exhausted its query time by the third case.
const CASES: usize = 6;

/// Rows one large case pushes: more than the report could hold as samples
/// under its 32 MiB materialization budget.
const LARGE_PULSE_ROWS: u32 = 20_000;

/// Points the fixture's pulse plot declares it shows.
const PULSE_DISPLAY_POINTS: u64 = 400;

/// Rows of the stream the fixture's frequency plot transforms whole: many
/// storage pages, yet a fraction of the 32 MiB report budget.
const SPECTRUM_ROWS: u32 = 1_024;

/// Rows of the whole-capture stream that alone exceed the 32 MiB report budget.
const OVERSIZED_SPECTRUM_ROWS: u32 = 8_192;

/// How a report too large for its whole budget names its permanent cause.
const UNSEALED: &str = "final execution report was not sealed";
const EXHAUSTED: &str = "final report materialization memory";

/// A storage read pool that admits running the cycle but never holds the cycle
/// report retaining every member, as the 256 MiB production pool cannot
/// hold a large enough cycle.
const REFUSING_POOL_BYTES: usize = 128 * 1024 * 1024;

/// Members of the cycle the refusing pool cannot hold at once.
const POOLED_MEMBERS: usize = 9;

/// Rows of each member's whole-capture stream: within one report's budget.
const POOLED_SPECTRUM_ROWS: u32 = 1_536;

/// How finalization names a cycle report it never sealed.
const CYCLE_UNSEALED: &str = "final cycle report was not sealed";

/// How finalization names a report the read pool refused on every attempt.
const POOL_REFUSED: &str = "the report does not fit the storage read pool";

/// An interactive query budget well below one stalled finalization read.
const TIGHT_QUERY_BUDGET: Duration = Duration::from_millis(200);

/// How long each native-output read finalization performs stalls within its
/// budget: past the interactive budget, well within finalization's own.
const OUTPUT_READ_STALL: Duration = Duration::from_secs(1);

/// How much slower the last case may settle than the first. Work that grows
/// with the store's history settles the sixth case several times slower.
const GROWTH: u32 = 3;

/// Cases of one cycle whose terminal captures seal concurrently.
const LOGGED_CASES: usize = 5;

/// Lines each logged case writes into its sidecar stream log.
const LOGGED_LINES: u32 = 3;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn run_case(handle: &RuntimeHandle, label: &str) -> Duration {
    run_case_execution(handle, label, Duration::from_secs(60))
        .await
        .0
}

async fn run_case_execution(
    handle: &RuntimeHandle,
    label: &str,
    bound: Duration,
) -> (Duration, dokime::core::TestExecutionId) {
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
    let started = Instant::now();
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
                break receipt.summary.phase;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("{label} records its terminal provider state"));
    let settled = started.elapsed();
    assert_eq!(phase, RunPhase::Completed, "{label}");
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
    (settled, execution.test_execution_id)
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
    let mut settled = Vec::with_capacity(CASES);
    for case in 0..CASES {
        settled.push(run_case(&handle, &format!("pulse.case{case}")).await);
    }
    let first = settled[0];
    let last = settled[CASES - 1];
    assert!(
        last <= first * GROWTH,
        "the last case settled in {last:?} against the first case's {first:?}: \
         replay and terminal capture grew with the store's history ({settled:?})"
    );
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
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "pulse_rows": LARGE_PULSE_ROWS }),
            )
            .unwrap()]),
    )
    .await;
    let (_, execution_id) =
        run_case_execution(&handle, "pulse.large", Duration::from_secs(300)).await;
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
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({
                    "pulse_rows": LARGE_PULSE_ROWS,
                    "spectrum_rows": SPECTRUM_ROWS,
                }),
            )
            .unwrap()]),
    )
    .await;
    let (_, execution_id) =
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

// a report whose kept rows exceed the whole report budget settles its run
// Failed with that permanent cause instead of retrying forever.
pub(crate) async fn oversized_report_settles() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "spectrum_rows": OVERSIZED_SPECTRUM_ROWS }),
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
    let settled = tokio::time::timeout(Duration::from_secs(300), async {
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
    .expect("an oversized report settles its run instead of retrying");
    assert_eq!(settled.summary.phase, RunPhase::Failed);
    assert!(
        settled
            .interruption
            .as_deref()
            .is_some_and(|cause| cause.starts_with(UNSEALED) && cause.contains(EXHAUSTED)),
        "the receipt names the exhausted report budget: {:?}",
        settled.interruption
    );
    let cycle = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap();
    assert_eq!(cycle.lifecycle, CycleStatus::Failed);
    assert!(
        cycle.saved.is_none(),
        "an unproduced report is never sealed"
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// A cycle whose members together exceed the storage read pool settles Failed
// in bounded time, the pool named as the cause, instead of retrying its report
// while the cycle stays Running.
pub(crate) async fn pool_refusal_settles() {
    let root = tempdir().unwrap();
    let mut refusing =
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "spectrum_rows": POOLED_SPECTRUM_ROWS }),
            )
            .unwrap()]);
    refusing.storage_reads.pool_bytes = std::num::NonZeroUsize::new(REFUSING_POOL_BYTES).unwrap();
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
    assert_eq!(settled.summary.phase, RunPhase::Failed);
    assert!(
        settled
            .interruption
            .as_deref()
            .is_some_and(|cause| cause.starts_with(CYCLE_UNSEALED) && cause.contains(POOL_REFUSED)),
        "the receipt names the refusing pool: {:?}",
        settled.interruption
    );
    let cycle = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap();
    assert_eq!(cycle.lifecycle, CycleStatus::Failed);
    assert!(
        cycle.saved.is_none(),
        "a refused cycle report is never sealed"
    );
    assert_eq!(cycle.test_executions.len(), POOLED_MEMBERS);
    let evidence = handle.evidence_snapshot().await.unwrap();
    let recorded = evidence
        .records
        .iter()
        .find(|record| record.record_type.as_str() == "test.cycle.finalization.failed")
        .expect("the unsealed cycle records its cause");
    assert!(
        recorded
            .message
            .as_deref()
            .is_some_and(|message| message.contains(POOL_REFUSED)),
        "the cycle evidence names the refusing pool: {recorded:?}"
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}

// Finalization reads native outputs under its own budget: a logged case whose
// every output read stalls past the interactive budget still seals, where a
// read under the interactive deadline would have exhausted it.
pub(crate) async fn slow_output_read_seals() {
    let root = tempdir().unwrap();
    let mut slow =
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "log_lines": LOGGED_LINES }),
            )
            .unwrap()]);
    slow.storage_reads.execution.timeout = TIGHT_QUERY_BUDGET;
    slow.storage_reads.finalization_read_stall = OUTPUT_READ_STALL;
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
