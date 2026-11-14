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
mod bundle_permissions;
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
mod managed_plugins;
mod media_captures;
mod plots_settings;
mod plots_snapshots;
mod plugin_entries;
mod plugins;
mod plugins_settings;
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
pub use dokime_operations_next::client::ClientSettings;

/* === Imports === */

// Standard Library Imports
use std::{net::SocketAddr, path::Path, sync::Arc};

// Third Party Imports

// Local Imports
use crate::{Error, Host, HostSettings, Result};
use dokime_operations_next::client::Context;
use dokime_operations_next::client::HostIdentity;
use dokime_protocol::client::aggregates_server::AggregatesServer;
use dokime_protocol::client::analyses_server::AnalysesServer;
use dokime_protocol::client::backend::overview_server::OverviewServer;
use dokime_protocol::client::backend_server::BackendServer;
use dokime_protocol::client::cases_server::CasesServer;
use dokime_protocol::client::content::artifacts_server::ArtifactsServer;
use dokime_protocol::client::content::execution_files_server::ExecutionFilesServer;
use dokime_protocol::client::cycles_server::CyclesServer;
use dokime_protocol::client::documents_server::DocumentsServer;
use dokime_protocol::client::evidence::health_events_server::HealthEventsServer;
use dokime_protocol::client::evidence::notes_server::NotesServer;
use dokime_protocol::client::evidence::records_server::RecordsServer;
use dokime_protocol::client::evidence::samples_server::SamplesServer;
use dokime_protocol::client::evidence::traceability_server::TraceabilityServer;
use dokime_protocol::client::evidence_server::EvidenceServer;
use dokime_protocol::client::executions::commands_server::CommandsServer;
use dokime_protocol::client::executions::progress_server::ProgressServer;
use dokime_protocol::client::executions::schemas_server::SchemasServer;
use dokime_protocol::client::executions_server::ExecutionsServer;
use dokime_protocol::client::files::uploads_server::UploadsServer;
use dokime_protocol::client::files_server::FilesServer;
use dokime_protocol::client::media::captures_server::CapturesServer;
use dokime_protocol::client::plots::snapshots_server::SnapshotsServer;
use dokime_protocol::client::plugins::settings_server::SettingsServer;
use dokime_protocol::client::plugins_server::PluginsServer;
use dokime_protocol::client::prompt_definitions_server::PromptDefinitionsServer;
use dokime_protocol::client::prompts::answers_server::AnswersServer;
use dokime_protocol::client::prompts_server::PromptsServer;
use dokime_protocol::client::record_schemas_server::RecordSchemasServer;
use dokime_protocol::client::reports_server::ReportsServer;
use dokime_protocol::client::secrets_server::SecretsServer;
use dokime_protocol::client::storage::sources_server::SourcesServer;
use dokime_protocol::client::stream_schemas_server::StreamSchemasServer;
use dokime_protocol::client::suites_server::SuitesServer;
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
    context: Context,
}
pub struct Server {
    endpoint: SocketAddr,
    host: Host,
    runtime: harmos_runtime::Runtime<composition::Application>,
    listener: Option<harmos_runtime::grpc::Attached>,
    close_timeout: Duration,
    status: Arc<harmos_runtime::services::status::Status>,
    status_task: Option<JoinHandle<Result<()>>>,
    _status_registration: Arc<harmos_runtime::services::status::Registration>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Server {
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    pub fn status(&self) -> &Arc<harmos_runtime::services::status::Status> {
        &self.status
    }
    pub async fn close(mut self) -> Result<()> {
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
        if let Some(task) = self.status_task.take() {
            task.abort();
        }
    }
}

/* === Public Functions === */

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
    dokime_operations_next::client::remove_root_secrets(&storage)?;
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
    let runtime = composition::launch(clock).await?;
    let status = runtime.services().configure_status(status::settings(&settings, backend_id.clone()))?;
    let administration = Arc::new(Mutex::new(()));
    let state = state::State {
        administration: administration.clone(),
        status: status.clone(),
        root: root.to_owned(),
        host: host.clone(),
        render_settings: crate::RenderSettings {
            page_size: NonZeroUsize::new(settings.page_items).unwrap(),
            display_points: NonZeroUsize::new(settings.page_items).unwrap(),
        },
        identity,
        temporary_report_limit: settings.max_cases,
    };
    let close_timeout = settings.mutation_timeout;
    let context = Context {
        administration,
        host: Arc::new(state),
        settings: settings.clone(),
    };
    context.host.restore_managed_plugins().await.map_err(|error| Error::Invalid(error.to_string()))?;
    dokime_operations_next::client::register_redactions(&context)?;
    let registration = Arc::new(
        status
            .contribute(Arc::new(status::Facts(Arc::downgrade(&context.host))))
            .map_err(|source| status::failure("Contribute Host Status", source))?,
    );
    let mut changes = context.host.changes();
    let weak_host = Arc::downgrade(&context.host);
    let services = Services { context };
    register_services(runtime.grpc(), services.clone())?;
    runtime
        .grpc()
        .configure_access(composition::access_settings(&settings), Arc::new(access_credentials::Store::new(host.store.clone(), backend_id)))
        .await?;
    let mut transport = composition::transport(&settings);
    transport.address = endpoint;
    let listener = runtime.grpc().attach(transport).await?;
    let endpoint = listener.local_addr();
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
        _status_registration: registration,
    })
}

/* === Private Functions === */

fn register_services(manager: &harmos_runtime::grpc::GrpcManager, services: Services) -> Result<()> {
    let message_bytes = services.context.settings.message_bytes;
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
        PluginsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
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
        dokime_protocol::client::settings_server::SettingsServer::new(services.clone())
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
        dokime_protocol::client::plots::settings_server::SettingsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        SnapshotsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        SettingsServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes),
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
