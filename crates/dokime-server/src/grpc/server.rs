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
use std::sync::Arc;
use std::{net::SocketAddr, time::Duration};
// Third Party Imports
use futures_util::StreamExt;
use tokio::{
    net::TcpListener,
    sync::{Semaphore, oneshot, watch},
    task::JoinHandle,
};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::{ServerTlsConfig, server::Server as TonicServer};
// Local Imports
use super::content::ContentServer;
use super::evidence::EvidenceServer;
use super::execution::ExecutionServer;
use super::history::HistoryServer;
use super::query::QueryServer;
use super::{Config, Error, backend::Service};
use dokime_app::RuntimeHandle;
use dokime_protocol::v1::{
    appearance_server::AppearanceServer, backend_server::BackendServer,
    catalog_server::CatalogServer, documents_server::DocumentsServer, health_server::HealthServer,
    notes_server::NotesServer, observation_server::ObservationServer,
    planning_server::PlanningServer, plots_server::PlotsServer, plugins_server::PluginsServer,
    prompts_server::PromptsServer, reports_server::ReportsServer, settings_server::SettingsServer,
    telemetry_server::TelemetryServer,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Deps {
    runtime: RuntimeHandle,
    stopping: watch::Receiver<bool>,
    max_message_bytes: usize,
    local_only: bool,
}

pub struct Server {
    address: SocketAddr,
    tls: bool,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<(), tonic::transport::Error>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Deps {
    fn runtime(&self) -> RuntimeHandle {
        self.runtime.clone()
    }

    fn stopping(&self) -> watch::Receiver<bool> {
        self.stopping.clone()
    }

    fn admission(permits: usize) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(permits))
    }
}

impl Server {
    pub async fn start(runtime: RuntimeHandle, mut config: Config) -> Result<Self, Error> {
        config.validate()?;
        let (stopping, mut drain) = watch::channel(false);
        // The diagnostics archive and the log tail are streams, so the backend
        // service needs the same drain signal every other content service holds.
        let backend = Service::new(runtime.clone(), &config, drain.clone()).await?;
        let tls = config.tls.is_some();
        let local_only = config.local_only();
        // Authentication and permission belong in front of the router, where the
        // requested method is still visible; see `access::Authorize`.
        let mut builder = TonicServer::builder()
            .layer(super::deadline::Deadlines::default())
            .layer(super::access::Authorize::new(config.policy.clone()))
            .concurrency_limit_per_connection(32)
            .max_concurrent_streams(32);
        if let Some(identity) = config.tls.take() {
            // Tonic server TLS uses the process default,
            // so feature inference is ambiguous; preserve any host-selected provider.
            let _ = rustls::crypto::ring::default_provider().install_default();

            builder = builder
                .tls_config(
                    ServerTlsConfig::new()
                        .identity(identity.into_tonic())
                        .timeout(Duration::from_secs(10)),
                )
                .map_err(|_| Error::Configuration("invalid TLS identity"))?;
        }
        let listener = TcpListener::bind(config.bind_addr).await?;
        let address = listener.local_addr()?;
        let deps = Deps {
            runtime,
            stopping: drain.clone(),
            max_message_bytes: config.max_message_bytes,
            local_only,
        };
        let bytes = deps.max_message_bytes;
        let runtime_operations = super::operations::runtime(&deps.runtime, bytes)?;
        // Every service is registered the same way: the configured limit on both
        // directions, and the subset of `Deps` it needs. Only the service type and
        // its fields vary, so only those appear here.
        macro_rules! services {
            ($($server:ident => $service:expr),+ $(,)?) => {
                builder$(.add_service(
                    $server::new($service)
                        .max_decoding_message_size(bytes)
                        .max_encoding_message_size(bytes),
                ))+
            };
        }
        let router = services! {
            BackendServer => backend,
            CatalogServer => super::catalog::Service {
                runtime: deps.runtime(),
            },
            ExecutionServer => super::execution::Service {
                runtime: deps.runtime(),
                max_message_bytes: bytes,
            },
            ObservationServer => super::observation::Service {
                runtime: deps.runtime(),
                stopping: deps.stopping(),
                max_message_bytes: bytes,
            },
            PlanningServer => super::planning::Service {
                runtime: deps.runtime(),
            },
            PromptsServer => super::prompts::Service {
                runtime: deps.runtime(),
            },
            QueryServer => super::query::Service {
                runtime: deps.runtime(),
                max_message_bytes: bytes,
            },
            NotesServer => super::notes::Service {
                runtime: deps.runtime(),
                max_message_bytes: bytes,
            },
            // Health serves one RPC from the same annotation state as Notes.
            HealthServer => super::notes::Service {
                runtime: deps.runtime(),
                max_message_bytes: bytes,
            },
            EvidenceServer => super::evidence::Service {
                runtime: deps.runtime(),
                stopping: deps.stopping(),
                max_message_bytes: bytes,
            },
            PluginsServer => super::plugins::Service {
                runtime: deps.runtime(),
                admission: Deps::admission(1),
                stopping: deps.stopping(),
                max_message_bytes: bytes,
            },
            DocumentsServer => super::documents::Service {
                runtime: deps.runtime(),
                stopping: deps.stopping(),
                max_message_bytes: bytes,
                admission: Deps::admission(4),
            },
            ContentServer => super::content::Service {
                runtime: deps.runtime(),
                stopping: deps.stopping(),
                max_message_bytes: bytes,
                local_only: deps.local_only,
            },
            AppearanceServer => super::appearance::Service {
                runtime: deps.runtime(),
                max_message_bytes: bytes,
            },
            SettingsServer => super::settings::Service {
                runtime: deps.runtime(),
                max_message_bytes: bytes,
            },
            PlotsServer => super::plots::Service {
                runtime: deps.runtime(),
                stopping: deps.stopping(),
                max_message_bytes: bytes,
            },
            TelemetryServer => super::telemetry::Service {
                runtime: deps.runtime(),
                max_message_bytes: bytes,
            },
            HistoryServer => super::history::Service {
                runtime: deps.runtime(),
                stopping: deps.stopping(),
                max_message_bytes: bytes,
            },
            ReportsServer => super::reports::Service {
                runtime: deps.runtime(),
                stopping: deps.stopping(),
                max_message_bytes: bytes,
            },
        };
        let router = router.add_service(runtime_operations);
        let (shutdown, stopped) = oneshot::channel();
        let cancel = CancellationToken::new();
        let connections = cancel.clone();
        let incoming = TcpListenerStream::new(listener).map(move |result| {
            result.and_then(|stream| {
                // Custom incoming streams bypass tonic's TCP setup. Small HTTP/2
                // windows must not wait for Nagle/delayed-ACK round trips.
                stream.set_nodelay(true)?;
                Ok(super::transport::Io::new(stream, connections.clone()))
            })
        });
        let task = tokio::spawn(async move {
            let _close_connections = cancel.clone().drop_guard();
            let serve = router.serve_with_incoming_shutdown(incoming, async {
                let _ = stopped.await;
                let _ = stopping.send(true);
            });
            tokio::pin!(serve);
            tokio::select! {
                result=&mut serve=>result,
                _=drain.changed()=>{
                    match tokio::time::timeout(Duration::from_secs(5),&mut serve).await {
                        Ok(result)=>result,
                        Err(_)=>{cancel.cancel();serve.await}
                    }
                }
            }
        });
        Ok(Self {
            address,
            tls,
            shutdown: Some(shutdown),
            task: Some(task),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn local_endpoint(&self) -> String {
        let mut address = self.address;
        if address.ip().is_unspecified() {
            address.set_ip(if address.is_ipv4() {
                std::net::Ipv4Addr::LOCALHOST.into()
            } else {
                std::net::Ipv6Addr::LOCALHOST.into()
            });
        }
        format!("{}://{address}", if self.tls { "https" } else { "http" })
    }

    pub fn is_finished(&self) -> bool {
        self.task.as_ref().is_none_or(JoinHandle::is_finished)
    }

    pub async fn shutdown(mut self) -> Result<(), Error> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(task) = self.task.take() {
            task.await??;
        }
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
