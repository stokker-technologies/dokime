/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Pairing Key Rotation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, RwLock};

// Third Party Imports
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

// Local Imports
use super::{Controller, Error, store};
use dokime::domain::model::remote::{CreatedPairing, PairingFile, RemoteEndpoint};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Key {
    verifier: RwLock<Option<[u8; 32]>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for Key {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Key").finish_non_exhaustive()
    }
}

impl Key {
    pub(super) fn new(verifier: Option<[u8; 32]>) -> Self {
        Self { verifier: RwLock::new(verifier) }
    }

    pub(crate) fn authenticate(&self, digest: &[u8; 32]) -> Result<(), tonic::Status> {
        let verifier = self.verifier.read().map_err(|_| tonic::Status::unavailable("backend key unavailable"))?;
        if verifier.as_ref().is_some_and(|expected| bool::from(digest.ct_eq(expected))) {
            return Ok(());
        }
        Err(tonic::Status::unauthenticated("invalid credentials"))
    }
}

impl Controller {
    pub(in crate::grpc) async fn export_pairing(self: &Arc<Self>, label: String) -> Result<CreatedPairing, Error> {
        let settings = {
            let state = self.state.lock().await;
            if state.closing {
                return Err(Error::Configuration("remote listener is closing"));
            }
            state.document.settings.clone()
        };
        if !settings.enabled {
            let settings = dokime::domain::model::remote::RemoteListenerSettings {
                enabled: true,
                address: "127.0.0.1:0".into(),
                tls_enabled: true,
                authentication_enabled: true,
                ..settings
            };
            self.change(super::Change::Settings(settings)).await?;
        }
        self.create_pairing(label).await
    }

    pub(super) async fn create_pairing(self: &Arc<Self>, label: String) -> Result<CreatedPairing, Error> {
        // Persist And Publish Even When The Native Caller Cancels.
        let controller = self.clone();
        tokio::spawn(async move {
            {
                controller.change(super::Change::Refresh).await?;
                controller.create(label).await
            }
        })
        .await?
    }

    async fn create(&self, label: String) -> Result<CreatedPairing, Error> {
        let mut state = self.state.lock().await;
        if state.closing {
            return Err(Error::Configuration("remote listener is closing"));
        }
        if state.error.is_some() {
            return Err(Error::Configuration("resolve the listener restart failure before creating a pairing"));
        }
        let bound = state.bound.as_ref().ok_or(Error::Configuration("remote listener must be active before creating a pairing"))?;
        let server = state.server.as_ref().ok_or(Error::Configuration("remote listener is not accepting calls"))?;
        let identity = self.runtime.watch_state().latest().state().identity.node.node_id.to_string();
        let mut pairing = PairingFile {
            version: 1,
            endpoints: endpoints(
                store::host_endpoints(server.address(), super::SERVER_REMOTE_NAMES as usize),
                server.address().port(),
                bound.tls.is_some(),
            ),
            backend_identifier: identity,
            label,
            certificate_pem: if bound.tls.is_some() {
                Some(state.document.identity.as_ref().ok_or(Error::Configuration("active certificate is unavailable"))?.certificate.clone())
            } else {
                None
            },
            access_key: None,
        };
        pairing.validate().map_err(Error::Listener)?;
        if bound.tls.is_some() {
            let identity = state.document.identity.as_ref().ok_or(Error::Configuration("active certificate is unavailable"))?;
            for endpoint in &pairing.endpoints {
                let authority = endpoint.split_once("://").expect("validated endpoint").1;
                let host = authority.rsplit_once(':').expect("validated port").0.trim_matches(['[', ']']);
                if !identity.names.iter().any(|name| name.eq_ignore_ascii_case(host)) {
                    return Err(Error::Configuration("remote endpoint is not covered by the active TLS certificate"));
                }
            }
        }
        if !matches!(bound.policy, crate::grpc::access::Policy::BackendKey(_)) {
            return Ok(CreatedPairing { pairing });
        }
        let mut candidate = state.document.clone();
        let random = Zeroizing::new(rand::random::<[u8; 32]>());
        let mut key = Zeroizing::new(String::with_capacity(64));
        use std::fmt::Write as _;
        for byte in random.iter() {
            write!(key, "{byte:02x}").expect("writing to a string succeeds");
        }
        candidate.verifier = Some(Sha256::digest(key.as_bytes()).into());
        self.write_security(&candidate).await?;
        *self.key.verifier.write().unwrap_or_else(std::sync::PoisonError::into_inner) = candidate.verifier;
        state.document = candidate;
        pairing.access_key = Some(key.to_string());
        Ok(CreatedPairing { pairing })
    }
}

/* === Public Functions === */

pub(super) fn endpoints(hosts: Vec<RemoteEndpoint>, port: u16, tls: bool) -> Vec<String> {
    hosts
        .into_iter()
        .map(|endpoint| {
            let name = endpoint.address;
            let host = if name.contains(':') { format!("[{name}]") } else { name };
            format!("{}://{host}:{port}", if tls { "https" } else { "http" })
        })
        .collect()
}

/* === Private Functions === */

/* === Tests === */
