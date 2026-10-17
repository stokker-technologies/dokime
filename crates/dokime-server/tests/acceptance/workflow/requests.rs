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
use dokime_domain::model;
use crate::support;
use crate::support::wire::*;
use dokime_client::contracts;
use dokime_server::grpc::{
    Config, Server,
    access::{BearerGrant, Role},
};

/* === Constants === */
const READER: &str = "owner-reader-token-012345678901234567890123456";
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
    Config::local()
        .with_bearer_auth(vec![
            grant(Role::Control),
            BearerGrant::new("fixture".into(), Role::Read, READER).unwrap(),
        ])
        .unwrap()
}

/* === Tests === */
#[tokio::test]
async fn crashed_run_replay() {
    if let Some(root) = std::env::var_os(CRASH_ROOT) {
        let root = std::path::PathBuf::from(root);
        let gate = Arc::new(Gate::default());
        let owner = fixture(&root, gate.clone()).await;
        let server = Server::start(owner.handle(), config()).await.unwrap();
        let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
        let cycle = support::planned_cycle(&client, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();
        let cases = client
            .unary::<contracts::execution::ListCases>(authenticated(
                contracts::execution::ListCases {
                    cycle_id: cycle.clone(),
                    ..Default::default()
                },
            ))
            .await
            .unwrap()
            .into_inner();
        let case = cases.cases[0].case_id.clone();
        let admission = accepted(
            client
                .unary::<contracts::execution::RunCase>(authenticated(
                    contracts::execution::RunCase {
                        cycle_id: cycle.clone(),
                        case_id: case.clone(),
                        context: None,
                        idempotency_key: Some("crash-operation".into()),
                    },
                ))
                .await
                .unwrap()
                .into_inner(),
        );
        gate.started.notified().await;
        let receipt = lookup(&mut client, &admission.command_id, READER)
            .await
            .unwrap();
        assert_eq!(
            receipt.phase,
            model::execution::wire::RunRequestPhase::Running
        );
        // Crash recovery applies to saved work; ordinary admission may still be buffered.
        owner.handle().storage().flush().await.unwrap();
        owner.handle().storage().flush_recovery().await.unwrap();
        let ready = serde_json::to_string(&(admission.command_id, cycle, case)).unwrap();
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
    let (request_id, cycle_id, case_id): (String, String, String) =
        tokio::task::spawn_blocking(move || {
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
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let receipt = lookup(&mut client, &request_id, READER).await.unwrap();
    assert_eq!(
        receipt.phase,
        model::execution::wire::RunRequestPhase::Interrupted
    );
    assert_eq!(receipt.prepared_count, 1);
    assert_eq!(receipt.finished_count, 0);
    assert!(receipt.finished_at.is_some());
    assert!(
        receipt
            .interruption
            .as_ref()
            .is_some_and(|reason| !reason.is_empty())
    );
    let replay = accepted(
        client
            .unary::<contracts::execution::RunCase>(authenticated(
                contracts::execution::RunCase {
                    cycle_id,
                    case_id,
                    context: None,
                    idempotency_key: Some("crash-operation".into()),
                },
            ))
            .await
            .unwrap()
            .into_inner(),
    );
    assert_eq!(replay.command_id, request_id);
    let restored = owner.handle().watch_state().latest();
    assert_eq!(restored.state().cycles.len(), 1);
    let cycle = restored.state().cycle_views().next().unwrap();
    assert_eq!(cycle.test_executions.len(), 1);
    let execution = &cycle.test_executions[0];
    assert_eq!(
        execution.status,
        dokime_runtime::execution::ExecutionStatus::Interrupted
    );
    assert_eq!(
        execution.test_execution_id.as_str(),
        receipt.last_claim.as_ref().unwrap().execution_id
    );
    assert!(execution.finished_at.is_some());
    assert_eq!(restored.state().run_requests.len(), 1);
    let recovered = restored.state().run_requests.values().next().unwrap();
    assert_eq!(
        recovered.summary.phase,
        dokime_runtime::execution::request::RunPhase::Interrupted
    );
    assert!(recovered.pending.is_empty());
    assert_eq!(
        lookup(&mut client, &request_id, READER).await.unwrap(),
        receipt
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
