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

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn preview_start_plans() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = fixture_suite(&["passed"]);
    runtime
        .set_suites(
            "fixture",
            dokime_app::definition::SuiteSet::new([dokime::domains::TestSuiteDefinition::new(
                "fixture.registration",
                suite.clone(),
            )
            .unwrap()]),
        )
        .await
        .unwrap();
    let mut live = runtime.watch_state();
    let initial_revision = live.latest().revision();

    let preview = runtime
        .preview_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("cycle.preview", "Preview Cycle"),
                cases: materialize_cases("fixture.registration", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await;
    let start = runtime
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("cycle.start", "Start Cycle"),
                cases: materialize_cases("fixture.registration", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await;
    live.next().await.unwrap();
    let cycles = runtime.cycles().await;
    let revision = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds")
        .revision;
    let query = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::Cycle,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();

    assert!(preview.is_ok());
    assert!(start.is_ok());
    assert_eq!(cycles.len(), 2);
    assert!(revision >= initial_revision + 2);
    assert!(cycles.iter().any(|cycle| {
        cycle.lifecycle == CycleStatus::Previewed
            && cycle.cases().count() == 1
            && cycle.test_executions.len() == 1
            && cycle.test_executions[0].status == ExecutionStatus::Skipped
            && cycle.test_executions[0].started_at.is_none()
            && cycle.test_executions[0].finished_at.is_none()
    }));
    assert!(cycles.iter().any(|cycle| {
        cycle.lifecycle == CycleStatus::Ready
            && cycle.started_at.is_none()
            && cycle.cases().count() == 1
            && cycle.test_executions.len() == 1
            && cycle.test_executions[0].status == ExecutionStatus::Skipped
            && cycle.test_executions[0].started_at.is_none()
            && cycle.test_executions[0].finished_at.is_none()
    }));
    let latest = live.latest().revision();
    assert!(latest >= initial_revision + 2);
    let RuntimeViewPayload::Cycles { frame } = query.payload else {
        panic!("expected cycle query frame");
    };
    assert_eq!(frame.count, 2);
}

/* === Tests === */
