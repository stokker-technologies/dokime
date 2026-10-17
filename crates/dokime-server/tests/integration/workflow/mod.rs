/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod pagination;

pub(crate) mod search;

pub(crate) mod files;

pub(crate) mod controls;

pub(crate) mod inspection;

pub(crate) mod requests;

pub(crate) mod steps;

pub(crate) mod documents_execution;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use prost::Message;
use tokio::sync::Notify;
// Local Imports
use dokime_domain::model;
use super::*;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::execution::CycleView;
use dokime_runtime::transport::ProviderExecutionFailure;
use dokime_runtime::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Progress;

#[derive(Default)]
struct BlockingStop {
    started: Notify,
    release: Notify,
}

struct Gated {
    root: tempfile::TempDir,
    gate: Arc<Gate>,
    owner: Runtime,
    service: dokime_server::grpc::Service,
    server: Server,
    execution: WireClient,
    cycle: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionFixture for Progress {
    fn run_cycle<'a>(
        &'a self,
        _context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move { Ok(support::fixture_provider_outcome(cycle, true)) })
    }
}

impl ExecutionFixture for BlockingStop {
    fn run_cycle<'a>(
        &'a self,
        _context: ExecutionFixtureContext,
        _cycle: &'a CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            self.started.notify_one();
            self.release.notified().await;
            Err(ProviderExecutionFailure::new(
                "blocking_stop",
                "blocking stop executor released",
            ))
        })
    }
}

impl Gated {
    async fn open() -> Self {
        Self::bounded(None).await
    }

    async fn open_bounded(max_message_bytes: usize) -> Self {
        Self::bounded(Some(max_message_bytes)).await
    }

    async fn bounded(max_message_bytes: Option<usize>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let gate = Arc::new(Gate::default());
        let owner = fixture(root.path(), gate.clone()).await;
        let mut config = Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap();
        if let Some(bytes) = max_message_bytes {
            config = config.with_max_message_bytes(bytes);
        }
        let service = dokime_server::grpc::Service::new(owner.handle(), config)
            .await
            .unwrap();
        let server = service.listen().await.unwrap();
        let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
        let cycle =
            support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
                .await
                .unwrap();
        Self {
            root,
            gate,
            owner,
            service,
            server,
            execution,
            cycle,
        }
    }
}

/* === Public Functions === */
/* === Private Functions === */
#[tokio::test]
async fn registered_create_definitions() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap(),
    )
    .await
    .unwrap();
    let connection = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let catalog = WireClient::new(connection.clone());
    let execution = WireClient::new(connection.clone());
    let observation = WireClient::new(connection);
    let suites = catalog
        .unary::<contracts::catalog::ListSuites>(authenticated(
            contracts::catalog::ListSuites::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(suites.suites.len(), 1);
    let suite = catalog
        .unary::<contracts::catalog::GetSuite>(authenticated(contracts::catalog::GetSuite {
            suite_id: suites.suites[0].suite_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(suite.templates.len(), 1);
    assert_eq!(
        suite.templates[0].parameters[0].value_type,
        model::definition::wire::ValueType::Text
    );
    assert!(matches!(
        suite.templates[0].parameters[0]
            .expression
            .as_ref()
            .unwrap()
            .nodes[0]
            .kind,
        Some(model::catalog::wire::ParameterNodeKind::Literal(_))
    ));
    let mut stream = observation
        .download::<contracts::observation::WatchRuntime>(authenticated(
            contracts::observation::WatchRuntime::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    let initial = stream.message().await.unwrap().unwrap();
    assert!(initial.resynchronized);
    let id = support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
        .await
        .unwrap();
    let planned = execution
        .unary::<contracts::execution::GetCycle>(authenticated(
            contracts::execution::GetCycle {
                cycle_id: id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(planned.status, model::execution::wire::CycleStatus::Ready);
    assert_eq!(planned.case_count, 1);
    assert_eq!(planned.execution_count, 1);
    let cases = execution
        .unary::<contracts::execution::ListCases>(authenticated(
            contracts::execution::ListCases {
                cycle_id: id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .cases;
    assert_eq!(cases[0].template_id, suite.templates[0].template_id);
    let admitted = execution
        .unary::<contracts::execution::RunCase>(authenticated(
            contracts::execution::RunCase {
                cycle_id: id.clone(),
                case_id: cases[0].case_id.clone(),
                context: None,
                idempotency_key: None,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        admitted.outcome,
        Some(model::execution::wire::RunReplyOutcome::Accepted(_))
    ));
    gate.started.notified().await;
    execution
        .unary::<contracts::execution::PauseCycle>(authenticated(contracts::execution::PauseCycle { cycle_id: id.clone(), context: None }))
        .await
        .unwrap();
    execution
        .unary::<contracts::execution::ResumeCycle>(authenticated(contracts::execution::ResumeCycle { cycle_id: id.clone(), context: None }))
        .await
        .unwrap();
    // Disconnect observation while admitted work is waiting on the fixture.
    drop(stream);
    gate.release.notify_one();
    let mut stream = observation
        .download::<contracts::observation::WatchRuntime>(authenticated(
            contracts::observation::WatchRuntime {
                cursor: initial.cursor.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(stream.message().await.unwrap().unwrap().resynchronized);
    {
        loop {
            let cycle = execution
                .unary::<contracts::execution::GetCycle>(authenticated(
                    contracts::execution::GetCycle {
                        cycle_id: id.clone(),
                    },
                ))
                .await
                .unwrap()
                .into_inner();
            if cycle.status == model::execution::wire::CycleStatus::Completed {
                assert_eq!(cycle.execution_count, 1);
                let attempts = execution
                    .unary::<contracts::execution::ListExecutions>(authenticated(
                        contracts::execution::ListExecutions {
                            cycle_id: id.clone(),
                            ..Default::default()
                        },
                    ))
                    .await
                    .unwrap()
                    .into_inner();
                assert_eq!(
                    attempts.executions[0].status,
                    model::execution::wire::ExecutionStatus::Passed
                );
                break;
            }
            stream.message().await.unwrap().unwrap();
        }
    }
    let list = execution
        .unary::<contracts::execution::ListCycles>(authenticated(
            contracts::execution::ListCycles::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(list.cycles.len(), 1);
    let other = support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
        .await
        .unwrap();
    execution
        .unary::<contracts::execution::StopCycle>(authenticated(contracts::execution::StopCycle { cycle_id: other.clone(), context: None }))
        .await
        .unwrap();
    assert_eq!(
        execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycle { cycle_id: other }
            ))
            .await
            .unwrap()
            .into_inner()
            .status,
        model::execution::wire::CycleStatus::Stopped
    );
    server.shutdown().await.unwrap();
    {
        loop {
            match stream.message().await {
                Ok(Some(_)) => {}
                Err(status) => {
                    assert_eq!(status.code(), Code::Unavailable);
                    break;
                }
                Ok(None) => panic!("listener shutdown must not report successful completion"),
            }
        }
    }
    assert!(!owner.handle().is_shutting_down());
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn stopping_running_unwind() {
    let root = tempfile::tempdir().unwrap();
    let blocker = Arc::new(BlockingStop::default());
    let mut owner = fixture_with_provider(root.path(), blocker.clone()).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id =
        support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();
    let reply = execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle_id)))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        reply.outcome,
        Some(model::execution::wire::RunReplyOutcome::Accepted(_))
    ));
    blocker.started.notified().await;

    execution
        .unary::<contracts::execution::StopCycle>(authenticated(contracts::execution::StopCycle { cycle_id: cycle_id.clone(), context: None }))
        .await
        .unwrap();
    let stopping = execution
        .unary::<contracts::execution::GetCycle>(authenticated(
            contracts::execution::GetCycle {
                cycle_id: cycle_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(stopping.status, model::execution::wire::CycleStatus::Stopping);

    blocker.release.notify_one();
    let stopped = support::on_state_change(&owner.handle(), async || {
        let summary = execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycle {
                    cycle_id: cycle_id.clone(),
                },
            ))
            .await
            .unwrap()
            .into_inner();
        if summary.status == model::execution::wire::CycleStatus::Stopped {
            return Some(summary);
        }
        None
    })
    .await;
    assert_eq!(stopped.execution_count, 1);
    let executions = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutions {
                cycle_id,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(executions.executions.len(), 1);
    assert_eq!(
        executions.executions[0].status,
        model::execution::wire::ExecutionStatus::Cancelled
    );

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn runtime_shell_cursor() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default())).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let connection = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let execution = WireClient::new(connection.clone());
    let observation = WireClient::new(connection);
    let first_cycle =
        support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();
    let second_cycle =
        support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();

    let first = observation
        .unary::<contracts::observation::GetRuntimeShell>(authenticated(
            contracts::observation::GetRuntimeShell {
                limit: 1,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.cycle_summaries.len(), 1);
    let next_cycle = first
        .next_after_cycle_id
        .clone()
        .expect("two cycles require a continuation");
    let cursor = first.observed_cursor.clone().expect("shell cursor");

    let second = observation
        .unary::<contracts::observation::GetRuntimeShell>(authenticated(
            contracts::observation::GetRuntimeShell {
                limit: 1,
                after_cycle_id: next_cycle.clone(),
                cursor: Some(cursor.clone()),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(second.cycle_summaries.len(), 1);
    assert_ne!(
        second.cycle_summaries[0].cycle_id,
        first.cycle_summaries[0].cycle_id
    );
    assert!(
        second.cycle_summaries[0].cycle_id == first_cycle
            || second.cycle_summaries[0].cycle_id == second_cycle
    );
    assert!(second.next_after_cycle_id.is_none());

    let future_cursor = model::execution::wire_cursor::Cursor {
        position: cursor.position + 1,
        ..cursor.clone()
    };
    assert_eq!(
        observation
            .unary::<contracts::observation::GetRuntimeShell>(authenticated(
                contracts::observation::GetRuntimeShell {
                    limit: 1,
                    after_cycle_id: next_cycle.clone(),
                    cursor: Some(future_cursor),
                }
            ))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::OutOfRange
    );
    let mut wrong_epoch = cursor.clone();
    wrong_epoch.epoch = "other-session".into();
    assert_eq!(
        observation
            .unary::<contracts::observation::GetRuntimeShell>(authenticated(
                contracts::observation::GetRuntimeShell {
                    limit: 1,
                    after_cycle_id: next_cycle.clone(),
                    cursor: Some(wrong_epoch),
                }
            ))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::FailedPrecondition
    );

    execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create()))
        .await
        .unwrap();
    assert_eq!(
        observation
            .unary::<contracts::observation::GetRuntimeShell>(authenticated(
                contracts::observation::GetRuntimeShell {
                    limit: 1,
                    after_cycle_id: next_cycle,
                    cursor: Some(cursor),
                }
            ))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::Aborted
    );

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn checkpoint_telemetry_queries() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture_with_provider(root.path(), Arc::new(Progress)).await;
    support::register_fixture_progress_execution(
        &owner.handle(),
        support::fixture_suite(&["passed"]),
    )
    .await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let connection = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let execution = WireClient::new(connection.clone());
    let history = WireClient::new(connection);
    let cycle_id =
        support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle_id)))
        .await
        .unwrap();
    let checkpoint = support::on_state_change(&owner.handle(), async || {
        let cycle = execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycle {
                    cycle_id: cycle_id.clone(),
                },
            ))
            .await
            .unwrap()
            .into_inner();
        if cycle.status == model::execution::wire::CycleStatus::Completed {
            return Some(
                history
                    .unary::<contracts::history::BrowseExecutions>(authenticated(
                        contracts::history::BrowseExecutions {
                            limit: 1,
                            ..Default::default()
                        },
                    ))
                    .await
                    .unwrap()
                    .into_inner()
                    .entries
                    .into_iter()
                    .find(|entry| {
                        entry.metadata.as_ref().is_some_and(|metadata| {
                            metadata.cycle_id.as_deref() == Some(cycle_id.as_str())
                        })
                    })
                    .and_then(|entry| entry.checkpoint)
                    .expect("completed execution checkpoint"),
            );
        }
        None
    })
    .await;

    let evidence = history
        .unary::<contracts::history::GetCheckpointEvidence>(authenticated(
            contracts::history::GetCheckpointEvidence {
                checkpoint: Some(checkpoint.clone()),
                sections: vec![model::history::wire::EvidenceSection::TelemetryTables],
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(evidence.telemetry_tables.len(), 1);
    let telemetry = &evidence.telemetry_tables[0];
    assert_eq!(telemetry.table, "fixture.progress");
    assert_eq!(
        telemetry
            .schema
            .as_ref()
            .map(|schema| schema.stream.as_str()),
        Some("fixture.progress")
    );

    let page = history
        .unary::<contracts::history::ListCheckpointTelemetry>(authenticated(
            contracts::history::ListCheckpointTelemetry {
                checkpoint: Some(checkpoint),
                table: telemetry.table.clone(),
                limit: 10,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.table, telemetry.table);
    assert_eq!(page.rows.len(), 1);

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn historical_checkpoint_content() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let server = Server::start(
        owner.handle(),
        Config::default().with_max_message_bytes(4 * 1024 * 1024),
    )
    .await
    .unwrap();
    let connection = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let execution = WireClient::new(connection.clone());
    let history = WireClient::new(connection.clone());
    let reports = WireClient::new(connection);
    let cycle_id =
        support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();
    let case_id = execution
        .unary::<contracts::execution::ListCases>(authenticated(
            contracts::execution::ListCases {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .cases[0]
        .case_id
        .clone();
    let frozen_case = owner
        .handle()
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id.as_str() == cycle_id)
        .unwrap()
        .cases()
        .find(|case| case.test_case_id.as_str() == case_id)
        .unwrap()
        .clone();
    execution
        .unary::<contracts::execution::RunCase>(authenticated(
            contracts::execution::RunCase {
                cycle_id: cycle_id.clone(),
                case_id,
                context: None,
                idempotency_key: None,
            },
        ))
        .await
        .unwrap();
    gate.started.notified().await;
    gate.release.notify_one();
    support::on_state_change(&owner.handle(), async || {
        if execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycle {
                    cycle_id: cycle_id.clone(),
                },
            ))
            .await
            .unwrap()
            .into_inner()
            .status
            == model::execution::wire::CycleStatus::Completed
        {
            return Some(());
        }
        None
    })
    .await;

    let browse = history
        .unary::<contracts::history::BrowseExecutions>(authenticated(
            contracts::history::BrowseExecutions {
                limit: 1,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let entry = browse
        .entries
        .iter()
        .find(|entry| {
            entry
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.cycle_id.as_deref() == Some(cycle_id.as_str()))
        })
        .expect("completed cycle execution checkpoint");
    let execution_id = entry.metadata.as_ref().unwrap().execution_id.clone();
    let checkpoint = entry
        .checkpoint
        .clone()
        .expect("completed execution checkpoint");

    let mut replacement = support::fixture_suite(&["failed"]);
    replacement.metadata.label = "Replacement Suite".into();
    support::register_fixture_execution(&owner.handle(), replacement).await;

    let document = history
        .unary::<contracts::history::GetCheckpointDocument>(authenticated(
            contracts::history::GetCheckpointDocument {
                checkpoint: Some(checkpoint.clone()),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(document.checkpoint, Some(checkpoint.clone()));
    assert_eq!(
        document
            .document
            .as_ref()
            .and_then(|document| document.execution.as_ref())
            .map(|execution| execution.execution_id.as_str()),
        Some(execution_id.as_str())
    );

    let evidence = history
        .unary::<contracts::history::GetCheckpointEvidence>(authenticated(
            contracts::history::GetCheckpointEvidence {
                checkpoint: Some(checkpoint.clone()),
                sections: vec![],
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(evidence.checkpoint, Some(checkpoint.clone()));
    let frozen_snapshot = document
        .document
        .as_ref()
        .unwrap()
        .execution
        .as_ref()
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap()
        .clone();
    let definition = frozen_snapshot.case_definition.as_ref().unwrap();
    assert_eq!(definition.case_id, frozen_case.test_case_id.as_str());
    assert_eq!(
        definition.template_id,
        frozen_case.test_case_template_id.as_str()
    );
    let metadata = definition.metadata.as_ref().unwrap();
    assert_eq!(
        metadata.identifier,
        frozen_case.metadata.identifier.as_str()
    );
    assert_eq!(metadata.label, frozen_case.metadata.label);
    assert_eq!(definition.suite.as_ref().unwrap().label, "Fixture Suite");
    assert_eq!(definition.generation_index, frozen_case.generation_index);
    assert_eq!(
        definition.parameters,
        vec![model::execution::wire::CaseParameter {
            name: "fixture.result".into(),
            value: Some(model::values::wire::Value::from(
                &dokime_runtime::Value::Text("passed".into())
            )),
        }]
    );
    assert!(
        frozen_snapshot
            .host_version
            .as_ref()
            .is_some_and(|version| !version.is_empty())
    );

    let context = history
        .unary::<contracts::history::GetCheckpointContext>(authenticated(
            contracts::history::GetCheckpointContext {
                checkpoint: Some(checkpoint.clone()),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(context.checkpoint, Some(checkpoint.clone()));
    assert_eq!(context.execution_id.as_deref(), Some(execution_id.as_str()));
    assert_eq!(
        context
            .execution_scope
            .as_ref()
            .and_then(|scope| scope.execution_id.as_deref()),
        Some(execution_id.as_str())
    );
    let records = history
        .unary::<contracts::history::ListCheckpointRecords>(authenticated(
            contracts::history::ListCheckpointRecords {
                checkpoint: Some(checkpoint.clone()),
                limit: 1,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(records.checkpoint, Some(checkpoint.clone()));

    let mut execution_stream = reports
        .download::<contracts::reports::DownloadExecutionReport>(authenticated(
            contracts::reports::DownloadExecutionReport {
                execution_id: checkpoint.execution_identifier.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let mut execution_html = Vec::new();
    while let Some(frame) = execution_stream.message().await.unwrap() {
        match frame.payload.expect("execution report payload") {
            model::reports::wire::ReportHtmlFramePayload::Report(report) => {
                assert_eq!(report.origin, model::reports::wire::ReportOrigin::Sealed);
                assert_eq!(
                    report.test_execution_id.as_deref(),
                    Some(checkpoint.execution_identifier.as_str())
                );
            }
            model::reports::wire::ReportHtmlFramePayload::Chunk(chunk) => {
                execution_html.extend(chunk.data)
            }
            model::reports::wire::ReportHtmlFramePayload::Complete(_) => {}
        }
    }
    assert!(String::from_utf8_lossy(&execution_html).contains("<!doctype html>"));

    let mut html_stream = reports
        .download::<contracts::reports::DownloadCycleReport>(authenticated(
            contracts::reports::DownloadCycleReport {
                cycle_id: cycle_id.clone(),
                identifier: "grpc.download".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    // Offset contiguity and the trailing digest belong to the shared chunked
    // download and are unit-tested there. What this end-to-end path is for is
    // that the stored report streams over the wire and that downloading it leaves
    // the sealed evidence alone.
    let mut html = Vec::new();
    while let Some(frame) = html_stream.message().await.unwrap() {
        match frame.payload.expect("report payload") {
            model::reports::wire::ReportHtmlFramePayload::Report(snapshot) => {
                assert_eq!(snapshot.identifier, "grpc.download");
            }
            model::reports::wire::ReportHtmlFramePayload::Chunk(chunk) => html.extend(chunk.data),
            model::reports::wire::ReportHtmlFramePayload::Complete(_) => {}
        }
    }
    assert!(String::from_utf8_lossy(&html).contains("<!doctype html>"));

    let refreshed = history
        .unary::<contracts::history::BrowseExecutions>(authenticated(
            contracts::history::BrowseExecutions {
                limit: 1,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .entries
        .into_iter()
        .find(|entry| {
            entry
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.execution_id == execution_id)
        })
        .and_then(|entry| entry.checkpoint)
        .expect("report publication checkpoint");
    assert_eq!(
        refreshed, checkpoint,
        "downloading a report cannot advance sealed evidence"
    );
    assert!(owner.handle().artifacts().await.unwrap().is_empty());

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();

    // A restarted runtime renders the captured definitions without installed plugins.
    // Report bytes belong only to each request.
    let mut rebooted = Runtime::start(crate::support::budgets::runtime_config(
        root.path(),
        support::metadata("grpc.fixture", "Fixture"),
    ))
    .await
    .unwrap();
    assert!(
        rebooted
            .handle()
            .watch_state()
            .latest()
            .state()
            .cycles
            .is_empty()
    );
    let rebooted_server = Server::start(
        rebooted.handle(),
        Config::default().with_max_message_bytes(4 * 1024 * 1024),
    )
    .await
    .unwrap();
    let rebooted_history = WireClient::connect(rebooted_server.local_endpoint())
        .await
        .unwrap();
    let restored = rebooted_history
        .unary::<contracts::history::GetCheckpointDocument>(authenticated(
            contracts::history::GetCheckpointDocument {
                checkpoint: Some(checkpoint.clone()),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        restored.document.unwrap().execution.unwrap().snapshot,
        Some(frozen_snapshot),
        "reboot without the original catalog must preserve captured provenance"
    );

    let rebooted_reports = WireClient::connect(rebooted_server.local_endpoint())
        .await
        .unwrap();
    let mut restored_report = rebooted_reports
        .download::<contracts::reports::DownloadCycleReport>(authenticated(
            contracts::reports::DownloadCycleReport {
                cycle_id: cycle_id.clone(),
                identifier: "grpc.download".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let mut restored_html = Vec::new();
    while let Some(frame) = restored_report.message().await.unwrap() {
        if let Some(model::reports::wire::ReportHtmlFramePayload::Chunk(chunk)) = frame.payload {
            assert_eq!(chunk.offset, restored_html.len() as u64);
            restored_html.extend(chunk.data);
        }
    }
    assert_eq!(
        restored_html, html,
        "reboot serves the original sealed report bytes"
    );
    assert!(rebooted.handle().artifacts().await.unwrap().is_empty());

    rebooted_server.shutdown().await.unwrap();
    rebooted.shutdown().await.unwrap();
}

#[tokio::test]
async fn read_role_admit() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default())).await;
    let read = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Read)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(read.local_endpoint()).await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::execution::CreateCycle>(authenticated(create()))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    client
        .unary::<contracts::execution::ListCycles>(authenticated(
            contracts::execution::ListCycles::default(),
        ))
        .await
        .unwrap();
    read.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default())).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut blank = create();
    blank.metadata.as_mut().unwrap().label = "  ".into();
    assert_eq!(
        client
            .unary::<contracts::execution::CreateCycle>(blank)
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let cycle_id = client
        .unary::<contracts::execution::CreateCycle>(create())
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let append = |plan| contracts::execution::AppendPlannedCases {
        cycle_id: cycle_id.clone(),
        plan: Some(plan),
        context: None,
    };
    // A malformed wire value is refused while the plan is decoded, before any
    // suite lookup. What the removed global parameter object also caught —
    // a value that decodes but does not fit its declaration — is now the
    // template parameter's own admission, exercised where that is decided.
    for value in [
        model::values::wire::Value::default(),
        model::values::wire::Value {
            nodes: vec![model::values::wire::ValueNode {
                kind: Some(model::values::wire::ValueKind::Array(
                    model::values::wire::ValueItems { items: vec![99] },
                )),
            }],
        },
    ] {
        let mut plan = selection_plan(&owner);
        plan.selections[0].parameters.push(
            model::planning::wire::RegisteredSelectionParametersEntry {
                key: "fixture.result".into(),
                value: Some(model::planning::wire::SelectionValues {
                    values: vec![value],
                }),
            },
        );
        assert_eq!(
            client
                .unary::<contracts::execution::AppendPlannedCases>(append(plan))
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }
    let mut plan = selection_plan(&owner);
    plan.selections[0].suite_id = "not.registered".into();
    let failure = client
        .unary::<contracts::execution::AppendPlannedCases>(append(plan))
        .await
        .unwrap_err();
    assert_eq!(failure.code(), Code::NotFound);
    let envelope = tonic_types::Status::decode(failure.details()).unwrap();
    assert_eq!(envelope.code, Code::NotFound as i32);
    assert_eq!(envelope.message, failure.message());
    assert_eq!(envelope.details.len(), 1);
    assert_eq!(
        envelope.details[0].type_url,
        "type.googleapis.com/dokime.v1.Rejection"
    );
    let rejection = <model::execution::wire::Rejection as harmos::Protobuf>::decode(
        envelope.details[0].value.as_slice(),
    )
    .unwrap();
    assert_eq!(rejection.code, "suite_not_found");
    assert_eq!(rejection.message, failure.message());
    assert!(
        client
            .unary::<contracts::execution::ListCases>(contracts::execution::ListCases {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner()
            .cases
            .is_empty()
    );
    assert_eq!(
        client
            .unary::<contracts::execution::ListCycles>(contracts::execution::ListCycles {
                limit: 101,
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let observation = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        observation
            .download::<contracts::observation::WatchRuntime>(
                contracts::observation::WatchRuntime {
                    cursor: Some(model::execution::wire_cursor::Cursor {
                        epoch: "other".into(),
                        stream: "runtime.state".into(),
                        position: 0
                    })
                }
            )
            .await
            .unwrap()
            .into_inner()
            .message()
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn stalled_stream_execution() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let service = dokime_server::grpc::Service::new(owner.handle(), Config::default())
        .await
        .unwrap();
    let server = service.listen().await.unwrap();
    let address = server.address();
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let id = support::planned_cycle(&execution, Request::new(create()), selection_plan(&owner))
        .await
        .unwrap();
    let result = execution
        .unary::<contracts::execution::RunCycle>(run(&id))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        result.outcome,
        Some(model::execution::wire::RunReplyOutcome::Accepted(_))
    ));
    gate.started.notified().await;
    let connection = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .initial_connection_window_size(1)
        .initial_stream_window_size(1)
        .connect()
        .await
        .unwrap();
    let observation = WireClient::new(connection);
    let mut stalled = observation
        .download::<contracts::observation::WatchRuntime>(
            contracts::observation::WatchRuntime::default(),
        )
        .await
        .unwrap()
        .into_inner();
    // Leave the stream unread so HTTP/2's one-byte receive window cannot advance.
    server.shutdown().await.unwrap();
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
    assert!(!owner.handle().is_shutting_down());
    async { while let Ok(Some(_)) = stalled.message().await {} }.await;
    gate.release.notify_one();
    let next = service.listen().await.unwrap();
    let client = WireClient::connect(next.local_endpoint()).await.unwrap();
    support::on_state_change(&owner.handle(), async || {
        if client
            .unary::<contracts::execution::GetCycle>(contracts::execution::GetCycle {
                cycle_id: id.clone(),
            })
            .await
            .unwrap()
            .into_inner()
            .status
            == model::execution::wire::CycleStatus::Completed
        {
            return Some(());
        }
        None
    })
    .await;
    next.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn run_cycle_targets() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture_with_provider(root.path(), Arc::new(Progress)).await;
    support::register_fixture_execution_pair(
        &owner.handle(),
        support::fixture_suite(&["passed"]),
        support::fixture_diagnostics_suite(&["passed"]),
    )
    .await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let connection = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let execution = WireClient::new(connection);
    let cycle_id =
        support::planned_cycle(&execution, authenticated(create()), selection_plan(&owner))
            .await
            .unwrap();
    let diagnostics = {
        let snapshot = owner.handle().watch_state().latest();
        let entry = snapshot.state().suites.get("fixture.diagnostics").unwrap();
        model::planning::wire::RegisteredSelection {
            suite_id: entry.suite_id.clone(),
            template_id: entry.suite.test_case_templates[0]
                .test_case_template_id
                .to_string(),
            ..Default::default()
        }
    };
    execution
        .unary::<contracts::execution::AppendPlannedCases>(authenticated(
            contracts::execution::AppendPlannedCases {
                cycle_id: cycle_id.clone(),
                plan: Some(model::planning::wire::SelectionPlan {
                    selections: vec![diagnostics],
                    max_cases: 0,
                    case_ids: vec![],
                    cycle_id: String::new(),
                }),
                context: None,
            },
        ))
        .await
        .unwrap();

    let admitted = execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle_id)))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(
            admitted.outcome,
            Some(model::execution::wire::RunReplyOutcome::Accepted(_))
        ),
        "{:?}",
        admitted.outcome
    );

    let completed = support::on_state_change(&owner.handle(), async || {
        let cycle = execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycle {
                    cycle_id: cycle_id.clone(),
                },
            ))
            .await
            .unwrap()
            .into_inner();
        if cycle.status == model::execution::wire::CycleStatus::Completed {
            return Some(cycle);
        }
        None
    })
    .await;
    assert_eq!(completed.case_count, 2);
    assert_eq!(completed.execution_count, 2);

    let executions = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutions {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .executions;
    assert_eq!(executions.len(), 2);
    assert!(
        executions
            .iter()
            .all(|entry| entry.status == model::execution::wire::ExecutionStatus::Passed),
        "{executions:?}"
    );

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

fn run(id: &str) -> contracts::execution::RunCycle {
    contracts::execution::RunCycle {
        cycle_id: id.into(),
        ..Default::default()
    }
}

/* === Tests === */
