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

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock, RwLock, Weak},
};

// Third Party Imports
use prost::Message;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

// Local Imports
use super::{ClientSettings, Services};
use crate::{Error, Result};
use dokime_operations_next::client::{Context, Host as ClientHost};
use dokime_protocol::v1;
use dokime_storage::Store;

/* === Constants === */

const REMOTE_KEY_SERVICE: &str = "dokime.remote-tls";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Control {
    store: Store,
    identity: v1::BackendIdentity,
    settings: ClientSettings,
    administration: Arc<Mutex<()>>,
    host: OnceLock<Weak<dyn ClientHost>>,
    active: Mutex<Active>,
    verifier: Arc<RwLock<Vec<u8>>>,
}
struct Active {
    security: Security,
    bound: Option<SocketAddr>,
    task: Option<JoinHandle<std::result::Result<(), tonic::transport::Error>>>,
    cancellation: CancellationToken,
    error: Option<String>,
}
#[derive(Clone, PartialEq, Message)]
struct Security {
    #[prost(message, optional, tag = "1")]
    settings: Option<v1::RemoteListenerSettings>,

    #[prost(string, tag = "2")]
    certificate: String,

    #[prost(string, tag = "3")]
    key_slot: String,

    #[prost(bytes = "vec", tag = "4")]
    verifier: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Control {
    pub(super) fn new(
        store: Store, identity: v1::BackendIdentity, settings: ClientSettings, administration: Arc<Mutex<()>>,
    ) -> Result<Arc<Self>> {
        let security = store.administration("remote-security")?.unwrap_or_else(|| Security {
            settings: Some(v1::RemoteListenerSettings {
                address: "127.0.0.1:0".into(),
                tls_enabled: true,
                authentication_enabled: true,
                certificate_names: vec!["localhost".into(), "127.0.0.1".into()],
                ..Default::default()
            }),
            ..Default::default()
        });
        let verifier = Arc::new(RwLock::new(security.verifier.clone()));
        Ok(Arc::new(Self {
            store,
            identity,
            settings,
            administration,
            host: OnceLock::new(),
            active: Mutex::new(Active {
                security,
                bound: None,
                task: None,
                cancellation: CancellationToken::new(),
                error: None,
            }),
            verifier,
        }))
    }
    pub(super) fn attach(&self, host: &Arc<dyn ClientHost>) -> Result<()> {
        self.host.set(Arc::downgrade(host)).map_err(|_| Error::Invalid("Attach the Remote Listener Only Once".into()))
    }
    pub(super) async fn restore(&self) -> Result<()> {
        let settings = self
            .store
            .administration::<v1::SettingsSnapshot>("preferences")?
            .and_then(|snapshot| snapshot.settings)
            .and_then(|settings| settings.listener)
            .or(self.active.lock().await.security.settings.clone());
        if let Some(settings) = settings
            && let Err(error) = self.update(settings).await
        {
            self.active.lock().await.error = Some(error.to_string());
        }
        Ok(())
    }
    pub(super) async fn read(&self) -> Result<v1::RemoteListenerStatus> {
        Ok(status(&*self.active.lock().await))
    }
    pub(super) async fn close(&self) -> Result<()> {
        let mut active = self.active.lock().await;
        self.stop(&mut active).await;
        Ok(())
    }
    pub(super) fn abort(&self) {
        if let Ok(mut active) = self.active.try_lock() {
            active.cancellation.cancel();
            if let Some(task) = active.task.take() {
                task.abort();
            }
        }
    }
    async fn stop(&self, active: &mut Active) {
        active.cancellation.cancel();
        if let Some(task) = active.task.take() {
            // A Remote Mutation Can Be Served by This Listener; Never Await Its Own RPC Settlement.
            task.abort();
            let _ = task.await;
        }
        active.bound = None;
    }
    pub(super) async fn update(&self, settings: v1::RemoteListenerSettings) -> Result<v1::RemoteListenerStatus> {
        validate(&settings)?;
        let mut active = self.active.lock().await;
        let mut candidate = active.security.clone();
        candidate.settings = Some(settings.clone());
        if settings.enabled
            && (candidate.certificate.is_empty()
                || active.security.settings.as_ref().is_none_or(|previous| previous.certificate_names != settings.certificate_names))
        {
            self.generate(&mut candidate)?;
        }
        self.stop(&mut active).await;
        if settings.enabled
            && let Err(error) = self.start(&mut active, &candidate).await
        {
            active.error = Some(error.to_string());
            return Err(error);
        }
        self.store.save_administration("remote-security", &candidate)?;
        // Publish the Listener Preferences Through the Same Revision Owner as Settings.
        let mut preferences: v1::SettingsSnapshot = self
            .store
            .administration("preferences")?
            .unwrap_or_else(dokime_operations_next::client::settings_read_application_settings::initial_preferences);
        if let Some(settings) = &mut preferences.settings
            && settings.listener != candidate.settings
        {
            settings.listener = candidate.settings.clone();
            let update = v1::UpdateSettings {
                expected_revision: preferences.revision,
                settings: preferences.settings.clone(),
            };
            let _ = self.store.update_preferences(update, &preferences)?;
        }
        if !active.security.key_slot.is_empty() && active.security.key_slot != candidate.key_slot {
            let _ = keyring::Entry::new(REMOTE_KEY_SERVICE, &format!("{}:{}", self.identity.backend_id, active.security.key_slot))
                .and_then(|entry| entry.delete_credential());
        }
        active.security = candidate;
        active.error = None;
        Ok(status(&active))
    }
    fn generate(&self, security: &mut Security) -> Result<()> {
        let names = security.settings.as_ref().unwrap().certificate_names.clone();
        let mut params =
            rcgen::CertificateParams::new(names).map_err(|_| invalid("Supply Valid DNS Names or IP Addresses for the Certificate"))?;
        params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        params.key_usages = vec![rcgen::KeyUsagePurpose::DigitalSignature];
        params.distinguished_name.push(rcgen::DnType::CommonName, "Dokime Remote Access");
        let key = rcgen::KeyPair::generate().map_err(|_| invalid("Retry TLS Key Generation"))?;
        let certificate = params.self_signed(&key).map_err(|_| invalid("Retry Certificate Generation"))?;
        let slot = uuid::Uuid::now_v7().to_string();
        let key_pem = Zeroizing::new(key.serialize_pem());
        keyring::Entry::new(REMOTE_KEY_SERVICE, &format!("{}:{slot}", self.identity.backend_id))
            .map_err(|_| invalid("Unlock the Operating System Credential Store"))?
            .set_password(&key_pem)
            .map_err(|_| invalid("Unlock the Operating System Credential Store"))?;
        security.certificate = certificate.pem();
        security.key_slot = slot;
        Ok(())
    }
    pub(super) async fn pair(&self, label: String) -> Result<v1::CreatedPairing> {
        if label.trim().is_empty() || label.len() > 256 || label.contains('\0') {
            return Err(invalid("Give the Pairing a Nonempty Label up to 256 Bytes Without NUL"));
        }
        let settings = self.active.lock().await.security.settings.clone().unwrap();
        if !settings.enabled {
            self.update(v1::RemoteListenerSettings { enabled: true, ..settings }).await?;
        }
        let mut active = self.active.lock().await;
        let bound = active.bound.ok_or_else(|| invalid("Enable the Remote Listener Before Exporting a Pairing"))?;
        let key = Zeroizing::new(rand::random::<[u8; 32]>().iter().map(|byte| format!("{byte:02x}")).collect::<String>());
        let mut candidate = active.security.clone();
        candidate.verifier = Sha256::digest(key.as_bytes()).to_vec();
        self.store.save_administration("remote-security", &candidate)?;
        *self.verifier.write().map_err(|_| invalid("Restart the Host After Its Key Lock Failed"))? = candidate.verifier.clone();
        active.security = candidate;
        let settings = active.security.settings.as_ref().unwrap();
        let endpoints = endpoints(bound, &settings.certificate_names);
        if endpoints.is_empty() {
            return Err(invalid("Configure a Reachable Address Covered by the Certificate Before Exporting"));
        }
        Ok(v1::CreatedPairing {
            pairing: Some(v1::PairingFile {
                version: 1,
                endpoints,
                backend_identifier: self.identity.backend_id.clone(),
                label,
                certificate_pem: Some(active.security.certificate.clone()),
                access_key: Some(key.to_string()),
            }),
        })
    }
    pub(super) async fn rotate(&self, names: Vec<String>) -> Result<v1::RemoteListenerStatus> {
        let mut settings = self.active.lock().await.security.settings.clone().unwrap();
        settings.certificate_names = names;
        validate(&settings)?;
        let mut active = self.active.lock().await;
        let mut candidate = active.security.clone();
        candidate.settings = Some(settings.clone());
        self.generate(&mut candidate)?;
        self.stop(&mut active).await;
        if settings.enabled {
            self.start(&mut active, &candidate).await?;
        }
        self.store.save_administration("remote-security", &candidate)?;
        let mut preferences: v1::SettingsSnapshot = self
            .store
            .administration("preferences")?
            .unwrap_or_else(dokime_operations_next::client::settings_read_application_settings::initial_preferences);
        if let Some(settings) = &mut preferences.settings
            && settings.listener != candidate.settings
        {
            settings.listener = candidate.settings.clone();
            self.store.update_preferences(
                v1::UpdateSettings {
                    expected_revision: preferences.revision,
                    settings: preferences.settings.clone(),
                },
                &preferences,
            )?;
        }
        if !active.security.key_slot.is_empty() && active.security.key_slot != candidate.key_slot {
            let _ = keyring::Entry::new(REMOTE_KEY_SERVICE, &format!("{}:{}", self.identity.backend_id, active.security.key_slot))
                .and_then(|entry| entry.delete_credential());
        }
        active.security = candidate;
        active.error = None;
        Ok(status(&active))
    }
    async fn start(&self, active: &mut Active, security: &Security) -> Result<()> {
        let settings = security.settings.as_ref().unwrap();
        let listener =
            TcpListener::bind(settings.address.parse::<SocketAddr>().map_err(|_| invalid("Supply a Listener IP Address and Port"))?)
                .await?;
        let bound = listener.local_addr()?;
        let key = Zeroizing::new(
            keyring::Entry::new(REMOTE_KEY_SERVICE, &format!("{}:{}", self.identity.backend_id, security.key_slot))
                .map_err(|_| invalid("Unlock the Credential Store and Retry Remote Access"))?
                .get_password()
                .map_err(|_| invalid("Unlock the Credential Store and Retry Remote Access"))?,
        );
        let tls = tonic::transport::ServerTlsConfig::new()
            .identity(tonic::transport::Identity::from_pem(security.certificate.clone(), key.as_bytes()));
        let host = self.host.get().and_then(Weak::upgrade).ok_or_else(|| invalid("Restart the Host Before Enabling Remote Access"))?;
        let context = Context {
            host,
            settings: self.settings.clone(),
            administration: self.administration.clone(),
        };
        let services = Services { context };
        let verifier = self.verifier.clone();
        let authenticate = move |request: tonic::Request<()>| {
            let token = request
                .metadata()
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .ok_or_else(|| tonic::Status::unauthenticated("Import a Current Pairing File to Connect"))?;
            let digest = Sha256::digest(token.as_bytes());
            let expected = verifier.read().map_err(|_| tonic::Status::unavailable("Restart the Host After Its Key Lock Failed"))?;
            if expected.len() != digest.len() || !bool::from(expected.as_slice().ct_eq(digest.as_slice())) {
                return Err(tonic::Status::unauthenticated("Import a Fresh Pairing File After Access Key Rotation"));
            }
            Ok(request)
        };
        let message_bytes = self.settings.message_bytes;
        let cancellation = CancellationToken::new();
        let shutdown = cancellation.clone();
        active.task = Some(tokio::spawn(async move {
            tonic::transport::Server::builder()
                .tls_config(tls)?
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::plugins_server::PluginsServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::catalog_server::CatalogServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::planning_server::PlanningServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::execution_server::ExecutionServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::observation_server::ObservationServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::prompts_server::PromptsServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::query_server::QueryServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::evidence_server::EvidenceServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::telemetry_server::TelemetryServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::reports_server::ReportsServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::health_server::HealthServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::backend_server::BackendServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::runtime_server::RuntimeServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::appearance_server::AppearanceServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::settings_server::SettingsServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::secrets_server::SecretsServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .add_service(tonic::service::interceptor::InterceptedService::new(
                    v1::remote_server::RemoteServer::new(services.clone())
                        .max_decoding_message_size(message_bytes)
                        .max_encoding_message_size(message_bytes),
                    authenticate.clone(),
                ))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown.cancelled_owned())
                .await
        }));
        active.cancellation = cancellation;
        active.bound = Some(bound);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid(message: &str) -> Error {
    Error::Invalid(format!("Configure Remote Access: {message}"))
}
fn validate(settings: &v1::RemoteListenerSettings) -> Result<()> {
    dokime_operations_next::client::remote_update_listener::validate_settings(settings)
        .map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(())
}
fn endpoints(bound: SocketAddr, names: &[String]) -> Vec<String> {
    if !bound.ip().is_unspecified() {
        return vec![format!("https://{bound}")];
    }
    names
        .iter()
        .map(|name| {
            let name = if name.contains(':') { format!("[{name}]") } else { name.clone() };
            format!("https://{name}:{}", bound.port())
        })
        .collect()
}
fn status(active: &Active) -> v1::RemoteListenerStatus {
    let settings = active.security.settings.clone().unwrap();
    v1::RemoteListenerStatus {
        settings: Some(settings.clone()),
        listening_address: active.bound.map(|address| address.to_string()),
        certificate_pem: (!active.security.certificate.is_empty()).then(|| active.security.certificate.clone()),
        key_configured: !active.security.verifier.is_empty(),
        error: active.error.clone(),
        endpoints: active
            .bound
            .map(|address| {
                endpoints(address, &settings.certificate_names)
                    .into_iter()
                    .map(|address| v1::RemoteEndpoint { address, ..Default::default() })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/* === Tests === */
