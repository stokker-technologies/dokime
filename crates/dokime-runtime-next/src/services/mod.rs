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

mod backend;
mod catalog;
mod clock;
mod content;
mod evidence;
mod execution;
mod health;
mod media;
mod notes;
mod observation;
mod planning;
mod plots;
mod plugins;
mod prompts;
mod query;
mod reports;
mod state;
mod telemetry;

/* === Re-Exports === */

pub use clock::WallClock;
pub use dokime_operations_next::client::ClientSettings;

/* === Imports === */

// Standard Library Imports
use std::{
    net::SocketAddr,
    path::Path,
    sync::{Arc, Mutex},
};

// Third Party Imports
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;

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
    cancellation: CancellationToken,
    task: Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>,
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
        self.cancellation.cancel();
        let mut task = self.task.take().ok_or_else(|| Error::Invalid("Close the Listener Only Once".into()))?;
        match tokio::time::timeout(self.close_timeout, &mut task).await {
            Ok(result) => {
                result??;
                Ok(())
            }
            Err(_) => {
                task.abort();
                let _ = task.await;
                Err(Error::Invalid("Close Listener: Client Streams Exceeded the Shutdown Deadline".into()))
            }
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.cancellation.cancel();
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

/* === Public Functions === */

pub async fn serve(endpoint: SocketAddr, root: &Path, host_settings: HostSettings, settings: ClientSettings) -> Result<Server> {
    if !endpoint.ip().is_loopback() {
        return Err(Error::Invalid("Serve the New Host on a Local Endpoint Until Remote Authentication Is Implemented".into()));
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
    let listener = TcpListener::bind(endpoint).await?;
    let endpoint = listener.local_addr()?;
    let host = Host::open(root, host_settings)?;
    let identity_path = root.join("backend-id");
    let backend_id = if identity_path.exists() {
        std::fs::read_to_string(&identity_path)?
    } else {
        use std::io::Write;
        let identifier = uuid::Uuid::now_v7().to_string();
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&identity_path)?;
        file.write_all(identifier.as_bytes())?;
        file.sync_all()?;
        identifier
    };
    let identity = v1::BackendIdentity {
        backend_id,
        epoch: uuid::Uuid::now_v7().to_string(),
        api: Some(v1::ApiIdentity {
            package: "dokime.v1".into(),
            version: "v1".into(),
            descriptor_sha256: Vec::new(),
        }),
        product_version: env!("CARGO_PKG_VERSION").into(),
        started_at: Some(prost_types::Timestamp {
            seconds: host.now_ns().div_euclid(1_000_000_000),
            nanos: host.now_ns().rem_euclid(1_000_000_000) as i32,
        }),
        display_name: "Dokime Execution Host".into(),
        ..Default::default()
    };
    let state = state::State {
        host: host.clone(),
        render_settings: crate::RenderSettings {
            page_size: std::num::NonZeroUsize::new(settings.page_items).unwrap(),
            display_points: std::num::NonZeroUsize::new(settings.page_items).unwrap(),
        },
        identity,
        urls: Mutex::new(std::collections::HashMap::new()),
        progress: Mutex::new(std::collections::HashMap::new()),
    };
    let message_bytes = settings.message_bytes;
    let close_timeout = settings.mutation_timeout;
    let context = dokime_operations_next::client::Context { host: Arc::new(state), settings };
    let services = Services { context };
    let cancellation = CancellationToken::new();
    let shutdown = cancellation.clone();
    let task = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(v1::media_server::MediaServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes))
            .add_service(v1::content_server::ContentServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes))
            .add_service(v1::notes_server::NotesServer::new(services.clone()).max_decoding_message_size(message_bytes).max_encoding_message_size(message_bytes))
            .add_service(
                v1::plots_server::PlotsServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::plugins_server::PluginsServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::catalog_server::CatalogServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::planning_server::PlanningServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::execution_server::ExecutionServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::observation_server::ObservationServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::prompts_server::PromptsServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::query_server::QueryServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::evidence_server::EvidenceServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::telemetry_server::TelemetryServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::reports_server::ReportsServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::health_server::HealthServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .add_service(
                v1::backend_server::BackendServer::new(services.clone())
                    .max_decoding_message_size(message_bytes)
                    .max_encoding_message_size(message_bytes),
            )
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown.cancelled_owned())
            .await
    });
    Ok(Server {
        endpoint,
        host,
        cancellation,
        task: Some(task),
        close_timeout,
    })
}

/* === Private Functions === */

/* === Tests === */
