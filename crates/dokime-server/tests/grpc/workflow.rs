/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "pagination.rs"]
pub(crate) mod pagination;

#[path = "search.rs"]
pub(crate) mod search;

#[path = "files.rs"]
pub(crate) mod files;

#[path = "controls.rs"]
pub(crate) mod controls;

#[path = "inspection.rs"]
pub(crate) mod inspection;

#[path = "requests.rs"]
pub(crate) mod requests;

#[path = "steps.rs"]
pub(crate) mod steps;

#[path = "documents_execution.rs"]
pub(crate) mod documents_execution;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use prost::Message;
use tokio::sync::Notify;
// Local Imports
use super::*;
use dokime_app::domains::ExecutionProviderTarget;
use dokime_app::execution::CycleView;
use dokime_app::transport::ProviderExecutionFailure;
use dokime_app::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};

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
        Box::pin(async move { Ok(common::fixture_provider_outcome(cycle, true)) })
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
        let cycle = execution
            .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
            .await
            .unwrap()
            .into_inner()
            .cycle_id;
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
pub(super) fn selection_plan(owner: &Runtime) -> contracts::planning::SelectionPlan {
    let snapshot = owner.handle().watch_state().latest();
    let suite = snapshot.state().suites.get("fixture.suite").unwrap();
    contracts::planning::SelectionPlan {
        selections: suite
            .suite
            .test_case_templates
            .iter()
            .map(|template| contracts::planning::RegisteredSelection {
                suite_id: suite.suite_id.clone(),
                template_id: template.test_case_template_id.to_string(),
                ..Default::default()
            })
            .collect(),
        max_cases: 0,
        case_ids: vec![],
    }
}

/* === Private Functions === */
async fn fixture(root: &std::path::Path, gate: Arc<Gate>) -> Runtime {
    fixture_with_provider(root, gate).await
}

async fn fixture_with_provider(
    root: &std::path::Path,
    provider: Arc<dyn ExecutionFixture>,
) -> Runtime {
    let owner = Runtime::with_execution_fixture(
        RuntimeConfig::new(root, common::metadata("grpc.fixture", "Fixture")),
        "fixture",
        provider,
    )
    .await
    .unwrap();
    common::register_fixture_execution(&owner.handle(), common::fixture_suite(&["passed"])).await;
    owner
}

fn create(owner: &Runtime) -> contracts::execution::CreateCycleRequest {
    contracts::execution::CreateCycleRequest {
        plan: Some(selection_plan(owner)),
        metadata: Some(contracts::common::Metadata {
            identifier: "grpc.cycle".into(),
            label: "gRPC cycle".into(),
            description: None,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn control(id: &str) -> contracts::execution::CycleControlRequest {
    contracts::execution::CycleControlRequest {
        cycle_id: id.into(),
        context: None,
    }
}

pub(crate) async fn registered_create_definitions() {
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
            contracts::catalog::ListSuitesRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(suites.suites.len(), 1);
    let suite = catalog
        .unary::<contracts::catalog::GetSuite>(authenticated(contracts::catalog::GetSuiteRequest {
            suite_id: suites.suites[0].suite_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(suite.templates.len(), 1);
    assert_eq!(
        suite.templates[0].parameters[0].value_type,
        contracts::schema::ValueType::Text
    );
    assert!(matches!(
        suite.templates[0].parameters[0]
            .expression
            .as_ref()
            .unwrap()
            .nodes[0]
            .kind,
        Some(contracts::catalog::ParameterNodeKind::Literal(_))
    ));
    let mut stream = observation
        .download::<contracts::observation::WatchRuntime>(authenticated(
            contracts::observation::WatchRuntimeRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    let initial = stream.message().await.unwrap().unwrap();
    assert!(initial.resynchronized);
    let id = execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let planned = execution
        .unary::<contracts::execution::GetCycle>(authenticated(
            contracts::execution::GetCycleRequest {
                cycle_id: id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(planned.status, contracts::execution::CycleStatus::Ready);
    assert_eq!(planned.case_count, 1);
    assert_eq!(planned.execution_count, 1);
    let cases = execution
        .unary::<contracts::execution::ListCases>(authenticated(
            contracts::execution::ListCasesRequest {
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
            contracts::execution::RunCaseRequest {
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
        Some(contracts::execution::RunReplyOutcome::Accepted(_))
    ));
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    execution
        .unary::<contracts::execution::PauseCycle>(authenticated(control(&id)))
        .await
        .unwrap();
    execution
        .unary::<contracts::execution::ResumeCycle>(authenticated(control(&id)))
        .await
        .unwrap();
    // Disconnect observation while admitted work is waiting on the fixture.
    drop(stream);
    gate.release.notify_one();
    let mut stream = observation
        .download::<contracts::observation::WatchRuntime>(authenticated(
            contracts::observation::WatchRuntimeRequest {
                cursor: initial.cursor.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(stream.message().await.unwrap().unwrap().resynchronized);
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let cycle = execution
                .unary::<contracts::execution::GetCycle>(authenticated(
                    contracts::execution::GetCycleRequest {
                        cycle_id: id.clone(),
                    },
                ))
                .await
                .unwrap()
                .into_inner();
            if cycle.status == contracts::execution::CycleStatus::Completed {
                assert_eq!(cycle.execution_count, 1);
                let attempts = execution
                    .unary::<contracts::execution::ListExecutions>(authenticated(
                        contracts::execution::ListExecutionsRequest {
                            cycle_id: id.clone(),
                            ..Default::default()
                        },
                    ))
                    .await
                    .unwrap()
                    .into_inner();
                assert_eq!(
                    attempts.executions[0].status,
                    contracts::execution::ExecutionStatus::Passed
                );
                break;
            }
            stream.message().await.unwrap().unwrap();
        }
    })
    .await
    .unwrap();
    let list = execution
        .unary::<contracts::execution::ListCycles>(authenticated(
            contracts::execution::ListCyclesRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(list.cycles.len(), 1);
    let other = execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    execution
        .unary::<contracts::execution::StopCycle>(authenticated(control(&other)))
        .await
        .unwrap();
    assert_eq!(
        execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycleRequest { cycle_id: other }
            ))
            .await
            .unwrap()
            .into_inner()
            .status,
        contracts::execution::CycleStatus::Stopped
    );
    tokio::time::timeout(Duration::from_secs(6), server.shutdown())
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
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
    })
    .await
    .unwrap();
    assert!(!owner.handle().is_shutting_down());
    owner.shutdown().await.unwrap();
}

pub(crate) async fn stopping_running_unwind() {
    let root = tempfile::tempdir().unwrap();
    let blocker = Arc::new(BlockingStop::default());
    let mut owner = fixture_with_provider(root.path(), blocker.clone()).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let reply = execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle_id)))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        reply.outcome,
        Some(contracts::execution::RunReplyOutcome::Accepted(_))
    ));
    tokio::time::timeout(Duration::from_secs(5), blocker.started.notified())
        .await
        .unwrap();

    execution
        .unary::<contracts::execution::StopCycle>(authenticated(control(&cycle_id)))
        .await
        .unwrap();
    let stopping = execution
        .unary::<contracts::execution::GetCycle>(authenticated(
            contracts::execution::GetCycleRequest {
                cycle_id: cycle_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(stopping.status, contracts::execution::CycleStatus::Stopping);

    blocker.release.notify_one();
    let stopped = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let summary = execution
                .unary::<contracts::execution::GetCycle>(authenticated(
                    contracts::execution::GetCycleRequest {
                        cycle_id: cycle_id.clone(),
                    },
                ))
                .await
                .unwrap()
                .into_inner();
            if summary.status == contracts::execution::CycleStatus::Stopped {
                break summary;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(stopped.execution_count, 1);
    let executions = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
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
        contracts::execution::ExecutionStatus::Cancelled
    );

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn runtime_shell_cursor() {
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
    let first_cycle = execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let second_cycle = execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;

    let first = observation
        .unary::<contracts::observation::GetRuntimeShell>(authenticated(
            contracts::observation::GetRuntimeShellRequest {
                limit: 1,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.cycle_summaries.len(), 1);
    assert!(first.cycle_summaries[0].suite.is_some());
    let next_cycle = first
        .next_after_cycle_id
        .clone()
        .expect("two cycles require a continuation");
    let cursor = first.observed_cursor.clone().expect("shell cursor");

    let second = observation
        .unary::<contracts::observation::GetRuntimeShell>(authenticated(
            contracts::observation::GetRuntimeShellRequest {
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

    let future_cursor = contracts::cursor::Cursor {
        position: cursor.position + 1,
        ..cursor.clone()
    };
    assert_eq!(
        observation
            .unary::<contracts::observation::GetRuntimeShell>(authenticated(
                contracts::observation::GetRuntimeShellRequest {
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
                contracts::observation::GetRuntimeShellRequest {
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
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap();
    assert_eq!(
        observation
            .unary::<contracts::observation::GetRuntimeShell>(authenticated(
                contracts::observation::GetRuntimeShellRequest {
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

pub(crate) async fn checkpoint_telemetry_queries() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture_with_provider(root.path(), Arc::new(Progress)).await;
    common::register_fixture_progress_execution(
        &owner.handle(),
        common::fixture_suite(&["passed"]),
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
    let cycle_id = execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle_id)))
        .await
        .unwrap();
    let checkpoint = tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let cycle = execution
                .unary::<contracts::execution::GetCycle>(authenticated(
                    contracts::execution::GetCycleRequest {
                        cycle_id: cycle_id.clone(),
                    },
                ))
                .await
                .unwrap()
                .into_inner();
            if cycle.status == contracts::execution::CycleStatus::Completed {
                break history
                    .unary::<contracts::history::BrowseExecutions>(authenticated(
                        contracts::history::BrowseExecutionsRequest {
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
                    .expect("completed execution checkpoint");
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();

    let evidence = history
        .unary::<contracts::history::GetCheckpointEvidence>(authenticated(
            contracts::history::GetCheckpointEvidenceRequest {
                checkpoint: Some(checkpoint.clone()),
                sections: vec![contracts::history::EvidenceSection::TelemetryTables],
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
            contracts::history::ListCheckpointTelemetryRequest {
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

pub(crate) async fn historical_checkpoint_content() {
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
    let cycle_id = execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let case_id = execution
        .unary::<contracts::execution::ListCases>(authenticated(
            contracts::execution::ListCasesRequest {
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
            contracts::execution::RunCaseRequest {
                cycle_id: cycle_id.clone(),
                case_id,
                context: None,
                idempotency_key: None,
            },
        ))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    gate.release.notify_one();
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            if execution
                .unary::<contracts::execution::GetCycle>(authenticated(
                    contracts::execution::GetCycleRequest {
                        cycle_id: cycle_id.clone(),
                    },
                ))
                .await
                .unwrap()
                .into_inner()
                .status
                == contracts::execution::CycleStatus::Completed
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();

    let browse = history
        .unary::<contracts::history::BrowseExecutions>(authenticated(
            contracts::history::BrowseExecutionsRequest {
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

    let mut replacement = common::fixture_suite(&["failed"]);
    replacement.metadata.label = "Replacement Suite".into();
    common::register_fixture_execution(&owner.handle(), replacement).await;

    let document = history
        .unary::<contracts::history::GetCheckpointDocument>(authenticated(
            contracts::history::GetCheckpointDocumentRequest {
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
            contracts::history::GetCheckpointEvidenceRequest {
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
        vec![contracts::execution::CaseParameter {
            name: "fixture.result".into(),
            value: Some(contracts::value::Value::from(
                &dokime_app::core::Value::Text("passed".into())
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
            contracts::history::GetCheckpointContextRequest {
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
            contracts::history::ListCheckpointRecordsRequest {
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
            contracts::reports::DownloadExecutionReportRequest {
                execution_id: checkpoint.execution_identifier.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let mut execution_html = Vec::new();
    while let Some(frame) = execution_stream.message().await.unwrap() {
        match frame.payload.expect("execution report payload") {
            contracts::reports::ReportHtmlFramePayload::Report(report) => {
                assert_eq!(report.origin, contracts::reports::ReportOrigin::Sealed);
                assert_eq!(
                    report.test_execution_id.as_deref(),
                    Some(checkpoint.execution_identifier.as_str())
                );
            }
            contracts::reports::ReportHtmlFramePayload::Chunk(chunk) => {
                execution_html.extend(chunk.data)
            }
            contracts::reports::ReportHtmlFramePayload::Complete(_) => {}
        }
    }
    assert!(String::from_utf8_lossy(&execution_html).contains("<!doctype html>"));

    let mut html_stream = reports
        .download::<contracts::reports::DownloadCycleReport>(authenticated(
            contracts::reports::DownloadCycleReportRequest {
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
            contracts::reports::ReportHtmlFramePayload::Report(snapshot) => {
                assert_eq!(snapshot.identifier, "grpc.download");
            }
            contracts::reports::ReportHtmlFramePayload::Chunk(chunk) => html.extend(chunk.data),
            contracts::reports::ReportHtmlFramePayload::Complete(_) => {}
        }
    }
    assert!(String::from_utf8_lossy(&html).contains("<!doctype html>"));

    let refreshed = history
        .unary::<contracts::history::BrowseExecutions>(authenticated(
            contracts::history::BrowseExecutionsRequest {
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
    let mut rebooted = Runtime::start(RuntimeConfig::new(
        root.path(),
        common::metadata("grpc.fixture", "Fixture"),
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
            contracts::history::GetCheckpointDocumentRequest {
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
            contracts::reports::DownloadCycleReportRequest {
                cycle_id: cycle_id.clone(),
                identifier: "grpc.download".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let mut restored_html = Vec::new();
    while let Some(frame) = restored_report.message().await.unwrap() {
        if let Some(contracts::reports::ReportHtmlFramePayload::Chunk(chunk)) = frame.payload {
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

pub(crate) async fn read_role_admit() {
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
            .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    client
        .unary::<contracts::execution::ListCycles>(authenticated(
            contracts::execution::ListCyclesRequest::default(),
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
    // A malformed wire value is refused while the plan is decoded, before any
    // suite lookup. What the removed global parameter object also caught —
    // a value that decodes but does not fit its declaration — is now the
    // template parameter's own admission, exercised where that is decided.
    for value in [
        contracts::value::Value::default(),
        contracts::value::Value {
            nodes: vec![contracts::value::ValueNode {
                kind: Some(contracts::value::ValueKind::Array(
                    contracts::value::ValueItems { items: vec![99] },
                )),
            }],
        },
    ] {
        let mut request = create(&owner);
        request.plan.as_mut().unwrap().selections[0]
            .parameters
            .push(contracts::planning::RegisteredSelectionParametersEntry {
                key: "fixture.result".into(),
                value: Some(contracts::planning::SelectionValues {
                    values: vec![value],
                }),
            });
        assert_eq!(
            client
                .unary::<contracts::execution::CreateCycle>(request)
                .await
                .unwrap_err()
                .code(),
            Code::InvalidArgument
        );
    }
    let mut request = create(&owner);
    request.plan.as_mut().unwrap().selections[0].suite_id = "not.registered".into();
    let failure = client
        .unary::<contracts::execution::CreateCycle>(request)
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
    let rejection = <contracts::execution::Rejection as harmos::Protobuf>::decode(
        envelope.details[0].value.as_slice(),
    )
    .unwrap();
    assert_eq!(rejection.code, "suite_not_found");
    assert_eq!(rejection.message, failure.message());
    assert!(
        client
            .unary::<contracts::execution::ListCycles>(
                contracts::execution::ListCyclesRequest::default()
            )
            .await
            .unwrap()
            .into_inner()
            .cycles
            .is_empty()
    );
    assert_eq!(
        client
            .unary::<contracts::execution::ListCycles>(contracts::execution::ListCyclesRequest {
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
                contracts::observation::WatchRuntimeRequest {
                    cursor: Some(contracts::cursor::Cursor {
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

pub(crate) async fn stalled_stream_execution() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let service = dokime_server::grpc::Service::new(owner.handle(), Config::default())
        .await
        .unwrap();
    let server = service.listen().await.unwrap();
    let address = server.address();
    let execution = WireClient::connect(server.local_endpoint()).await.unwrap();
    let id = execution
        .unary::<contracts::execution::CreateCycle>(create(&owner))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let result = execution
        .unary::<contracts::execution::RunCycle>(run(&id))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        result.outcome,
        Some(contracts::execution::RunReplyOutcome::Accepted(_))
    ));
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
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
            contracts::observation::WatchRuntimeRequest::default(),
        )
        .await
        .unwrap()
        .into_inner();
    // Leave the stream unread so HTTP/2's one-byte receive window cannot advance.
    tokio::time::timeout(Duration::from_secs(8), server.shutdown())
        .await
        .unwrap()
        .unwrap();
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
    assert!(!owner.handle().is_shutting_down());
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Ok(Some(_)) = stalled.message().await {}
    })
    .await
    .unwrap();
    gate.release.notify_one();
    let next = service.listen().await.unwrap();
    let client = WireClient::connect(next.local_endpoint()).await.unwrap();
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            if client
                .unary::<contracts::execution::GetCycle>(contracts::execution::GetCycleRequest {
                    cycle_id: id.clone(),
                })
                .await
                .unwrap()
                .into_inner()
                .status
                == contracts::execution::CycleStatus::Completed
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    next.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn run_cycle_targets() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture_with_provider(root.path(), Arc::new(Progress)).await;
    common::register_fixture_execution_pair(
        &owner.handle(),
        common::fixture_suite(&["passed"]),
        common::fixture_diagnostics_suite(&["passed"]),
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
    let cycle_id = execution
        .unary::<contracts::execution::CreateCycle>(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let diagnostics = {
        let snapshot = owner.handle().watch_state().latest();
        let entry = snapshot.state().suites.get("fixture.diagnostics").unwrap();
        contracts::planning::RegisteredSelection {
            suite_id: entry.suite_id.clone(),
            template_id: entry.suite.test_case_templates[0]
                .test_case_template_id
                .to_string(),
            ..Default::default()
        }
    };
    execution
        .unary::<contracts::execution::AppendPlannedCases>(authenticated(
            contracts::execution::AppendPlannedCasesRequest {
                cycle_id: cycle_id.clone(),
                plan: Some(contracts::planning::SelectionPlan {
                    selections: vec![diagnostics],
                    max_cases: 0,
                    case_ids: vec![],
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
            Some(contracts::execution::RunReplyOutcome::Accepted(_))
        ),
        "{:?}",
        admitted.outcome
    );

    let completed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let cycle = execution
                .unary::<contracts::execution::GetCycle>(authenticated(
                    contracts::execution::GetCycleRequest {
                        cycle_id: cycle_id.clone(),
                    },
                ))
                .await
                .unwrap()
                .into_inner();
            if cycle.status == contracts::execution::CycleStatus::Completed {
                break cycle;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(completed.case_count, 2);
    assert_eq!(completed.execution_count, 2);

    let executions = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
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
            .all(|entry| entry.status == contracts::execution::ExecutionStatus::Passed),
        "{executions:?}"
    );

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

fn run(id: &str) -> contracts::execution::RunCycleRequest {
    contracts::execution::RunCycleRequest {
        cycle_id: id.into(),
        ..Default::default()
    }
}

/* === Tests === */
