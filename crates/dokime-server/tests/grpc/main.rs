/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "authorization.rs"]
pub(crate) mod authorization;
pub(crate) mod evidence;

#[path = "../support/mod.rs"]
pub(crate) mod common;

#[path = "content.rs"]
pub(crate) mod content;

#[path = "diagnostics.rs"]
pub(crate) mod diagnostics;

#[path = "workflow.rs"]
pub(crate) mod workflow;

#[path = "append.rs"]
pub(crate) mod append;

#[path = "catalog.rs"]
pub(crate) mod catalog;

#[path = "planning.rs"]
pub(crate) mod planning;

#[path = "appearance.rs"]
pub(crate) mod appearance;

#[path = "notes.rs"]
pub(crate) mod notes;

#[path = "prompts.rs"]
pub(crate) mod prompts;

#[path = "query.rs"]
pub(crate) mod query;

#[path = "documents.rs"]
pub(crate) mod documents;

#[path = "history.rs"]
pub(crate) mod history;

#[path = "plugins.rs"]
pub(crate) mod plugins;

#[path = "settings.rs"]
pub(crate) mod settings;

#[path = "telemetry.rs"]
pub(crate) mod telemetry;

#[path = "traceability.rs"]
pub(crate) mod traceability;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::time::Duration;
// Third Party Imports
use tokio::sync::Notify;
use tonic::{
    Code, Request,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint},
};
// Local Imports
use dokime_app::{
    ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture, Runtime, RuntimeConfig,
    core::{Identifier, Metadata},
    domains::ExecutionProviderTarget,
    execution::{CycleView, ExecutionStatus},
    transport::{ProviderCycleOutcome, ProviderExecutionOutcome, ProviderStatusOutcome},
};
use dokime_protocol::v1::{self, backend_client::BackendClient};
use dokime_server::grpc::{
    Config, Server, TlsIdentity,
    access::{BearerGrant, Role},
};

/* === Constants === */
const TOKEN: &str = "fixture-token-012345678901234567890123456789";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Gate {
    started: Notify,
    release: Notify,
    outcome: fn(&CycleView) -> ProviderCycleOutcome,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for Gate {
    fn default() -> Self {
        Self::reporting(passed_outcome)
    }
}

impl Gate {
    fn reporting(outcome: fn(&CycleView) -> ProviderCycleOutcome) -> Self {
        Self {
            started: Notify::new(),
            release: Notify::new(),
            outcome,
        }
    }
}

impl ExecutionFixture for Gate {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            self.started.notify_one();
            let released = self.release.notified();
            tokio::pin!(released);
            loop {
                if context.is_cancelled() {
                    return Err(dokime_app::transport::ProviderExecutionFailure::new(
                        "cancel",
                        "fixture acknowledged cancellation",
                    ));
                }
                tokio::select! {
                    _ = &mut released => break,
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {},
                }
            }
            Ok((self.outcome)(cycle))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn runtime(root: &std::path::Path) -> Runtime {
    let config = RuntimeConfig::new(
        root,
        Metadata::new(
            Identifier::new("grpc.fixture").unwrap(),
            "gRPC fixture",
            None,
        ),
    );
    Runtime::start(config).await.unwrap()
}

fn grant(role: Role) -> BearerGrant {
    BearerGrant::new("fixture".into(), role, TOKEN).unwrap()
}

fn auth<T>(message: T, token: &str) -> Request<T> {
    let mut request = Request::new(message);
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    request
}

fn authenticated<T>(message: T) -> Request<T> {
    auth(message, TOKEN)
}

fn passed_outcome(cycle: &CycleView) -> ProviderCycleOutcome {
    ProviderCycleOutcome {
        status: "passed".into(),
        dependency_imports: vec![],
        provider: ProviderStatusOutcome {
            status: "active".into(),
            domain_registration_count: 1,
            test_suite_count: 1,
            test_suite_identifiers: vec!["fixture.suite".into()],
            error: None,
        },
        failures: vec![],
        executions: cycle
            .test_executions
            .iter()
            .filter(|execution| {
                matches!(
                    execution.status,
                    ExecutionStatus::Pending | ExecutionStatus::Running
                )
            })
            .map(|execution| {
                let case = cycle
                    .cases()
                    .find(|case| case.test_case_id == execution.test_case_id)
                    .unwrap();
                ProviderExecutionOutcome {
                    case_id: case.test_case_id.to_string(),
                    test_execution_id: execution.test_execution_id.to_string(),
                    template_identifier: case.metadata.identifier.to_string(),
                    label: case.metadata.label.clone(),
                    module: "grpc.fixture".into(),
                    status: "passed".into(),
                    phases: vec![],
                    records: vec![],
                    telemetry: vec![],
                    criteria: vec![],
                    failures: vec![],
                }
            })
            .collect(),
    }
}

async fn connect(server: &Server) -> BackendClient<Channel> {
    BackendClient::connect(server.local_endpoint())
        .await
        .unwrap()
}

pub(crate) async fn discovery_shares_independently() {
    use harmos::operations::OperationDeclaration;
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let node_id = handle.identity().node.node_id.to_string();
    let epoch = handle.process_epoch().to_string();
    let server = Server::start(handle.clone(), Config::default())
        .await
        .unwrap();
    let address = server.address();
    let mut client = connect(&server).await;
    let identity = client
        .get_identity(v1::GetIdentityRequest {})
        .await
        .unwrap()
        .into_inner();
    assert_eq!(identity.backend_id, node_id);
    assert_eq!(identity.epoch, epoch);
    assert_eq!(identity.display_name, "gRPC fixture");
    assert!(identity.started_at.is_none());
    assert_eq!(identity.api.unwrap().descriptor_sha256.len(), 32);
    let health = client
        .get_health(v1::GetHealthRequest {})
        .await
        .unwrap()
        .into_inner();
    assert_eq!(health.readiness(), v1::Readiness::Ready);
    assert_eq!(health.epoch, epoch);
    let native = dokime::client::Client::connect(
        dokime::client::Target::local(server.local_endpoint()).unwrap(),
        None,
        Some(&node_id),
    )
    .await
    .unwrap();
    let managed = native.health().await.unwrap();
    let managed_identity = native.identity().await.unwrap();
    assert_eq!(managed_identity.backend_id, node_id);
    assert_eq!(managed_identity.epoch, epoch);
    assert_eq!(managed_identity.display_name, identity.display_name);
    assert_eq!(managed_identity.product_version, identity.product_version);
    let operations = native.operations().unwrap();
    assert!(operations.iter().any(|operation| {
        operation.contract.operation().id.as_str() == dokime::operations::runtime::GetIdentity::ID
            && operation.access == dokime::operations::Access::Read
    }));
    let request = dokime::client::operations::Request::from_json(
        dokime::operations::runtime::GetIdentity::ID,
        b"{}",
    )
    .unwrap();
    let response = native.invoke(request).await.unwrap();
    assert_eq!(response["backend_id"], node_id);
    assert_eq!(response["epoch"], epoch);
    assert_eq!(managed.backend_id, node_id);
    assert_eq!(managed.epoch, epoch);
    assert_eq!(
        managed.readiness,
        dokime::operations::runtime::Readiness::Ready
    );
    assert!(managed.observed_at_nanoseconds < 1_000_000_000);
    native.close_and_wait().await;
    let capabilities = client
        .get_capabilities(v1::GetCapabilitiesRequest {})
        .await
        .unwrap()
        .into_inner();
    use prost::Message;
    let descriptor =
        prost_types::FileDescriptorSet::decode(dokime_server::grpc::FILE_DESCRIPTOR_SET).unwrap();
    let expected = descriptor
        .file
        .into_iter()
        .flat_map(|file| {
            let package = file.package.unwrap_or_default();
            file.service.into_iter().flat_map(move |service| {
                let name = format!("/{package}.{}", service.name.unwrap());
                service
                    .method
                    .into_iter()
                    .map(move |method| format!("{name}/{}", method.name.unwrap()))
            })
        })
        .collect::<std::collections::BTreeSet<_>>();
    let advertised = capabilities
        .methods
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        advertised.len(),
        capabilities.methods.len(),
        "duplicate advertised method"
    );
    assert_eq!(advertised, expected);
    assert_eq!(capabilities.limits.unwrap().max_request_bytes, 1024 * 1024);
    tokio::time::timeout(Duration::from_secs(5), server.shutdown())
        .await
        .unwrap()
        .unwrap();
    // Keep the generated client/stream alive throughout listener drain.
    drop(client);
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
    assert!(!handle.is_shutting_down());
    let second = Server::start(handle, Config::default()).await.unwrap();
    assert_eq!(
        connect(&second)
            .await
            .get_identity(v1::GetIdentityRequest {})
            .await
            .unwrap()
            .into_inner()
            .epoch,
        epoch
    );
    second.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
    let mut rebooted = runtime(root.path()).await;
    assert_eq!(
        rebooted.handle().identity().node.node_id.to_string(),
        node_id
    );
    assert_ne!(rebooted.handle().process_epoch().to_string(), epoch);
    rebooted.shutdown().await.unwrap();
}

pub(crate) async fn malformed_credentials_wire() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let config = Config::local()
        .with_bearer_auth(vec![grant(Role::Read)])
        .unwrap();
    let server = Server::start(owner.handle(), config).await.unwrap();
    let mut client = connect(&server).await;
    let mut invalid = Request::new(v1::GetIdentityRequest {});
    invalid.metadata_mut().insert(
        "authorization",
        "Bearer a-different-long-token-01234567890123456789"
            .parse()
            .unwrap(),
    );
    assert_eq!(
        client.get_identity(invalid).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    for authorization in [
        format!("Basic {TOKEN}"),
        "Bearer short".to_owned(),
        format!("Bearer  {TOKEN}"),
    ] {
        let mut malformed = Request::new(v1::GetIdentityRequest {});
        malformed
            .metadata_mut()
            .insert("authorization", authorization.parse().unwrap());
        let error = client.get_identity(malformed).await.unwrap_err();
        assert_eq!(error.code(), Code::Unauthenticated);
        assert_eq!(error.message(), "invalid credentials");
    }
    let mut duplicate = authenticated(v1::GetIdentityRequest {});
    duplicate
        .metadata_mut()
        .append("authorization", format!("Bearer {TOKEN}").parse().unwrap());
    assert_eq!(
        client.get_identity(duplicate).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    client
        .get_identity(authenticated(v1::GetIdentityRequest {}))
        .await
        .unwrap();
    client
        .get_health(authenticated(v1::GetHealthRequest {}))
        .await
        .unwrap();
    client
        .get_capabilities(authenticated(v1::GetCapabilitiesRequest {}))
        .await
        .unwrap();
    client
        .get_diagnostics(authenticated(v1::GetDiagnosticsRequest {}))
        .await
        .unwrap();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn diagnostics_projects_paths() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let epoch = owner.handle().session_context().await.0.to_string();
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut client = connect(&server).await;
    let diagnostics = client
        .get_diagnostics(v1::GetDiagnosticsRequest {})
        .await
        .unwrap()
        .into_inner();
    let backend = diagnostics.server.expect("server diagnostics");
    assert!(!backend.version.is_empty());
    assert_eq!(backend.process.unwrap().pid, std::process::id());
    let runtime = diagnostics.runtime.expect("runtime diagnostics");
    assert_eq!(runtime.session_id, epoch);
    assert_eq!(runtime.storage_root, root.path().to_string_lossy());
    assert_eq!(
        runtime.startup_doctor.unwrap().status(),
        v1::DiagnosticCheckStatus::Ok
    );
    assert!(
        runtime
            .path_conventions
            .iter()
            .any(|path| path.key() == v1::DiagnosticPathKey::HubLog)
    );
    let counts = runtime.counts.expect("diagnostic counts");
    assert_eq!(counts.cycles, 0);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn remote_requires_binding() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let remote = "0.0.0.0:0".parse().unwrap();
    assert!(
        Server::start(owner.handle(), Config::local().with_bind_addr(remote))
            .await
            .is_err()
    );
    assert!(Config::remote(remote, TlsIdentity::from_pem(vec![], vec![]), vec![]).is_err());
    let invalid = Config::remote(
        remote,
        TlsIdentity::from_pem(vec![], vec![]),
        vec![grant(Role::Admin)],
    )
    .unwrap();
    assert!(Server::start(owner.handle(), invalid).await.is_err());
    assert!(!owner.handle().is_shutting_down());
    owner.shutdown().await.unwrap();
}

pub(crate) async fn tls_listener_clients() {
    // A throwaway localhost identity generated for this run; nothing is committed.
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate = generated.cert.pem().into_bytes();
    let key = generated.signing_key.serialize_pem().into_bytes();
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let config = Config::remote(
        "0.0.0.0:0".parse().unwrap(),
        TlsIdentity::from_pem(certificate.clone(), key),
        vec![grant(Role::Read)],
    )
    .unwrap();
    assert!(!format!("{config:?}").contains(TOKEN));
    assert!(!format!("{config:?}").contains("PRIVATE KEY"));
    let server = Server::start(owner.handle(), config).await.unwrap();
    let endpoint = Endpoint::from_shared(server.local_endpoint()).unwrap();
    assert!(
        endpoint.clone().connect().await.is_err(),
        "untrusted certificate must be rejected"
    );
    let stream = endpoint
        .tls_config(
            ClientTlsConfig::new()
                .domain_name("localhost")
                .ca_certificate(Certificate::from_pem(&certificate)),
        )
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = BackendClient::new(stream);
    assert_eq!(
        client
            .get_identity(v1::GetIdentityRequest {})
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    client
        .get_identity(authenticated(v1::GetIdentityRequest {}))
        .await
        .unwrap();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let _ = multithread;
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn catalog_contract() {
    {
        eprintln!("Scenario: catalog::schemas_exact_pages");
        run_scenario(catalog::schemas_exact_pages, false, None, false);
    }
    {
        eprintln!("Scenario: catalog::catalog_suite_extensions");
        run_scenario(catalog::catalog_suite_extensions, false, None, false);
    }
    {
        eprintln!("Scenario: plugins::plugin_stage_identity");
        run_scenario(plugins::plugin_stage_identity, false, None, false);
    }
    {
        eprintln!("Scenario: plugins::invalid_uploads_mutate");
        run_scenario(plugins::invalid_uploads_mutate, false, None, false);
    }
    {
        eprintln!("Scenario: plugins::listener_shutdown_admission");
        run_scenario(plugins::listener_shutdown_admission, false, None, false);
    }
    {
        eprintln!("Scenario: plugins::oversized_permissions_consent");
        run_scenario(plugins::oversized_permissions_consent, false, None, false);
    }
    {
        eprintln!("Scenario: settings::frame_readable_probe");
        run_scenario(settings::frame_readable_probe, false, None, false);
    }
    {
        eprintln!("Scenario: settings::storage_sources_admin");
        run_scenario(settings::storage_sources_admin, false, None, false);
    }
    {
        eprintln!("Scenario: settings::plugin_settings_admin");
        run_scenario(settings::plugin_settings_admin, false, None, false);
    }
    {
        eprintln!("Scenario: settings::backend_policy_revision");
        run_scenario(settings::backend_policy_revision, false, None, false);
    }
    {
        eprintln!("Scenario: settings::app_settings_role");
        run_scenario(settings::app_settings_role, false, None, false);
    }
}

#[test]
fn evidence_contract() {
    {
        eprintln!("Scenario: content::artifact_transfer_listener");
        run_scenario(content::artifact_transfer_listener, false, None, false);
    }
    {
        eprintln!("Scenario: content::artifact_upload_owned");
        run_scenario(content::artifact_upload_owned, false, None, false);
    }
    {
        eprintln!("Scenario: content::artifact_upload_digest");
        run_scenario(content::artifact_upload_digest, false, None, false);
    }
    {
        eprintln!("Scenario: content::artifact_upload_match");
        run_scenario(content::artifact_upload_match, false, None, false);
    }
    {
        eprintln!("Scenario: content::artifact_upload_completion");
        run_scenario(content::artifact_upload_completion, false, None, false);
    }
    {
        eprintln!("Scenario: content::large_artifact_buffering");
        run_scenario(content::large_artifact_buffering, false, None, false);
    }
    {
        eprintln!("Scenario: documents::current_document_rows");
        run_scenario(documents::current_document_rows, false, None, false);
    }
    {
        eprintln!("Scenario: documents::summary_complete_telemetry");
        run_scenario(documents::summary_complete_telemetry, false, None, false);
    }
    {
        eprintln!("Scenario: documents::duration_x_axis");
        run_scenario(documents::duration_x_axis, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::search::cycle_search_once");
        run_scenario(workflow::search::cycle_search_once, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::search::listed_cases_page");
        run_scenario(workflow::search::listed_cases_page, false, None, false);
    }
    {
        eprintln!("Scenario: telemetry::telemetry_pages_window");
        run_scenario(telemetry::telemetry_pages_window, false, None, false);
    }
    {
        eprintln!("Scenario: traceability::get_traceability_cursor");
        run_scenario(traceability::get_traceability_cursor, false, None, false);
    }
    {
        eprintln!("Scenario: traceability::traceability_bounds_clone");
        run_scenario(traceability::traceability_bounds_clone, false, None, false);
    }
}

#[test]
fn execution_contract() {
    if std::env::var_os(workflow::requests::CRASH_ROOT).is_some() {
        run_scenario(workflow::requests::crashed_run_replay, false, None, false);
        return;
    }
    {
        eprintln!("Scenario: planning::ordered_selections_provider");
        run_scenario(planning::ordered_selections_provider, false, None, false);
    }
    {
        eprintln!("Scenario: planning::planning_refuses_inputs");
        run_scenario(planning::planning_refuses_inputs, false, None, false);
    }
    {
        eprintln!("Scenario: planning::template_conflicts_disambiguate");
        run_scenario(
            planning::template_conflicts_disambiguate,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: planning::effective_native_materialization");
        run_scenario(
            planning::effective_native_materialization,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: planning::payload_budget_allocation");
        run_scenario(planning::payload_budget_allocation, false, None, false);
    }
    {
        eprintln!("Scenario: planning::registered_cases_selection");
        run_scenario(planning::registered_cases_selection, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::requests::commands_owner_receipts");
        run_scenario(
            workflow::requests::commands_owner_receipts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: workflow::requests::keyed_runs_restart");
        run_scenario(workflow::requests::keyed_runs_restart, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::requests::invalid_run_admission");
        run_scenario(
            workflow::requests::invalid_run_admission,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: workflow::requests::crashed_run_replay");
        run_scenario(workflow::requests::crashed_run_replay, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::registered_create_definitions");
        run_scenario(workflow::registered_create_definitions, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::stopping_running_unwind");
        run_scenario(workflow::stopping_running_unwind, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::runtime_shell_cursor");
        run_scenario(workflow::runtime_shell_cursor, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::checkpoint_telemetry_queries");
        run_scenario(workflow::checkpoint_telemetry_queries, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::historical_checkpoint_content");
        run_scenario(workflow::historical_checkpoint_content, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::read_role_admit");
        run_scenario(workflow::read_role_admit, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::stalled_stream_execution");
        run_scenario(workflow::stalled_stream_execution, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::run_cycle_targets");
        run_scenario(workflow::run_cycle_targets, false, None, false);
    }
}

#[test]
fn protocol_contract() {
    {
        eprintln!("Scenario: appearance::appearance_asset_bound");
        run_scenario(appearance::appearance_asset_bound, false, None, false);
    }
    {
        eprintln!("Scenario: appearance::appearance_refuses_transfer");
        run_scenario(appearance::appearance_refuses_transfer, false, None, false);
    }
    {
        eprintln!("Scenario: append::append_cases_control");
        run_scenario(append::append_cases_control, false, None, false);
    }
    {
        eprintln!("Scenario: authorization::table_covers_serves");
        authorization::table_covers_serves();
    }
    {
        eprintln!("Scenario: authorization::permission_matrix_enforcement");
        run_scenario(
            authorization::permission_matrix_enforcement,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: workflow::controls::cycle_edits_state");
        run_scenario(workflow::controls::cycle_edits_state, false, None, false);
    }
    {
        eprintln!("Scenario: diagnostics::collected_archive_material");
        run_scenario(diagnostics::collected_archive_material, false, None, false);
    }
    {
        eprintln!("Scenario: diagnostics::log_stream_file");
        run_scenario(diagnostics::log_stream_file, false, None, false);
    }
    {
        eprintln!("Scenario: diagnostics::level_filter_stream");
        run_scenario(diagnostics::level_filter_stream, false, None, false);
    }
    {
        eprintln!("Scenario: diagnostics::missing_file_live");
        run_scenario(diagnostics::missing_file_live, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::documents_execution::running_execution_owners");
        run_scenario(
            workflow::documents_execution::running_execution_owners,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::fixed_pages_gap");
        run_scenario(evidence::fixed_pages_gap, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::filtered_scans_resync");
        run_scenario(evidence::filtered_scans_resync, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::capture_cursors_restarts");
        run_scenario(evidence::capture_cursors_restarts, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::oversized_delivery_evidence");
        run_scenario(evidence::oversized_delivery_evidence, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::files::live_file_rewrites");
        run_scenario(workflow::files::live_file_rewrites, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::files::execution_file_buffer");
        run_scenario(workflow::files::execution_file_buffer, false, None, false);
    }
    {
        eprintln!("Scenario: history::deletion_validates_unsupported");
        run_scenario(history::deletion_validates_unsupported, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::inspection::inspection_separates_outputs");
        run_scenario(
            workflow::inspection::inspection_separates_outputs,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: workflow::inspection::planned_params_limit");
        run_scenario(
            workflow::inspection::planned_params_limit,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: discovery_shares_independently");
        run_scenario(discovery_shares_independently, false, None, false);
    }
    {
        eprintln!("Scenario: malformed_credentials_wire");
        run_scenario(malformed_credentials_wire, false, None, false);
    }
    {
        eprintln!("Scenario: diagnostics_projects_paths");
        run_scenario(diagnostics_projects_paths, false, None, false);
    }
    {
        eprintln!("Scenario: remote_requires_binding");
        run_scenario(remote_requires_binding, false, None, false);
    }
    {
        eprintln!("Scenario: tls_listener_clients");
        run_scenario(tls_listener_clients, false, None, false);
    }
    {
        eprintln!("Scenario: notes::scoped_annotation_workflow");
        run_scenario(notes::scoped_annotation_workflow, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::pagination::large_cycles_order");
        run_scenario(workflow::pagination::large_cycles_order, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::pagination::large_run_preparation");
        run_scenario(
            workflow::pagination::large_run_preparation,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: prompts::prompt_answers_identity");
        run_scenario(prompts::prompt_answers_identity, false, None, false);
    }
    {
        eprintln!("Scenario: prompts::prompt_condition_commit");
        run_scenario(prompts::prompt_condition_commit, false, None, false);
    }
    {
        eprintln!("Scenario: query::aggregate_answers_asked");
        run_scenario(query::aggregate_answers_asked, false, None, false);
    }
    {
        eprintln!("Scenario: query::unknown_column_exist");
        run_scenario(query::unknown_column_exist, false, None, false);
    }
    {
        eprintln!("Scenario: query::unspecified_source_read");
        run_scenario(query::unspecified_source_read, false, None, false);
    }
    {
        eprintln!("Scenario: query::detailed_status_directory");
        run_scenario(query::detailed_status_directory, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::steps::step_pages_selections");
        run_scenario(workflow::steps::step_pages_selections, false, None, false);
    }
}

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
