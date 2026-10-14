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
use harmos::{MemorySize, Protobuf};
use sha2::{Digest, Sha256};

// Local Imports
use super::*;
use dokime_contract::operations::content;
use dokime_gui::rpc::{Bridge, FILE_DESCRIPTOR_SET, MAX_FRAME_BYTES, handler};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

fn generation(session: &dokime_gui::connection::ConnectionSession) -> u64 {
    session
        .status()
        .generation
        .parse()
        .expect("fixture generation is numeric")
}

fn header(size_bytes: u64, scope: Option<content::ArtifactScope>) -> content::UploadArtifactHeader {
    content::UploadArtifactHeader {
        scope,
        filename: "upload-sink-fixture.bin".into(),
        content_type: Some("application/octet-stream".into()),
        size_bytes,
        sha256: None,
    }
}

fn app_and_window(
    session: &dokime_gui::connection::ConnectionSession,
) -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::WebviewWindow<tauri::test::MockRuntime>,
) {
    let app = tauri::test::mock_builder()
        .manage(session.clone())
        .manage(Bridge::default())
        .invoke_handler(handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    (app, window)
}

#[derive(Default)]
struct ActiveFixture {
    release: tokio::sync::Notify,
}
impl dokime_app::ExecutionFixture for ActiveFixture {
    fn run_cycle<'a>(
        &'a self,
        _context: dokime_app::ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
        _target: &'a dokime::domains::ExecutionProviderTarget,
    ) -> dokime_app::ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            use dokime_app::transport::{
                ProviderCycleOutcome, ProviderExecutionOutcome, ProviderStatusOutcome,
            };
            self.release.notified().await;
            let execution = &cycle.test_executions[0];
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
    runtime: dokime_app::Runtime,
    _root: tempfile::TempDir,
    scope: content::ArtifactScope,
    execution: std::sync::Arc<ActiveFixture>,
}

impl ScopedFixture {
    async fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let metadata = || {
            dokime_app::core::Metadata::new(
                dokime_app::core::Identifier::new("rpc.artifact-upload").unwrap(),
                "Artifact upload fixture",
                None,
            )
        };
        let execution = std::sync::Arc::new(ActiveFixture::default());
        let runtime = dokime_app::Runtime::with_execution_fixture(
            dokime_app::RuntimeConfig::new(root.path(), metadata()),
            "fixture",
            execution.clone(),
        )
        .await
        .unwrap();
        let handle = runtime.handle();
        let template = dokime_app::definition::TestCaseTemplate {
            test_case_template_id: dokime_app::core::TemplateId::new(),
            metadata: metadata(),
            tags: vec![],
            parameter_groups: vec![],
            parameters: vec![],
            files: vec![],
            document: None,
            phases: vec![],
            criteria: vec![],
            continue_on_failure: false,
            prompt_definitions: vec![],
            subscriptions: vec![],
            attributes: Default::default(),
        };
        let suite = dokime_app::definition::TestSuite {
            metadata: metadata(),
            tags: vec![],
            parameter_groups: vec![],
            test_case_templates: vec![template],
        };
        handle
            .set_suites(
                "provider:fixture",
                dokime_app::definition::SuiteSet::new([dokime::domains::TestSuiteDefinition {
                    suite_id: "rpc.artifact-upload".into(),
                    product: None,
                    tags: vec![],
                    workflow: Some("execute_cycle".into()),
                    suite,
                    record_contracts: vec![],
                    telemetry_contracts: vec![],
                }]),
            )
            .await
            .unwrap();
        let cycle_id_typed = handle
            .start_registered_cycle("rpc.artifact-upload", metadata(), false, Default::default())
            .await
            .unwrap();
        let cycle_id = cycle_id_typed.to_string();
        handle
            .run_cycle_cases(cycle_id_typed, Default::default())
            .await;
        let mut state = handle.watch_state();
        let server = dokime_server::grpc::Server::start(
            handle,
            dokime_server::grpc::Config::local()
                .with_bearer_auth(vec![
                    dokime_server::grpc::access::BearerGrant::new(
                        "native-gui".into(),
                        dokime_server::grpc::access::Role::Control,
                        "artifact-upload-fixture-token-01234567890123",
                    )
                    .unwrap(),
                ])
                .unwrap(),
        )
        .await
        .unwrap();
        let session = dokime_gui::connection::ConnectionSession::default();
        session
            .connect(dokime_gui::connection::ConnectRequest {
                target: dokime_gui::connection::ConnectionTarget::Local {
                    endpoint: server.local_endpoint(),
                },
                credentials: dokime_gui::connection::ConnectionCredentials::Bearer {
                    token: "artifact-upload-fixture-token-01234567890123".into(),
                    remember: false,
                },
                expected_backend_id: None,
            })
            .await
            .unwrap();
        let snapshot = tokio::time::timeout(
            Duration::from_secs(5),
            state.wait_until(|snapshot| {
                snapshot
                    .state()
                    .executions
                    .values()
                    .any(|execution| execution.test_cycle_id.as_str() == cycle_id)
            }),
        )
        .await
        .expect("a started cycle plans at least one execution")
        .expect("the runtime remains live during fixture setup");
        let execution_id = snapshot
            .state()
            .executions
            .values()
            .find(|execution| execution.test_cycle_id.as_str() == cycle_id)
            .unwrap()
            .test_execution_id
            .to_string();
        Self {
            session,
            server,
            runtime,
            _root: root,
            scope: content::ArtifactScope {
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

pub(crate) async fn upload_sink_completion() {
    let fixture = ScopedFixture::new().await;
    let (app, window) = app_and_window(&fixture.session);
    let generation_value = generation(&fixture.session);

    let open_bytes = request_bytes(
        generation_value,
        &header(6, Some(fixture.scope.clone())).encode_to_vec(),
    );
    let response = invoke(&window, "artifact_upload_open", open_bytes).expect("open admitted");
    let tauri::ipc::InvokeResponseBody::Raw(handle_bytes) = response else {
        panic!("open replies with a raw stream handle");
    };
    assert_eq!(handle_bytes.len(), 16);
    let id = u64::from_le_bytes(handle_bytes[8..16].try_into().unwrap());

    invoke(
        &window,
        "artifact_upload_write",
        request_bytes(generation_value, &write_payload(id, 1, b"foo")),
    )
    .expect("first slice accepted");
    invoke(
        &window,
        "artifact_upload_write",
        request_bytes(generation_value, &write_payload(id, 2, b"bar")),
    )
    .expect("second slice accepted");

    let finished = invoke(
        &window,
        "artifact_upload_finish",
        request_bytes(generation_value, &id.to_le_bytes()),
    )
    .expect("finish resolves once the server verifies the digest");
    let tauri::ipc::InvokeResponseBody::Raw(metadata_bytes) = finished else {
        panic!("finish replies with raw metadata bytes");
    };
    let payload = &metadata_bytes[8..];
    assert!(
        content::ArtifactMetadata::decoding_memory(payload).unwrap() <= MAX_FRAME_BYTES * 8,
        "the metadata decoding budget stays within the native transfer bound"
    );
    let metadata = content::ArtifactMetadata::decode(payload).unwrap();
    assert!(
        metadata.memory_size() <= MAX_FRAME_BYTES * 8,
        "the decoded metadata stays within the native transfer bound"
    );
    assert_eq!(metadata.size_bytes, 6);
    assert_eq!(
        metadata.sha256,
        Sha256::digest(b"foobar")
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert_eq!(metadata.state, content::ArtifactState::Complete);

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}

pub(crate) async fn upload_sink_sequence() {
    let fixture = super::Fixture::new().await;
    let (app, window) = app_and_window(&fixture.session);
    let generation_value = generation(&fixture.session);

    let open_bytes = request_bytes(generation_value, &header(3, None).encode_to_vec());
    let response = invoke(&window, "artifact_upload_open", open_bytes).expect("open admitted");
    let tauri::ipc::InvokeResponseBody::Raw(handle_bytes) = response else {
        panic!("open replies with a raw stream handle");
    };
    let id = u64::from_le_bytes(handle_bytes[8..16].try_into().unwrap());

    let error = invoke(
        &window,
        "artifact_upload_write",
        request_bytes(generation_value, &write_payload(id, 2, b"foo")),
    )
    .expect_err("a write that skips a sequence number is refused");
    assert_eq!(error["kind"], "invalid_ack");

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}

pub(crate) async fn upload_sink_owner() {
    let fixture = super::Fixture::new().await;
    let (app, window) = app_and_window(&fixture.session);
    let generation_value = generation(&fixture.session);

    let open_bytes = request_bytes(generation_value, &header(3, None).encode_to_vec());
    let response = invoke(&window, "artifact_upload_open", open_bytes).expect("open admitted");
    let tauri::ipc::InvokeResponseBody::Raw(handle_bytes) = response else {
        panic!("open replies with a raw stream handle");
    };
    let id = u64::from_le_bytes(handle_bytes[8..16].try_into().unwrap());

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");

    let error = invoke(
        &window,
        "artifact_upload_write",
        request_bytes(generation_value, &write_payload(id, 1, b"foo")),
    )
    .expect_err("a stream cancelled with its owner is no longer reachable");
    assert_eq!(error["kind"], "unknown_stream");
    fixture.close().await;
}

fn recording_files(dir: &std::path::Path) -> std::collections::HashSet<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) async fn recording_cancel_file() {
    let fixture = super::Fixture::new().await;
    let (app, window) = app_and_window(&fixture.session);
    let generation_value = generation(&fixture.session);

    let recordings_dir = {
        use tauri::Manager;
        app.path()
            .app_cache_dir()
            .expect("test app cache dir resolves")
            .join("recordings")
    };
    let before = recording_files(&recordings_dir);

    invoke(
        &window,
        "recording_open",
        request_bytes(generation_value, &header(0, None).encode_to_vec()),
    )
    .expect("recording sink opens");
    invoke(
        &window,
        "recording_write",
        request_bytes(generation_value, b"fragment"),
    )
    .expect("write accepted");

    let mut created: Vec<_> = recording_files(&recordings_dir)
        .difference(&before)
        .cloned()
        .collect();
    let temp_name = created
        .pop()
        .expect("recording_open must create exactly one temp file");
    assert!(
        created.is_empty(),
        "recording_open must create exactly one temp file"
    );
    let temp_path = recordings_dir.join(temp_name);
    assert!(temp_path.exists(), "the temp file must exist before cancel");

    invoke(
        &window,
        "recording_cancel",
        request_bytes(generation_value, &[]),
    )
    .expect("cancel accepted");

    assert!(
        !temp_path.exists(),
        "cancel must delete the sink's temp file"
    );

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}

pub(crate) fn upload_sink_runtime() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let fixture = runtime.block_on(super::Fixture::new());
    let (app, window) = app_and_window(&fixture.session);
    let generation_value: u64 = fixture.session.status().generation.parse().unwrap();
    let bytes = request_bytes(generation_value, &header(1, None).encode_to_vec());

    let response = invoke(&window, "artifact_upload_open", bytes)
        .expect("the upload sink opener admits the stream");
    let tauri::ipc::InvokeResponseBody::Raw(handle) = response else {
        panic!("the upload sink opener replies with a raw stream handle");
    };
    assert_eq!(handle.len(), 16);

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}
