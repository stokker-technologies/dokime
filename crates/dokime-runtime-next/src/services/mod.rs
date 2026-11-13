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
mod appearance;
mod backend;
mod bundle_permissions;
mod bundles;
mod catalog;
mod clock;
mod composition;
mod content;
mod diagnostics;
mod documents;
mod evidence;
mod execution;
mod health;
mod history;
mod managed_plugins;
mod media;
mod notes;
mod observation;
mod planning;
mod plots;
mod plugin_entries;
mod plugins;
mod prompts;
mod query;
mod reports;
mod runtime;
mod secrets;
mod settings;
mod state;
mod telemetry;

/* === Re-Exports === */

pub use clock::WallClock;
pub use diagnostics::Diagnostics;
pub use dokime_operations_next::client::ClientSettings;

/* === Imports === */

// Standard Library Imports
use std::{net::SocketAddr, path::Path, sync::Arc};

// Third Party Imports

// Local Imports
use crate::{Error, Host, HostSettings, Result};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
struct Services {
    context: dokime_operations_next::client::Context,
}
pub struct Server {
    endpoint: SocketAddr,
    host: Host,
    runtime: harmos_runtime::Runtime<composition::Application>,
    listener: Option<harmos_runtime::grpc::Attached>,
    close_timeout: std::time::Duration,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Server {
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    pub async fn close(mut self) -> Result<()> {
        self.host.shutdown().await?;
        if let Some(listener) = self.listener.take() {
            listener.detach().await?;
        }
        self.runtime.shutdown_with_timeout(self.close_timeout).await?;
        Ok(())
    }
}

/* === Public Functions === */

pub async fn serve(endpoint: SocketAddr, root: &Path, host_settings: HostSettings, settings: ClientSettings) -> Result<Server> {
    let diagnostics = Diagnostics::new(
        host_settings.clock.clone(),
        std::num::NonZeroUsize::new(settings.page_items).ok_or_else(|| Error::Invalid("Serve with Positive Page Items".into()))?,
        std::num::NonZeroUsize::new(settings.message_bytes).ok_or_else(|| Error::Invalid("Serve with Positive Message Bytes".into()))?,
    );
    serve_with_diagnostics(endpoint, root, host_settings, settings, diagnostics).await
}
pub async fn serve_with_diagnostics(
    endpoint: SocketAddr, root: &Path, host_settings: HostSettings, settings: ClientSettings, diagnostics: Diagnostics,
) -> Result<Server> {
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
    let identity_path = root.join("backend-id");
    let backend_id = if identity_path.exists() {
        std::fs::read_to_string(&identity_path).map_err(|error| {
            Error::Invalid(format!("Start Host: Cannot Read Backend Identity; Make a Backup and Reset or Choose a Fresh Root: {error}"))
        })?
    } else {
        use std::io::Write;
        let identifier = uuid::Uuid::now_v7().to_string();
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&identity_path)?;
        file.write_all(identifier.as_bytes())?;
        file.sync_all()?;
        identifier
    };
    uuid::Uuid::parse_str(&backend_id)
        .map_err(|_| Error::Invalid("Start Host: Cannot Decode Backend Identity; Make a Backup and Reset or Choose a Fresh Root".into()))?;
    let identity = v1::BackendIdentity {
        backend_id: backend_id.clone(),
        epoch: uuid::Uuid::now_v7().to_string(),
        api: Some(v1::ApiIdentity {
            package: "dokime.v1".into(),
            version: "v1".into(),
            descriptor_sha256: Vec::new(),
        }),
        product_version: env!("CARGO_PKG_VERSION").into(),
        target: bundles::native_target().into(),
        started_at: Some(prost_types::Timestamp {
            seconds: host.now_ns().div_euclid(1_000_000_000),
            nanos: host.now_ns().rem_euclid(1_000_000_000) as i32,
        }),
        display_name: "Dokime Execution Host".into(),
        ..Default::default()
    };
    let administration = Arc::new(tokio::sync::Mutex::new(()));
    let state = state::State {
        administration: administration.clone(),
        diagnostics,
        root: root.to_owned(),
        host: host.clone(),
        render_settings: crate::RenderSettings {
            page_size: std::num::NonZeroUsize::new(settings.page_items).unwrap(),
            display_points: std::num::NonZeroUsize::new(settings.page_items).unwrap(),
        },
        identity,
        temporary_report_limit: settings.max_cases,
    };
    let close_timeout = settings.mutation_timeout;
    let context = dokime_operations_next::client::Context {
        administration,
        host: Arc::new(state),
        settings: settings.clone(),
    };
    context.host.restore_managed_plugins().await.map_err(|error| Error::Invalid(error.to_string()))?;
    let services = Services { context };
    let runtime = composition::launch(clock).await?;
    register_services(runtime.grpc(), services.clone())?;
    runtime
        .grpc()
        .configure_access(composition::access_settings(&settings), Arc::new(access_credentials::Store::new(host.store.clone(), backend_id)))
        .await?;
    let mut transport = composition::transport(&settings);
    transport.address = endpoint;
    let listener = runtime.grpc().attach(transport).await?;
    let endpoint = listener.local_addr();
    Ok(Server {
        endpoint,
        host,
        runtime,
        listener: Some(listener),
        close_timeout,
    })
}

/* === Private Functions === */

fn register_services(manager: &harmos_runtime::grpc::GrpcManager, services: Services) -> Result<()> {
    let message_bytes = services.context.settings.message_bytes;
    manager.register_service(
        v1::appearance_server::AppearanceServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::backend_server::BackendServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::catalog_server::CatalogServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::content_server::ContentServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::documents_server::DocumentsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::media_server::MediaServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::notes_server::NotesServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::plots_server::PlotsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::evidence_server::EvidenceServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::execution_server::ExecutionServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::health_server::HealthServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::history_server::HistoryServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::observation_server::ObservationServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::planning_server::PlanningServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::plugins_server::PluginsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::prompts_server::PromptsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::query_server::QueryServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::reports_server::ReportsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::runtime_server::RuntimeServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::secrets_server::SecretsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::settings_server::SettingsServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    manager.register_service(
        v1::telemetry_server::TelemetryServer::new(services.clone())
            .max_decoding_message_size(message_bytes)
            .max_encoding_message_size(message_bytes),
    )?;
    Ok(())
}

/* === Tests === */
