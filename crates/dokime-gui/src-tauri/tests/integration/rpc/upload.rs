/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::wire::WireMessage;
use dokime::harmos::MemorySize;
use sha2::{Digest, Sha256};

use super::invoke;
use crate::support::{artifact, server::Fixture};
use dokime::domain::model;
use dokime_gui::rpc::{Bridge, FILE_DESCRIPTOR_SET, handler};

fn generation(session: &dokime_gui::connection::ConnectionSession) -> u64 {
    session.status().generation.parse().expect("fixture generation is numeric")
}

fn header(size_bytes: u64, scope: Option<model::content::wire::ArtifactScope>) -> model::content::wire::UploadArtifactHeader {
    model::content::wire::UploadArtifactHeader {
        scope,
        filename: "upload-sink-fixture.bin".into(),
        content_type: Some("application/octet-stream".into()),
        size_bytes,
        sha256: None,
    }
}

fn app_and_window(
    session: &dokime_gui::connection::ConnectionSession,
) -> (tauri::App<tauri::test::MockRuntime>, tauri::WebviewWindow<tauri::test::MockRuntime>) {
    let app = tauri::test::mock_builder()
        .manage(session.clone())
        .manage(Bridge::default())
        .invoke_handler(handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
    (app, window)
}

#[derive(Default)]
struct ActiveFixture {
    release: tokio::sync::Notify,
}
impl dokime::ExecutionFixture for ActiveFixture {
    fn run_cycle<'a>(
        &'a self, context: dokime::ExecutionFixtureContext, cycle: &'a dokime::domain::model::execution::result::CycleView,
        _target: &'a dokime::domains::ExecutionProviderTarget,
    ) -> dokime::ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            use dokime::transport::{ProviderCycleOutcome, ProviderExecutionOutcome, ProviderStatusOutcome};
            let execution = &cycle.test_executions[0];
            context
                .update_execution_phase(
                    execution.scope.clone(),
                    dokime::Identifier::new("rpc.artifact-upload.transfer").unwrap(),
                    "Artifact Transfer".into(),
                    "running".into(),
                )
                .await
                .unwrap();
            self.release.notified().await;
            Ok(ProviderCycleOutcome {
                status: "passed".into(),
                dependency_imports: vec![],
                failures: vec![],
                provider: ProviderStatusOutcome {
                    status: "active".into(),
                    domain_registration_count: 1,
                    test_suite_count: 1,
                    test_suite_identifiers: vec!["rpc.artifact-upload".into()],
                    error: None,
                },
                executions: vec![ProviderExecutionOutcome {
                    case_id: execution.test_case_id.to_string(),
                    test_execution_id: execution.test_execution_id.to_string(),
                    template_identifier: "rpc.artifact-upload".into(),
                    label: "Artifact upload fixture".into(),
                    module: "fixture".into(),
                    status: "passed".into(),
                    phases: vec![],
                    records: vec![],
                    telemetry: vec![],
                    criteria: vec![],
                    failures: vec![],
                }],
            })
        })
    }
}

struct ScopedFixture {
    session: dokime_gui::connection::ConnectionSession,
    server: dokime_server::grpc::Server,
    runtime: dokime::Runtime,
    _root: tempfile::TempDir,
    scope: model::content::wire::ArtifactScope,
    execution: std::sync::Arc<ActiveFixture>,
}

impl ScopedFixture {
    async fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let metadata = || dokime::Metadata::new(dokime::Identifier::new("rpc.artifact-upload").unwrap(), "Artifact upload fixture", None);
        let execution = std::sync::Arc::new(ActiveFixture::default());
        let runtime =
            dokime::Runtime::with_execution_fixture(dokime::RuntimeConfig::new(root.path(), metadata()), "fixture", execution.clone())
                .await
                .unwrap();
        let handle = runtime.handle();
        let template = dokime::definition::TestCaseTemplate {
            test_case_template_id: dokime::model::identity::new_id::<dokime::TemplateId>(),
            metadata: metadata(),
            tags: vec![],
            parameter_groups: vec![],
            parameters: vec![],
            files: vec![],
            document: None,
            phases: vec![],
            criteria: vec![],
            prompt_definitions: vec![],
            subscriptions: vec![],
            attributes: Default::default(),
        };
        let suite = dokime::definition::TestSuite {
            metadata: metadata(),
            tags: vec![],
            parameter_groups: vec![],
            test_case_templates: vec![template],
        };
        handle
            .set_suites(
                "provider:fixture",
                dokime::definition::SuiteSet::new([dokime::domains::TestSuiteDefinition {
                    suite_id: "rpc.artifact-upload".into(),
                    tags: vec![],
                    workflow: Some("execute_cycle".into()),
                    suite,
                    record_contracts: vec![],
                    telemetry_contracts: vec![],
                }]),
            )
            .await
            .unwrap();
        let (cycle_id_typed, executions) = artifact::registered_cycle(&handle, "rpc.artifact-upload", metadata()).await;
        let cycle_id = cycle_id_typed.to_string();
        let execution_id_typed = executions[0].clone();
        let receipt = handle
            .start_execution(execution_id_typed.clone(), dokime::domain::model::execution::ExecutionMode::Automatic, Default::default())
            .await;
        assert_eq!(receipt.status, dokime::runtime::command::CommandStatus::Accepted);
        let mut state = handle.watch_state();
        let server = dokime_server::grpc::Server::start(handle, dokime_server::grpc::Config::local()).await.unwrap();
        let session = dokime_gui::connection::ConnectionSession::default();
        session
            .connect(dokime_gui::connection::ConnectRequest {
                target: dokime_gui::connection::ConnectionTarget::Local { endpoint: server.local_endpoint() },
                secrets: dokime_gui::connection::ConnectionSecrets::None,
                expected_backend_id: None,
            })
            .await
            .unwrap();
        let snapshot = state
            .wait_until(|snapshot| {
                snapshot
                    .state()
                    .executions
                    .get(&execution_id_typed)
                    .is_some_and(|execution| execution.status == dokime::domain::model::execution::result::ExecutionStatus::Running)
            })
            .await
            .expect("the runtime remains live during fixture setup");
        let execution_id = snapshot
            .state()
            .executions
            .values()
            .find(|execution| execution.test_execution_id == execution_id_typed)
            .unwrap()
            .test_execution_id
            .to_string();
        Self {
            session,
            server,
            runtime,
            _root: root,
            scope: model::content::wire::ArtifactScope {
                cycle_id: Some(cycle_id),
                case_id: None,
                execution_id: Some(execution_id),
            },
            execution,
        }
    }

    async fn close(mut self) {
        self.execution.release.notify_one();
        self.session.disconnect().unwrap();
        self.server.shutdown().await.unwrap();
        self.runtime.shutdown().await.unwrap();
    }
}

fn request_bytes(generation: u64, payload: &[u8]) -> Vec<u8> {
    let mut bytes = generation.to_le_bytes().to_vec();
    bytes.extend(Sha256::digest(FILE_DESCRIPTOR_SET));
    bytes.extend_from_slice(payload);
    bytes
}

fn write_payload(id: u64, sequence: u64, slice: &[u8]) -> Vec<u8> {
    let mut payload = id.to_le_bytes().to_vec();
    payload.extend(sequence.to_le_bytes());
    payload.extend_from_slice(slice);
    payload
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn upload_sink_completes() {
    let fixture = ScopedFixture::new().await;
    let (app, window) = app_and_window(&fixture.session);
    let generation_value = generation(&fixture.session);

    let open_bytes = request_bytes(generation_value, &header(6, Some(fixture.scope.clone())).wire_bytes());
    let response = invoke(&window, "artifact_upload_open", open_bytes).expect("open admitted");
    let tauri::ipc::InvokeResponseBody::Raw(handle_bytes) = response else {
        panic!("open replies with a raw stream handle");
    };
    assert_eq!(handle_bytes.len(), 16);
    let id = u64::from_le_bytes(handle_bytes[8..16].try_into().unwrap());

    invoke(&window, "artifact_upload_write", request_bytes(generation_value, &write_payload(id, 1, b"foo"))).expect("first slice accepted");
    invoke(&window, "artifact_upload_write", request_bytes(generation_value, &write_payload(id, 2, b"bar")))
        .expect("second slice accepted");

    let finished = invoke(&window, "artifact_upload_finish", request_bytes(generation_value, &id.to_le_bytes()))
        .expect("finish resolves once the server verifies the digest");
    let tauri::ipc::InvokeResponseBody::Raw(metadata_bytes) = finished else {
        panic!("finish replies with raw metadata bytes");
    };
    let payload = &metadata_bytes[8..];
    let (metadata, admitted_memory) = model::content::wire::ArtifactMetadata::from_wire(payload).unwrap();
    let transfer_bound = 4_194_304 * 8;
    assert!(admitted_memory <= transfer_bound, "the metadata decoding budget stays within the native transfer bound");
    assert!(metadata.memory_size() <= transfer_bound, "the decoded metadata stays within the native transfer bound");
    assert_eq!(metadata.size_bytes, 6);
    assert_eq!(metadata.sha256, Sha256::digest(b"foobar").iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    assert_eq!(metadata.state, model::content::wire::ArtifactState::Complete);

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn upload_refuses_gap() {
    let fixture = Fixture::new().await;
    let (app, window) = app_and_window(&fixture.session);
    let generation_value = generation(&fixture.session);

    let open_bytes = request_bytes(generation_value, &header(3, None).wire_bytes());
    let response = invoke(&window, "artifact_upload_open", open_bytes).expect("open admitted");
    let tauri::ipc::InvokeResponseBody::Raw(handle_bytes) = response else {
        panic!("open replies with a raw stream handle");
    };
    let id = u64::from_le_bytes(handle_bytes[8..16].try_into().unwrap());

    let error = invoke(&window, "artifact_upload_write", request_bytes(generation_value, &write_payload(id, 2, b"foo")))
        .expect_err("a write that skips a sequence number is refused");
    assert_eq!(error["kind"], "invalid_ack");

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn upload_owner_cancels() {
    let fixture = Fixture::new().await;
    let (app, window) = app_and_window(&fixture.session);
    let generation_value = generation(&fixture.session);

    let open_bytes = request_bytes(generation_value, &header(3, None).wire_bytes());
    let response = invoke(&window, "artifact_upload_open", open_bytes).expect("open admitted");
    let tauri::ipc::InvokeResponseBody::Raw(handle_bytes) = response else {
        panic!("open replies with a raw stream handle");
    };
    let id = u64::from_le_bytes(handle_bytes[8..16].try_into().unwrap());

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");

    let error = invoke(&window, "artifact_upload_write", request_bytes(generation_value, &write_payload(id, 1, b"foo")))
        .expect_err("a stream cancelled with its owner is no longer reachable");
    assert_eq!(error["kind"], "unknown_stream");
    fixture.close().await;
}

fn recording_files(dir: &std::path::Path) -> std::collections::HashSet<String> {
    std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|entry| entry.ok()).filter_map(|entry| entry.file_name().into_string().ok()).collect())
        .unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn recording_cancel_deletes() {
    let fixture = Fixture::new().await;
    let (app, window) = app_and_window(&fixture.session);
    let generation_value = generation(&fixture.session);

    let recordings_dir = {
        use tauri::Manager;
        app.path().app_cache_dir().expect("test app cache dir resolves").join("recordings")
    };
    let before = recording_files(&recordings_dir);

    invoke(&window, "recording_open", request_bytes(generation_value, &header(0, None).wire_bytes())).expect("recording sink opens");
    invoke(&window, "recording_write", request_bytes(generation_value, b"fragment")).expect("write accepted");

    let mut created: Vec<_> = recording_files(&recordings_dir).difference(&before).cloned().collect();
    let temp_name = created.pop().expect("recording_open must create exactly one temp file");
    assert!(created.is_empty(), "recording_open must create exactly one temp file");
    let temp_path = recordings_dir.join(temp_name);
    assert!(temp_path.exists(), "the temp file must exist before cancel");

    invoke(&window, "recording_cancel", request_bytes(generation_value, &[])).expect("cancel accepted");

    assert!(!temp_path.exists(), "cancel must delete the sink's temp file");

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}

#[test]
fn upload_sink_opens() {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let fixture = runtime.block_on(Fixture::new());
    let (app, window) = app_and_window(&fixture.session);
    let generation_value: u64 = fixture.session.status().generation.parse().unwrap();
    let bytes = request_bytes(generation_value, &header(1, None).wire_bytes());

    let response = invoke(&window, "artifact_upload_open", bytes).expect("the upload sink opener admits the stream");
    let tauri::ipc::InvokeResponseBody::Raw(handle) = response else {
        panic!("the upload sink opener replies with a raw stream handle");
    };
    assert_eq!(handle.len(), 16);

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}
