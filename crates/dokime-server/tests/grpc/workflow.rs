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
use dokime_protocol::v1;
use dokime_protocol::v1::{
    catalog_client::CatalogClient, execution_client::ExecutionClient,
    history_client::HistoryClient, observation_client::ObservationClient,
    reports_client::ReportsClient,
};

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
    server: Server,
    execution: ExecutionClient<Channel>,
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
        let server = Server::start(owner.handle(), config).await.unwrap();
        let mut execution = ExecutionClient::connect(server.local_endpoint())
            .await
            .unwrap();
        let cycle = execution
            .create_cycle(authenticated(create(&owner)))
            .await
            .unwrap()
            .into_inner()
            .cycle_id;
        Self {
            root,
            gate,
            owner,
            server,
            execution,
            cycle,
        }
    }
}

/* === Public Functions === */
pub(super) fn selection_plan(owner: &Runtime) -> v1::SelectionPlan {
    let snapshot = owner.handle().watch_state().latest();
    let suite = snapshot.state().suites.get("fixture.suite").unwrap();
    v1::SelectionPlan {
        selections: suite
            .suite
            .test_case_templates
            .iter()
            .map(|template| v1::RegisteredSelection {
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

fn create(owner: &Runtime) -> v1::CreateCycleRequest {
    v1::CreateCycleRequest {
        plan: Some(selection_plan(owner)),
        metadata: Some(v1::Metadata {
            identifier: "grpc.cycle".into(),
            label: "gRPC cycle".into(),
            description: None,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn control(id: &str) -> v1::CycleControlRequest {
    v1::CycleControlRequest {
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
    let mut catalog = CatalogClient::new(connection.clone());
    let mut execution = ExecutionClient::new(connection.clone());
    let mut observation = ObservationClient::new(connection);
    let suites = catalog
        .list_suites(authenticated(v1::ListSuitesRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(suites.suites.len(), 1);
    let suite = catalog
        .get_suite(authenticated(v1::GetSuiteRequest {
            suite_id: suites.suites[0].suite_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(suite.templates.len(), 1);
    assert_eq!(
        suite.templates[0].parameters[0].value_type(),
        v1::ValueType::Text
    );
    assert!(matches!(
        suite.templates[0].parameters[0]
            .expression
            .as_ref()
            .unwrap()
            .kind,
        Some(v1::parameter_expression::Kind::Literal(_))
    ));
    let mut stream = observation
        .watch_runtime(authenticated(v1::WatchRuntimeRequest::default()))
        .await
        .unwrap()
        .into_inner();
    let initial = stream.message().await.unwrap().unwrap();
    assert!(initial.resynchronized);
    let id = execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let planned = execution
        .get_cycle(authenticated(v1::GetCycleRequest {
            cycle_id: id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(planned.status(), v1::CycleStatus::Ready);
    assert_eq!(planned.case_count, 1);
    assert_eq!(planned.execution_count, 1);
    let cases = execution
        .list_cases(authenticated(v1::ListCasesRequest {
            cycle_id: id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .cases;
    assert_eq!(cases[0].template_id, suite.templates[0].template_id);
    let admitted = execution
        .run_case(authenticated(v1::RunCaseRequest {
            cycle_id: id.clone(),
            case_id: cases[0].case_id.clone(),
            context: None,
            idempotency_key: None,
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        admitted.outcome,
        Some(v1::run_reply::Outcome::Accepted(_))
    ));
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    execution
        .pause_cycle(authenticated(control(&id)))
        .await
        .unwrap();
    execution
        .resume_cycle(authenticated(control(&id)))
        .await
        .unwrap();
    // Disconnect observation while admitted work is waiting on the fixture.
    drop(stream);
    gate.release.notify_one();
    let mut stream = observation
        .watch_runtime(authenticated(v1::WatchRuntimeRequest {
            cursor: initial.cursor.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(stream.message().await.unwrap().unwrap().resynchronized);
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let cycle = execution
                .get_cycle(authenticated(v1::GetCycleRequest {
                    cycle_id: id.clone(),
                }))
                .await
                .unwrap()
                .into_inner();
            if cycle.status() == v1::CycleStatus::Completed {
                assert_eq!(cycle.execution_count, 1);
                let attempts = execution
                    .list_executions(authenticated(v1::ListExecutionsRequest {
                        cycle_id: id.clone(),
                        ..Default::default()
                    }))
                    .await
                    .unwrap()
                    .into_inner();
                assert_eq!(attempts.executions[0].status(), v1::ExecutionStatus::Passed);
                break;
            }
            stream.message().await.unwrap().unwrap();
        }
    })
    .await
    .unwrap();
    let list = execution
        .list_cycles(authenticated(v1::ListCyclesRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(list.cycles.len(), 1);
    let other = execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    execution
        .stop_cycle(authenticated(control(&other)))
        .await
        .unwrap();
    assert_eq!(
        execution
            .get_cycle(authenticated(v1::GetCycleRequest { cycle_id: other }))
            .await
            .unwrap()
            .into_inner()
            .status(),
        v1::CycleStatus::Stopped
    );
    tokio::time::timeout(Duration::from_secs(6), server.shutdown())
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while stream.message().await.unwrap().is_some() {}
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
    let mut execution = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let cycle_id = execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let reply = execution
        .run_cycle(authenticated(run(&cycle_id)))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        reply.outcome,
        Some(v1::run_reply::Outcome::Accepted(_))
    ));
    tokio::time::timeout(Duration::from_secs(5), blocker.started.notified())
        .await
        .unwrap();

    execution
        .stop_cycle(authenticated(control(&cycle_id)))
        .await
        .unwrap();
    let stopping = execution
        .get_cycle(authenticated(v1::GetCycleRequest {
            cycle_id: cycle_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(stopping.status(), v1::CycleStatus::Stopping);

    blocker.release.notify_one();
    let stopped = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let summary = execution
                .get_cycle(authenticated(v1::GetCycleRequest {
                    cycle_id: cycle_id.clone(),
                }))
                .await
                .unwrap()
                .into_inner();
            if summary.status() == v1::CycleStatus::Stopped {
                break summary;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(stopped.execution_count, 1);
    let executions = execution
        .list_executions(authenticated(v1::ListExecutionsRequest {
            cycle_id,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(executions.executions.len(), 1);
    assert_eq!(
        executions.executions[0].status(),
        v1::ExecutionStatus::Cancelled
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
    let mut execution = ExecutionClient::new(connection.clone());
    let mut observation = ObservationClient::new(connection);
    let first_cycle = execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let second_cycle = execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;

    let first = observation
        .get_runtime_shell(authenticated(v1::GetRuntimeShellRequest {
            limit: 1,
            ..Default::default()
        }))
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
        .get_runtime_shell(authenticated(v1::GetRuntimeShellRequest {
            limit: 1,
            after_cycle_id: next_cycle.clone(),
            cursor: Some(cursor.clone()),
        }))
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

    let future_cursor = v1::Cursor {
        position: cursor.position + 1,
        ..cursor.clone()
    };
    assert_eq!(
        observation
            .get_runtime_shell(authenticated(v1::GetRuntimeShellRequest {
                limit: 1,
                after_cycle_id: next_cycle.clone(),
                cursor: Some(future_cursor),
            }))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::OutOfRange
    );
    let mut wrong_epoch = cursor.clone();
    wrong_epoch.epoch = "other-session".into();
    assert_eq!(
        observation
            .get_runtime_shell(authenticated(v1::GetRuntimeShellRequest {
                limit: 1,
                after_cycle_id: next_cycle.clone(),
                cursor: Some(wrong_epoch),
            }))
            .await
            .unwrap_err()
            .code(),
        tonic::Code::FailedPrecondition
    );

    execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap();
    assert_eq!(
        observation
            .get_runtime_shell(authenticated(v1::GetRuntimeShellRequest {
                limit: 1,
                after_cycle_id: next_cycle,
                cursor: Some(cursor),
            }))
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
    let mut execution = ExecutionClient::new(connection.clone());
    let mut history = HistoryClient::new(connection);
    let cycle_id = execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    execution
        .run_cycle(authenticated(run(&cycle_id)))
        .await
        .unwrap();
    let checkpoint = tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let cycle = execution
                .get_cycle(authenticated(v1::GetCycleRequest {
                    cycle_id: cycle_id.clone(),
                }))
                .await
                .unwrap()
                .into_inner();
            if cycle.status() == v1::CycleStatus::Completed {
                break history
                    .browse_executions(authenticated(v1::BrowseExecutionsRequest {
                        limit: 1,
                        ..Default::default()
                    }))
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
        .get_checkpoint_evidence(authenticated(v1::GetCheckpointEvidenceRequest {
            checkpoint: Some(checkpoint.clone()),
            sections: vec![v1::EvidenceSection::TelemetryTables as i32],
        }))
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
        .list_checkpoint_telemetry(authenticated(v1::ListCheckpointTelemetryRequest {
            checkpoint: Some(checkpoint),
            table: telemetry.table.clone(),
            limit: 10,
            ..Default::default()
        }))
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
    let mut execution = ExecutionClient::new(connection.clone());
    let mut history = HistoryClient::new(connection.clone());
    let mut reports = ReportsClient::new(connection);
    let cycle_id = execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let case_id = execution
        .list_cases(authenticated(v1::ListCasesRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        }))
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
        .run_case(authenticated(v1::RunCaseRequest {
            cycle_id: cycle_id.clone(),
            case_id,
            context: None,
            idempotency_key: None,
        }))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    gate.release.notify_one();
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            if execution
                .get_cycle(authenticated(v1::GetCycleRequest {
                    cycle_id: cycle_id.clone(),
                }))
                .await
                .unwrap()
                .into_inner()
                .status()
                == v1::CycleStatus::Completed
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();

    let browse = history
        .browse_executions(authenticated(v1::BrowseExecutionsRequest {
            limit: 1,
            ..Default::default()
        }))
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
        .get_checkpoint_document(authenticated(v1::GetCheckpointDocumentRequest {
            checkpoint: Some(checkpoint.clone()),
        }))
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
        .get_checkpoint_evidence(authenticated(v1::GetCheckpointEvidenceRequest {
            checkpoint: Some(checkpoint.clone()),
            sections: vec![],
        }))
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
        vec![v1::CaseParameter {
            name: "fixture.result".into(),
            value: Some(v1::Value {
                kind: Some(v1::value::Kind::TextValue("passed".into())),
            }),
        }]
    );
    assert!(
        frozen_snapshot
            .host_version
            .as_ref()
            .is_some_and(|version| !version.is_empty())
    );

    let context = history
        .get_checkpoint_context(authenticated(v1::GetCheckpointContextRequest {
            checkpoint: Some(checkpoint.clone()),
        }))
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
        .list_checkpoint_records(authenticated(v1::ListCheckpointRecordsRequest {
            checkpoint: Some(checkpoint.clone()),
            limit: 1,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(records.checkpoint, Some(checkpoint.clone()));

    let refusal = reports
        .render_checkpoint_report(authenticated(v1::RenderCheckpointReportRequest {
            context: Some(checkpoint.clone()),
            checkpoints: vec![checkpoint.clone()],
            identifier: "grpc.rendered".into(),
        }))
        .await
        .unwrap_err();
    assert_eq!(refusal.code(), Code::Unimplemented);

    let mut html_stream = reports
        .download_cycle_report(authenticated(v1::DownloadCycleReportRequest {
            cycle_id: cycle_id.clone(),
            identifier: "grpc.download".into(),
        }))
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
            v1::report_html_frame::Payload::Report(snapshot) => {
                assert_eq!(snapshot.identifier, "grpc.download");
            }
            v1::report_html_frame::Payload::Chunk(chunk) => html.extend(chunk.data),
            v1::report_html_frame::Payload::Complete(_) => {}
        }
    }
    assert!(String::from_utf8_lossy(&html).contains("<!doctype html>"));

    let refreshed = history
        .browse_executions(authenticated(v1::BrowseExecutionsRequest {
            limit: 1,
            ..Default::default()
        }))
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
    let mut rebooted_history = HistoryClient::connect(rebooted_server.local_endpoint())
        .await
        .unwrap();
    let restored = rebooted_history
        .get_checkpoint_document(authenticated(v1::GetCheckpointDocumentRequest {
            checkpoint: Some(checkpoint.clone()),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        restored.document.unwrap().execution.unwrap().snapshot,
        Some(frozen_snapshot),
        "reboot without the original catalog must preserve captured provenance"
    );

    let mut rebooted_reports = ReportsClient::connect(rebooted_server.local_endpoint())
        .await
        .unwrap();
    let mut restored_report = rebooted_reports
        .download_cycle_report(authenticated(v1::DownloadCycleReportRequest {
            cycle_id: cycle_id.clone(),
            identifier: "grpc.download".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    let mut restored_html = Vec::new();
    while let Some(frame) = restored_report.message().await.unwrap() {
        if let Some(v1::report_html_frame::Payload::Chunk(chunk)) = frame.payload {
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
    let mut client = ExecutionClient::connect(read.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        client
            .create_cycle(authenticated(create(&owner)))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    client
        .list_cycles(authenticated(v1::ListCyclesRequest::default()))
        .await
        .unwrap();
    read.shutdown().await.unwrap();
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    // A malformed wire value is refused while the plan is decoded, before any
    // suite lookup. What the removed global parameter object also caught —
    // a value that decodes but does not fit its declaration — is now the
    // template parameter's own admission, exercised where that is decided.
    for value in [
        v1::Value::default(),
        v1::Value {
            kind: Some(v1::value::Kind::NullValue(99)),
        },
    ] {
        let mut request = create(&owner);
        request.plan.as_mut().unwrap().selections[0]
            .parameters
            .insert(
                "fixture.result".into(),
                v1::SelectionValues {
                    values: vec![value],
                },
            );
        assert_eq!(
            client.create_cycle(request).await.unwrap_err().code(),
            Code::InvalidArgument
        );
    }
    let mut request = create(&owner);
    request.plan.as_mut().unwrap().selections[0].suite_id = "not.registered".into();
    let failure = client.create_cycle(request).await.unwrap_err();
    assert_eq!(failure.code(), Code::NotFound);
    let envelope = tonic_types::Status::decode(failure.details()).unwrap();
    assert_eq!(envelope.code, Code::NotFound as i32);
    assert_eq!(envelope.message, failure.message());
    assert_eq!(envelope.details.len(), 1);
    assert_eq!(
        envelope.details[0].type_url,
        "type.googleapis.com/dokime.v1.Rejection"
    );
    let rejection = v1::Rejection::decode(envelope.details[0].value.as_slice()).unwrap();
    assert_eq!(rejection.code, "suite_not_found");
    assert_eq!(rejection.message, failure.message());
    assert!(
        client
            .list_cycles(v1::ListCyclesRequest::default())
            .await
            .unwrap()
            .into_inner()
            .cycles
            .is_empty()
    );
    assert_eq!(
        client
            .list_cycles(v1::ListCyclesRequest {
                limit: 101,
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let mut observation = ObservationClient::connect(server.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        observation
            .watch_runtime(v1::WatchRuntimeRequest {
                cursor: Some(v1::Cursor {
                    epoch: "other".into(),
                    stream: "runtime.state".into(),
                    position: 0
                })
            })
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
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let address = server.address();
    let mut execution = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let id = execution
        .create_cycle(create(&owner))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let result = execution.run_cycle(run(&id)).await.unwrap().into_inner();
    assert!(matches!(
        result.outcome,
        Some(v1::run_reply::Outcome::Accepted(_))
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
    let mut observation = ObservationClient::new(connection);
    let mut stalled = observation
        .watch_runtime(v1::WatchRuntimeRequest::default())
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
    let next = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut client = ExecutionClient::connect(next.local_endpoint())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            if client
                .get_cycle(v1::GetCycleRequest {
                    cycle_id: id.clone(),
                })
                .await
                .unwrap()
                .into_inner()
                .status()
                == v1::CycleStatus::Completed
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
    let mut execution = ExecutionClient::new(connection);
    let cycle_id = execution
        .create_cycle(authenticated(create(&owner)))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let diagnostics = {
        let snapshot = owner.handle().watch_state().latest();
        let entry = snapshot.state().suites.get("fixture.diagnostics").unwrap();
        v1::RegisteredSelection {
            suite_id: entry.suite_id.clone(),
            template_id: entry.suite.test_case_templates[0]
                .test_case_template_id
                .to_string(),
            ..Default::default()
        }
    };
    execution
        .append_planned_cases(authenticated(v1::AppendPlannedCasesRequest {
            cycle_id: cycle_id.clone(),
            plan: Some(v1::SelectionPlan {
                selections: vec![diagnostics],
                max_cases: 0,
                case_ids: vec![],
            }),
            context: None,
        }))
        .await
        .unwrap();

    let admitted = execution
        .run_cycle(authenticated(run(&cycle_id)))
        .await
        .unwrap()
        .into_inner();
    assert!(
        matches!(admitted.outcome, Some(v1::run_reply::Outcome::Accepted(_))),
        "{:?}",
        admitted.outcome
    );

    let completed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let cycle = execution
                .get_cycle(authenticated(v1::GetCycleRequest {
                    cycle_id: cycle_id.clone(),
                }))
                .await
                .unwrap()
                .into_inner();
            if cycle.status() == v1::CycleStatus::Completed {
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
        .list_executions(authenticated(v1::ListExecutionsRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .executions;
    assert_eq!(executions.len(), 2);
    assert!(
        executions
            .iter()
            .all(|entry| entry.status() == v1::ExecutionStatus::Passed),
        "{executions:?}"
    );

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

fn run(id: &str) -> v1::RunCycleRequest {
    v1::RunCycleRequest {
        cycle_id: id.into(),
        ..Default::default()
    }
}

/* === Tests === */
