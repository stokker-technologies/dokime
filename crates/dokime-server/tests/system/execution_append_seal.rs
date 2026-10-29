/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Append And Seal At Full Scale: A 1,003-Member Cycle Appends In One
// Request And Seals; Integration Keeps The 100-Member Guard Of The Same Path.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::support::{self, wire::*};
use dokime::client::contracts;
use dokime::domain::model;
use dokime::storage::sqlite::{StorageCounters, counters};
use dokime_server::grpc::Server;

/* === Constants === */

const SMALL: usize = 10;
const LARGE: usize = 1_003;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Storage Work Of One Stage, As Deterministic Counter Deltas.
#[derive(Debug, Clone, Copy)]
struct Work {
    transactions: u64,
    batches: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Work {
    fn between(before: StorageCounters, after: StorageCounters) -> Self {
        Self {
            transactions: after.committed_transactions - before.committed_transactions,
            batches: after.data_batches - before.data_batches,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn create(wire: &WireClient, label: &str) -> String {
    wire.unary::<contracts::execution::CreateCycle>(auth(
        contracts::execution::CreateCycle {
            metadata: Some(model::identity::wire::Metadata {
                label: label.into(),
                ..Default::default()
            }),
            ..Default::default()
        },
        TOKEN,
    ))
    .await
    .unwrap()
    .into_inner()
    .cycle_id
}

async fn summary(wire: &WireClient, cycle_id: &str) -> model::execution::wire::CycleSummary {
    wire.unary::<contracts::execution::GetCycle>(auth(contracts::execution::GetCycle { cycle_id: cycle_id.into() }, TOKEN))
        .await
        .unwrap()
        .into_inner()
}

// Appends `members` attempts of one case in one request, then seals the cycle;
// returns the storage work of the append and of the seal.
async fn append_and_seal(handle: &dokime::RuntimeHandle, wire: &WireClient, case_id: &str, members: usize) -> (Work, Work) {
    let cycle_id = create(wire, &format!("Append {members}")).await;
    let request = contracts::execution::AppendExecutions {
        cycle_id: cycle_id.clone(),
        executions: (0..members)
            .map(|_| contracts::execution::ExecutionBinding {
                execution_id: dokime::model::identity::new_id::<dokime::TestExecutionId>().to_string(),
                case_id: case_id.into(),
            })
            .collect(),
    };
    let before = counters();
    let added = wire.unary::<contracts::execution::AppendExecutions>(auth(request, TOKEN)).await.unwrap().into_inner();
    let append = Work::between(before, counters());
    assert_eq!(added.execution_ids.len(), members);
    let cycle = dokime::TestCycleId::from_uuid(cycle_id.clone()).unwrap();
    assert_eq!(handle.watch_state().latest().state().cycle_view(&cycle).unwrap().test_executions.len(), members);

    let before = counters();
    wire.unary::<contracts::execution::SealCycle>(auth(
        contracts::execution::SealCycle {
            cycle_id: cycle_id.clone(),
            context: None,
        },
        TOKEN,
    ))
    .await
    .unwrap();
    let sealed = support::on_state_change(handle, async || {
        let summary = summary(wire, &cycle_id).await;
        (summary.phase == dokime::domain::model::execution::result::CyclePhase::Sealed).then_some(summary)
    })
    .await;
    let seal = Work::between(before, counters());
    assert!(sealed.sealed);
    assert_eq!(sealed.execution_count, members as u64);
    (append, seal)
}

/* === Tests === */

#[tokio::test]
async fn execution_append_seal() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let suite = support::fixture_suite(&["passed"]);
    support::register_fixture_execution(&handle, suite.clone()).await;
    let server = Server::start(handle.clone(), keyed()).await.unwrap();
    let wire = WireClient::connect(server.local_endpoint()).await.unwrap();
    let generated = wire
        .unary::<contracts::planning::MaterializeTemplate>(auth(
            contracts::planning::MaterializeTemplate {
                selection: Some(model::planning::wire::RegisteredSelection {
                    suite_id: suite.metadata.identifier.to_string(),
                    template_id: suite.test_case_templates[0].test_case_template_id.to_string(),
                    ..Default::default()
                }),
            },
            TOKEN,
        ))
        .await
        .unwrap()
        .into_inner();
    let case_id = generated.cases[0].case_id.clone();

    // The Reference Stage And The Full-Scale Stage Run The Same Path On One Fixture.
    let (small_append, small_seal) = append_and_seal(&handle, &wire, &case_id, SMALL).await;
    let (large_append, large_seal) = append_and_seal(&handle, &wire, &case_id, LARGE).await;
    // Linear Scaling: Each Member Of The Full-Scale Stage Costs No More Storage Work Than A
    // Member Of The Reference Stage (Measured 2026-10-27: Append 45 -> 4,017 Transactions,
    // Seal 234 -> 17,633 Transactions For 10 -> 1,003 Members).
    for (stage, small, large) in [("append", small_append, large_append), ("seal", small_seal, large_seal)] {
        assert!(large.transactions * SMALL as u64 <= small.transactions * LARGE as u64, "{stage}: {small:?} -> {large:?}");
        assert!(large.batches * SMALL as u64 <= small.batches * LARGE as u64, "{stage}: {small:?} -> {large:?}");
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
