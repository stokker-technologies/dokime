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
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::support;
use crate::support::wire::*;
use dokime::client::contracts;
use dokime::domain::model;
use dokime_server::grpc::{Config, Server};

/* === Constants === */

const CRASH_ROOT: &str = "DOKIME_TEST_RUN_REQUEST_CRASH_ROOT";
const READY: &str = "crash-fixture-ready ";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Child(std::process::Child);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn config() -> Config {
    keyed()
}

#[tokio::test]
async fn crashed_run_replay() {
    if let Some(root) = std::env::var_os(CRASH_ROOT) {
        let root = std::path::PathBuf::from(root);
        let gate = Arc::new(Gate::default());
        let owner = fixture(&root, gate.clone()).await;
        let server = Server::start(owner.handle(), config()).await.unwrap();
        let client = WireClient::connect(server.local_endpoint()).await.unwrap();
        let cycle = support::planned_cycle(&client, authenticated(create()), selection_plan(&owner)).await.unwrap();
        let executions = client
            .unary::<contracts::execution::ListExecutions>(authenticated(contracts::execution::ListExecutions {
                cycle_id: cycle.clone(),
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        let execution = executions.executions[0].execution_id.clone();
        let admission = accepted(
            client
                .unary::<contracts::execution::StartExecution>(authenticated(contracts::execution::StartExecution {
                    execution_id: execution.clone(),
                    mode: model::execution::wire::ExecutionMode::Automatic,
                }))
                .await
                .unwrap()
                .into_inner(),
        );
        gate.started.notified().await;
        let command = model::identity::CommandId::from_uuid(admission.command_id.clone()).unwrap();
        let receipt = owner.handle().run_request(&command).await.unwrap().unwrap();
        assert_eq!(receipt.summary.phase, model::execution::request::RunPhase::Running);
        // Crash recovery applies to saved work; ordinary admission may still be buffered.
        owner.handle().storage().flush().await.unwrap();
        owner.handle().storage().flush_recovery().await.unwrap();
        let ready = serde_json::to_string(&(admission.command_id, cycle, execution)).unwrap();
        println!("{READY}{ready}");
        std::future::pending::<()>().await;
        unreachable!();
    }
    let root = tempfile::tempdir().unwrap();
    let test = format!("{}::crashed_run_replay", module_path!().split_once("::").unwrap().1);
    let mut child = Child(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &test, "--nocapture"])
            .env(CRASH_ROOT, root.path())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap(),
    );

    // The Child Reports Readiness On Stdout Once Its Run Is Saved
    let stdout = child.0.stdout.take().unwrap();
    let (request_id, cycle_id, execution_id): (String, String, String) = tokio::task::spawn_blocking(move || {
        use std::io::BufRead;
        let line = std::io::BufReader::new(stdout)
            .lines()
            .map_while(Result::ok)
            .find_map(|line| line.strip_prefix(READY).map(str::to_owned))
            .expect("crash fixture exited before readiness");
        serde_json::from_str(&line).unwrap()
    })
    .await
    .unwrap();
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let mut owner = runtime(root.path()).await;
    let server = Server::start(owner.handle(), config()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let command = model::identity::CommandId::from_uuid(request_id.clone()).unwrap();
    let receipt = owner.handle().run_request(&command).await.unwrap().unwrap();
    assert_eq!(receipt.summary.phase, model::execution::request::RunPhase::Interrupted);
    assert_eq!(receipt.summary.prepared, 1);
    assert_eq!(receipt.summary.finished, 0);
    assert!(receipt.summary.finished_at.is_some());
    assert!(receipt.interruption.as_ref().is_some_and(|reason| !reason.is_empty()));
    let replay = accepted(
        client
            .unary::<contracts::execution::StartExecution>(authenticated(contracts::execution::StartExecution {
                execution_id: execution_id.clone(),
                mode: model::execution::wire::ExecutionMode::Automatic,
            }))
            .await
            .unwrap()
            .into_inner(),
    );
    assert_eq!(replay.command_id, request_id);
    let restored = owner.handle().watch_state().latest();
    assert!(
        restored.state().cycles.values().all(|cycle| cycle.phase == model::execution::result::CyclePhase::Sealed && cycle.saved.is_some()),
        "recovery seals previous-session cycles before serving admission"
    );
    let cycle_id = dokime::TestCycleId::from_uuid(cycle_id).unwrap();
    let cycle_owner =
        dokime::storage::objects::cycle::read(owner.handle().storage(), &cycle_id, dokime::harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    let cycle = &cycle_owner.value;
    assert_eq!(cycle.phase, dokime::domain::model::execution::result::CyclePhase::Sealed);
    assert!(cycle.saved.is_some());
    assert_eq!(cycle.execution_ids.len(), 1);
    let id = dokime::TestExecutionId::from_uuid(execution_id).unwrap();
    assert_eq!(cycle.execution_ids[0], id);
    let execution_owner =
        dokime::storage::objects::execution::read(owner.handle().storage(), &id, dokime::harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    let execution = &execution_owner.value;
    assert_eq!(execution.status, dokime::domain::model::execution::result::ExecutionStatus::Interrupted);
    assert_eq!(execution.test_execution_id, receipt.last_claim.as_ref().unwrap().execution_id);
    assert!(execution.finished_at.is_some());
    assert_eq!(restored.state().run_requests.len(), 1);
    let recovered = restored.state().run_requests.values().next().unwrap();
    assert_eq!(recovered.summary.phase, dokime::domain::model::execution::request::RunPhase::Interrupted);
    assert!(recovered.pending.is_empty());
    assert!(execution.saved.is_some());
    let repeated = owner.handle().run_request(&command).await.unwrap().unwrap();
    assert_eq!(&*repeated, &*receipt);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
