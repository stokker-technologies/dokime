/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod contract;
mod credentials;
mod errors;
mod transport;

/* === Re-Exports === */
pub use contract::*;

/* === Imports === */
// Standard Library Imports
use std::{
    future::Future,
    sync::{Arc, Mutex},
};
// Third Party Imports
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;
// Local Imports
use credentials::{KeyringStore, SecretStore};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct ConnectionSession {
    inner: Arc<Mutex<SessionState>>,
    secrets: Arc<dyn SecretStore>,
    secret_operations: Arc<tokio::sync::Mutex<()>>,
}

struct SessionState {
    generation: u64,
    cancellation: CancellationToken,
    snapshot: ConnectionSnapshot,
    lease: Option<SessionLease>,
}

#[derive(Clone)]
pub struct SessionLease {
    generation: String,
    client: dokime::client::Client,
    cancellation: CancellationToken,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for ConnectionSession {
    fn default() -> Self {
        Self::new(Arc::new(KeyringStore))
    }
}

impl ConnectionSession {
    fn new(secrets: Arc<dyn SecretStore>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(SessionState {
                generation: 0,
                cancellation: CancellationToken::new(),
                snapshot: ConnectionSnapshot {
                    generation: "0".into(),
                    phase: ConnectionPhase::Disconnected,
                    endpoint: None,
                    remote: false,
                    backend: None,
                    error: None,
                },
                lease: None,
            })),
            secrets,
            secret_operations: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub(crate) fn validate_target(target: ConnectionTarget) -> Result<(), ConnectionError> {
        transport::Target::parse(target).map(|_| ())
    }

    pub fn status(&self) -> ConnectionSnapshot {
        self.inner
            .lock()
            .expect("connection state poisoned")
            .snapshot
            .clone()
    }

    pub fn lease(&self, generation: &str) -> Result<SessionLease, ConnectionError> {
        let state = self.inner.lock().expect("connection state poisoned");
        if state.snapshot.generation != generation {
            return Err(ConnectionError::Superseded);
        }
        state.lease.clone().ok_or(ConnectionError::NotConnected)
    }

    pub fn disconnect(&self) -> Result<ConnectionSnapshot, ConnectionError> {
        let mut state = self.inner.lock().expect("connection state poisoned");
        state.advance()?;
        state.snapshot.phase = ConnectionPhase::Disconnected;
        state.snapshot.endpoint = None;
        state.snapshot.remote = false;
        Ok(state.snapshot.clone())
    }

    pub async fn connect(
        &self,
        request: ConnectRequest,
    ) -> Result<ConnectionSnapshot, ConnectionError> {
        // Reject malformed targets before disturbing an existing valid connection.
        let target = transport::Target::parse(request.target)?;
        let (generation, cancellation) = {
            let mut state = self.inner.lock().expect("connection state poisoned");
            state.advance()?;
            state.snapshot.phase = ConnectionPhase::Connecting;
            state.snapshot.endpoint = Some(target.endpoint.clone());
            state.snapshot.remote = target.remote;
            (state.generation, state.cancellation.clone())
        };
        let result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => Err(ConnectionError::Superseded),
            result = self.establish(&target, request.credentials, request.expected_backend_id.as_deref(), generation, cancellation.clone()) => result,
        };
        let mut state = self.inner.lock().expect("connection state poisoned");
        if state.generation != generation {
            return Err(ConnectionError::Superseded);
        }
        match result {
            Ok((lease, backend)) => {
                state.lease = Some(lease);
                state.snapshot.backend = Some(backend);
                state.snapshot.phase = ConnectionPhase::Connected;
                Ok(state.snapshot.clone())
            }
            Err(error) => {
                state.cancellation.cancel();
                state.snapshot.phase = ConnectionPhase::Failed;
                state.snapshot.error = Some(error.clone());
                Err(error)
            }
        }
    }

    async fn establish(
        &self,
        target: &transport::Target,
        credentials: ConnectionCredentials,
        expected_id: Option<&str>,
        generation: u64,
        cancellation: CancellationToken,
    ) -> Result<(SessionLease, BackendSnapshot), ConnectionError> {
        let (token, remember) = match credentials {
            ConnectionCredentials::None => (None, false),
            ConnectionCredentials::Saved => {
                let endpoint = target.endpoint.clone();
                let token = self
                    .access_credentials(generation, &cancellation, move |store| {
                        store.read(&endpoint)
                    })
                    .await?
                    .ok_or(ConnectionError::CredentialsUnavailable)?;
                (Some(token), false)
            }
            ConnectionCredentials::Bearer { token, remember } => {
                (Some(Zeroizing::new(token)), remember)
            }
        };
        let client = dokime::client::Client::connect(
            target.inner.clone(),
            token.as_ref().map(|token| token.as_str()),
            expected_id,
        )
        .await
        .map_err(ConnectionError::from)?;
        let backend = transport::backend(client.backend());
        let lease = SessionLease {
            generation: generation.to_string(),
            client,
            cancellation,
        };
        if remember && let Some(token) = token {
            let endpoint = target.endpoint.clone();
            self.access_credentials(generation, &lease.cancellation, move |store| {
                store.write(&endpoint, token.as_str())
            })
            .await?;
        }
        Ok((lease, backend))
    }

    async fn access_credentials<T: Send + 'static>(
        &self,
        generation: u64,
        cancellation: &CancellationToken,
        operation: impl FnOnce(&dyn SecretStore) -> Result<T, ConnectionError> + Send + 'static,
    ) -> Result<T, ConnectionError> {
        let ownership = Arc::clone(&self.secret_operations).lock_owned().await;
        let store = Arc::clone(&self.secrets);
        let state = Arc::clone(&self.inner);
        let cancellation = cancellation.clone();
        tokio::task::spawn_blocking(move || {
            // Cancellation cannot stop an OS keyring operation already in progress.
            // Keep serialization owned by the blocking work, even if its caller
            // drops the JoinHandle. Newer writes must run after that operation.
            let _ownership = ownership;
            if cancellation.is_cancelled()
                || state.lock().expect("connection state poisoned").generation != generation
            {
                return Err(ConnectionError::Superseded);
            }
            operation(store.as_ref())
        })
        .await
        .map_err(|_| ConnectionError::CredentialsUnavailable)?
    }
}

impl SessionState {
    fn advance(&mut self) -> Result<(), ConnectionError> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(ConnectionError::GenerationExhausted)?;
        self.cancellation.cancel();
        if let Some(lease) = self.lease.take() {
            lease.client.close();
        }
        self.generation = generation;
        self.cancellation = CancellationToken::new();
        self.snapshot.generation = generation.to_string();
        self.snapshot.backend = None;
        self.snapshot.error = None;
        Ok(())
    }
}

impl Drop for SessionState {
    fn drop(&mut self) {
        self.cancellation.cancel();
        if let Some(lease) = self.lease.take() {
            lease.client.close();
        }
    }
}

impl SessionLease {
    pub(crate) fn client(&self) -> &dokime::client::Client {
        &self.client
    }

    pub fn generation(&self) -> &str {
        &self.generation
    }

    pub(crate) async fn guard<T>(
        &self,
        future: impl Future<Output = T>,
    ) -> Result<T, ConnectionError> {
        let result = tokio::select! {
            biased;
            _ = self.cancellation.cancelled() => return Err(ConnectionError::Superseded),
            result = self.client.guard(future) => result.map_err(ConnectionError::from)?,
        };
        if self.cancellation.is_cancelled() {
            return Err(ConnectionError::Superseded);
        }
        Ok(result)
    }

    pub(crate) fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    pub async fn execute<T>(
        &self,
        request: impl Future<Output = Result<T, tonic::Status>>,
    ) -> Result<T, ConnectionError> {
        self.guard(request).await?.map_err(errors::rpc_error)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
