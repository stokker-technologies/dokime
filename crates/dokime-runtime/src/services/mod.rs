/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

mod access_credentials;
mod aggregates;
mod analyses;
mod backend;
mod backend_overview;
mod bundles;
mod cases;
mod catalog_prompt_definitions;
mod catalog_record_schemas;
mod catalog_stream_schemas;
mod catalog_suites;
mod clock;
mod composition;
mod content_artifacts;
mod content_execution_files;
mod cycles;
mod documents;
mod evidence;
mod evidence_health_events;
mod evidence_notes;
mod evidence_records;
mod evidence_samples;
mod evidence_traceability;
mod executions;
mod executions_commands;
mod executions_progress;
mod executions_schemas;
mod files;
mod files_uploads;
mod media_captures;
mod plots_settings;
mod plots_snapshots;
mod prompts;
mod prompts_answers;
mod reports;
mod secrets;
mod settings;
mod state;
mod status;
mod storage_sources;

/* === Re-Exports === */

pub use clock::WallClock;
pub use dokime_operations::host::ClientSettings;

/* === Imports === */

// Standard Library Imports
use std::{net::SocketAddr, path::Path, sync::Arc};

// Third Party Imports
use harmos_runtime::{grpc::GrpcManager, services::status::Status};

// Local Imports
use crate::{Error, Host, HostSettings, Result};
use dokime_operations::host::Context;
use dokime_operations::host::HostIdentity;
use dokime_protocol::host::RemoteListenerSettings;
use dokime_protocol::host::SettingsGetResponse;
use dokime_protocol::host::aggregates_server::AggregatesServer;
use dokime_protocol::host::analyses_server::AnalysesServer;
use dokime_protocol::host::backend::overview_server::OverviewServer;
use dokime_protocol::host::backend_server::BackendServer;
use dokime_protocol::host::cases_server::CasesServer;
use dokime_protocol::host::content::artifacts_server::ArtifactsServer;
use dokime_protocol::host::content::execution_files_server::ExecutionFilesServer;
use dokime_protocol::host::cycles_server::CyclesServer;
use dokime_protocol::host::documents_server::DocumentsServer;
use dokime_protocol::host::evidence::health_events_server::HealthEventsServer;
use dokime_protocol::host::evidence::notes_server::NotesServer;
use dokime_protocol::host::evidence::records_server::RecordsServer;
use dokime_protocol::host::evidence::samples_server::SamplesServer;
use dokime_protocol::host::evidence::traceability_server::TraceabilityServer;
use dokime_protocol::host::evidence_server::EvidenceServer;
use dokime_protocol::host::executions::commands_server::CommandsServer;
use dokime_protocol::host::executions::progress_server::ProgressServer;
use dokime_protocol::host::executions::schemas_server::SchemasServer;
use dokime_protocol::host::executions_server::ExecutionsServer;
use dokime_protocol::host::files::uploads_server::UploadsServer;
use dokime_protocol::host::files_server::FilesServer;
use dokime_protocol::host::media::captures_server::CapturesServer;
use dokime_protocol::host::plots::snapshots_server::SnapshotsServer;
use dokime_protocol::host::prompt_definitions_server::PromptDefinitionsServer;
use dokime_protocol::host::prompts::answers_server::AnswersServer;
use dokime_protocol::host::prompts_server::PromptsServer;
use dokime_protocol::host::record_schemas_server::RecordSchemasServer;
use dokime_protocol::host::reports_server::ReportsServer;
use dokime_protocol::host::secrets_server::SecretsServer;
use dokime_protocol::host::storage::sources_server::SourcesServer;
use dokime_protocol::host::stream_schemas_server::StreamSchemasServer;
use dokime_protocol::host::suites_server::SuitesServer;
use dokime_storage::Store;
use prost_types::Timestamp;
use std::fs::OpenOptions;
use std::fs::TryLockError;
use std::num::NonZeroUsize;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
struct Services {
    context: Arc<Context>,
}
pub struct Server {
    endpoint: SocketAddr,
    host: Host,
    runtime: Arc<harmos_runtime::Runtime<composition::Application>>,
    listener: Option<harmos_runtime::grpc::Attached>,
    close_timeout: Duration,
    status: Arc<harmos_runtime::services::status::Status>,
    status_task: Option<JoinHandle<Result<()>>>,
    plugin_task: Option<JoinHandle<Result<()>>>,
    plugin_observation: tokio_util::sync::CancellationToken,
    _status_registration: Arc<harmos_runtime::services::status::Registration>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Server {
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    pub fn status(&self) -> &Arc<Status> {
        &self.status
    }
    pub async fn close(mut self) -> Result<()> {
        self.plugin_observation.cancel();
        if let Some(task) = self.plugin_task.take() {
            task.await??;
        }
        self.host.shutdown().await?;
        if let Some(task) = self.status_task.take() {
            task.await??;
        }
        if let Some(listener) = self.listener.take() {
            listener.detach().await?;
        }
        self.runtime.shutdown_with_timeout(self.close_timeout).await?;
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.plugin_observation.cancel();
        if let Some(task) = self.plugin_task.take() {
            task.abort();
        }
        if let Some(task) = self.status_task.take() {
            task.abort();
        }
    }
}

#[tonic::async_trait]
impl dokime_protocol::host::effects_server::Effects for Services {
    async fn create(
        &self, _request: tonic::Request<dokime_protocol::host::EffectsCreateRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::EffectsCreateResponse>, tonic::Status> {
        Err(dokime_operations::caller_status(harmos_runtime::Failure::wrong_caller("dokime.host.v1.Effects/Create")))
    }
    async fn get(
        &self, _request: tonic::Request<dokime_protocol::host::EffectsGetRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::EffectsGetResponse>, tonic::Status> {
        Err(dokime_operations::caller_status(harmos_runtime::Failure::wrong_caller("dokime.host.v1.Effects/Get")))
    }
    async fn set(
        &self, _request: tonic::Request<dokime_protocol::host::EffectsSetRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::EffectsSetResponse>, tonic::Status> {
        Err(dokime_operations::caller_status(harmos_runtime::Failure::wrong_caller("dokime.host.v1.Effects/Set")))
    }
}

#[tonic::async_trait]
impl dokime_protocol::host::evidence::logs_server::Logs for Services {
    async fn append(
        &self, _request: tonic::Request<dokime_protocol::host::evidence::LogsAppendRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::evidence::LogsAppendResponse>, tonic::Status> {
        Err(dokime_operations::caller_status(harmos_runtime::Failure::wrong_caller("dokime.host.v1.evidence.Logs/Append")))
    }
}

#[tonic::async_trait]
impl dokime_protocol::host::files::captures_server::Captures for Services {
    async fn start(
        &self, _request: tonic::Request<dokime_protocol::host::files::CapturesStartRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::files::CapturesStartResponse>, tonic::Status> {
        Err(dokime_operations::caller_status(harmos_runtime::Failure::wrong_caller("dokime.host.v1.files.Captures/Start")))
    }
    async fn stop(
        &self, _request: tonic::Request<dokime_protocol::host::files::CapturesStopRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::files::CapturesStopResponse>, tonic::Status> {
        Err(dokime_operations::caller_status(harmos_runtime::Failure::wrong_caller("dokime.host.v1.files.Captures/Stop")))
    }
}

#[tonic::async_trait]
impl dokime_protocol::host::plugins::secrets_server::Secrets for Services {
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::plugins::SecretsListRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::plugins::SecretsListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.plugins.Secrets/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::plugins::secrets::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::host::plugins::SecretsSetRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::plugins::SecretsSetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.plugins.Secrets/Set", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::plugins::secrets::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn delete(
        &self, request: tonic::Request<dokime_protocol::host::plugins::SecretsDeleteRequest>,
    ) -> std::result::Result<tonic::Response<dokime_protocol::host::plugins::SecretsDeleteResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.plugins.Secrets/Delete", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::plugins::secrets::delete::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/// Compose Declared Defaults Without Opening Storage, Credentials or Transports.
pub fn initial_preferences() -> SettingsGetResponse {
    let mut snapshot = dokime_operations::host::settings::get::initial_preferences();
    let listener = composition::initial_listener();
    snapshot.value.as_mut().expect("declared application settings").listener = Some(RemoteListenerSettings {
        enabled: listener.enabled,
        address: listener.endpoint,
        certificate_names: listener.certificate_names,
        tls_enabled: true,
        authentication_enabled: true,
    });
    snapshot
}

/// Remove Protected Values Before Explicitly Discarding a Storage Root.
/// The Host Must Be Stopped; Evidence and the Root Itself Are Retained.
pub fn remove_credentials(root: &Path) -> Result<()> {
    if !root.join("backend-id").is_file() {
        return Ok(());
    }
    let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("local.lock"))?;
    match lease.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Err(Error::OccupiedRoot(root.to_owned())),
        Err(TryLockError::Error(error)) => return Err(error.into()),
    }
    let host_id = std::fs::read_to_string(root.join("backend-id"))?;
    let storage = Store::open(root)?;
    dokime_operations::host::remove_root_secrets(&storage)?;
    access_credentials::Store::new(storage, host_id).remove()?;
    Ok(())
}

pub async fn serve(endpoint: SocketAddr, root: &Path, host_settings: HostSettings, settings: ClientSettings) -> Result<Server> {
    if !endpoint.ip().is_loopback() {
        return Err(Error::Invalid("Use the Authenticated Remote Listener for Non-Loopback Access".into()));
    }
    if settings.read_timeout.is_zero()
        || settings.mutation_timeout.is_zero()
        || settings.watch_timeout.is_zero()
        || settings.max_cases == 0
        || settings.page_items == 0
        || settings.stream_buffer == 0
        || settings.message_bytes < 1024
    {
        return Err(Error::Invalid("Serve with Positive Deadlines and Bounds, and at Least 1024 Message Bytes".into()));
    }
    let clock = host_settings.clock.clone();
    let host = Host::open(root, host_settings)?;
    let backend_id = host.store.identity().to_owned();
    let identity = HostIdentity {
        backend_id: backend_id.clone(),
        epoch: uuid::Uuid::now_v7().to_string(),
        product_version: env!("CARGO_PKG_VERSION").into(),
        target: bundles::native_target().into(),
        started_at: Some(Timestamp {
            seconds: host.now_ns().div_euclid(1_000_000_000),
            nanos: host.now_ns().rem_euclid(1_000_000_000) as i32,
        }),
        display_name: "Dokime Execution Host".into(),
    };
    let runtime = composition::launch(clock, &host.store, root, &host.settings.placement, &settings).await?;
    let status = runtime.services().configure_status(status::settings(&settings, backend_id.clone()))?;
    let administration = Arc::new(Mutex::new(()));
    let catalogs = Arc::new(std::sync::RwLock::new(Vec::new()));
    let state = state::State {
        status: status.clone(),
        root: root.to_owned(),
        host: host.clone(),
        operations: runtime.operations.clone(),
        catalogs: catalogs.clone(),
        render_settings: crate::RenderSettings {
            page_size: NonZeroUsize::new(settings.page_items).unwrap(),
            display_points: NonZeroUsize::new(settings.page_items).unwrap(),
        },
        identity,
        temporary_report_limit: settings.max_cases,
    };
    let close_timeout = settings.mutation_timeout;
    let context = Arc::new(Context {
        administration,
        host: Arc::new(state),
        settings: settings.clone(),
        operations: runtime.operations.clone(),
        caller: None,
    });
    host.executions.bind_context(&context)?;
    context.host.refresh_plugin_states().await?;
    dokime_operations::host::plugins::secrets::reconcile(&context).await?;
    dokime_operations::host::register_redactions(&context)?;
    let registration = Arc::new(
        status
            .contribute(Arc::new(status::Facts {
                host: Arc::downgrade(&context.host),
                plugins: catalogs,
            }))
            .map_err(|source| status::failure("Contribute Host Status", source))?,
    );
    let mut changes = context.host.changes();
    let weak_host = Arc::downgrade(&context.host);
    let services = Services { context };
    let plugin_context = Arc::downgrade(&services.context);
    register_services(runtime.grpc(), services.clone())?;
    runtime
        .grpc()
        .configure_access(composition::access_settings(&settings), Arc::new(access_credentials::Store::new(host.store.clone(), backend_id)))
        .await?;
    let mut transport = composition::transport(&settings);
    transport.address = endpoint;
    let listener = runtime.grpc().attach(transport).await?;
    let endpoint = listener.local_addr();
    let plugin_observation = tokio_util::sync::CancellationToken::new();
    let plugin_task =
        tokio::spawn(observe_plugins(runtime.operations.clone(), plugin_context, settings.clone(), plugin_observation.clone()));
    let changed_registration = registration.clone();
    let status_task = tokio::spawn(async move {
        while changes.changed().await.is_ok() {
            changed_registration.changed().map_err(|source| status::failure("Refresh Host Status", source))?;
            if weak_host.upgrade().is_none_or(|host| host.shutting_down()) {
                break;
            }
        }
        Ok(())
    });
    Ok(Server {
        endpoint,
        host,
        runtime,
        listener: Some(listener),
        close_timeout,
        status,
        status_task: Some(status_task),
        plugin_task: Some(plugin_task),
        plugin_observation,
        _status_registration: registration,
    })
}

/* === Private Functions === */

async fn observe_plugins(
    operations: harmos_runtime::operations::OperationManager, context: std::sync::Weak<Context>, settings: ClientSettings,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<()> {
    use harmos_protocol::host::{PluginsWatchRequest, plugins_watch_response::Event};
    use harmos_runtime::builtins::plugins::Watch;
    use harmos_runtime::operations::StreamSettings;
    loop {
        let mut watch = operations.download_protobuf::<Watch>(
            PluginsWatchRequest { name: None, at: None },
            StreamSettings {
                items: NonZeroUsize::new(settings.stream_buffer).unwrap(),
                bytes: NonZeroUsize::new(settings.message_bytes).unwrap(),
            },
        )?;
        loop {
            let event = tokio::select! {
                biased;
                _ = cancellation.cancelled() => {
                    watch.request_cancel();
                    match watch.join().await {
                        Ok(_) | Err(harmos_runtime::Error::OperationRefused(harmos_runtime::Refusal::Cancelled { .. })) => {}
                        Err(source) => return Err(source.into()),
                    }
                    return Ok(());
                }
                event = watch.next() => event?,
            };
            let Some(event) = event else {
                return Err(Error::Invalid("Observe Plugins: Reopen the Closed Plugin Watch".into()));
            };
            let Some(context) = context.upgrade() else {
                watch.request_cancel();
                match watch.join().await {
                    Ok(_) | Err(harmos_runtime::Error::OperationRefused(harmos_runtime::Refusal::Cancelled { .. })) => {}
                    Err(source) => return Err(source.into()),
                }
                return Ok(());
            };
            context.host.refresh_plugin_states().await?;
            dokime_operations::host::plugins::secrets::reconcile(&context).await?;
            context.host.signal_change();
            if matches!(event.event, Some(Event::ResyncRequired(_))) {
                watch.request_cancel();
                match watch.join().await {
                    Ok(_) | Err(harmos_runtime::Error::OperationRefused(harmos_runtime::Refusal::Cancelled { .. })) => {}
                    Err(source) => return Err(source.into()),
                }
                break;
            }
        }
    }
}

fn register_services(manager: &GrpcManager, services: Services) -> Result<()> {
    let message_bytes = services.context.settings.message_bytes;
    manager.register_service(
        dokime_protocol::host::plugins::secrets_server::SecretsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        dokime_protocol::host::files::captures_server::CapturesServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        dokime_protocol::host::evidence::logs_server::LogsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        dokime_protocol::host::effects_server::EffectsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        FilesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        UploadsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        AggregatesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        BackendServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        CasesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        AnalysesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        CyclesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        DocumentsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        EvidenceServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        SchemasServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        ExecutionsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        PromptsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        ReportsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        SecretsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        dokime_protocol::host::settings_server::SettingsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        OverviewServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        PromptDefinitionsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        RecordSchemasServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        StreamSchemasServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        SuitesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        ArtifactsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        ExecutionFilesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        HealthEventsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        NotesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        RecordsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        SamplesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        TraceabilityServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        CommandsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        ProgressServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        CapturesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        dokime_protocol::host::plots::settings_server::SettingsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        SnapshotsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        AnswersServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        SourcesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    Ok(())
}

/* === Tests === */
