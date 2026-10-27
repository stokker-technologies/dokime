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
mod errors;
pub mod registry;
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
use dokime::client::secrets::{KeyringStore, SecretStore};
use dokime::domain::model::settings::application::ClientPreferences;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct ConnectionSession {
    preferences: Arc<Mutex<ClientPreferences>>,
    inner: Arc<Mutex<SessionState>>,
    secrets: Arc<dyn SecretStore>,
    secret_operations: Arc<tokio::sync::Mutex<()>>,
}

struct SessionState {
    generation: u64,
    cancellation: CancellationToken,
    snapshot: ConnectionSnapshot,
    lease: Option<SessionLease>,
    profile: Option<String>,
}

#[derive(Clone)]
pub struct SessionLease {
    generation: String,
    client: dokime::client::Client,
    cancellation: CancellationToken,
    preferences: Arc<Mutex<ClientPreferences>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for ConnectionSession {
    fn default() -> Self {
        Self::with_secrets(Arc::new(KeyringStore))
    }
}

impl ConnectionSession {
    pub(crate) fn preferences(&self) -> ClientPreferences {
        *self.preferences.lock().expect("client preferences poisoned")
    }

    pub(crate) fn set_preferences(&self, preferences: ClientPreferences) {
        *self.preferences.lock().expect("client preferences poisoned") = preferences;
    }

    pub(crate) fn disconnect_profile(&self, name: &str) -> Result<bool, ConnectionError> {
        let mut state = self.inner.lock().expect("connection state poisoned");
        if state.profile.as_deref() != Some(name) {
            return Ok(false);
        }
        state.advance()?;
        state.snapshot.phase = ConnectionPhase::Disconnected;
        state.snapshot.endpoint = None;
        state.snapshot.remote = false;
        Ok(true)
    }

    pub(crate) fn selected_profile(&self, generation: &str, name: String) -> Result<(), ConnectionError> {
        let mut state = self.inner.lock().expect("connection state poisoned");
        if state.snapshot.generation != generation || state.lease.is_none() {
            return Err(ConnectionError::Superseded);
        }
        state.profile = Some(name);
        Ok(())
    }

    pub fn with_secrets(secrets: Arc<dyn SecretStore>) -> Self {
        Self::with_secrets_and_settings(secrets, ClientPreferences::default())
    }

    fn with_secrets_and_settings(secrets: Arc<dyn SecretStore>, settings: ClientPreferences) -> Self {
        Self {
            preferences: Arc::new(Mutex::new(settings)),
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
                profile: None,
            })),
            secrets,
            secret_operations: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub(crate) fn validate_target(target: ConnectionTarget) -> Result<(), ConnectionError> {
        transport::Target::parse(target).map(|_| ())
    }

    pub fn status(&self) -> ConnectionSnapshot {
        self.inner.lock().expect("connection state poisoned").snapshot.clone()
    }

    pub fn lease(&self, generation: &str) -> Result<SessionLease, ConnectionError> {
        let state = self.inner.lock().expect("connection state poisoned");
        if state.snapshot.generation != generation {
            return Err(ConnectionError::Superseded);
        }
        state.lease.clone().ok_or(ConnectionError::NotConnected)
    }

    // Keep A Final Native Side Effect Serialized With Generation Invalidation.
    pub(crate) fn with_current<T>(&self, generation: &str, action: impl FnOnce() -> T) -> Result<T, ConnectionError> {
        let state = self.inner.lock().expect("connection state poisoned");
        if state.snapshot.generation != generation {
            return Err(ConnectionError::Superseded);
        }
        if state.lease.is_none() {
            return Err(ConnectionError::NotConnected);
        }
        let result = action();
        drop(state);
        Ok(result)
    }

    pub fn disconnect(&self) -> Result<ConnectionSnapshot, ConnectionError> {
        let mut state = self.inner.lock().expect("connection state poisoned");
        state.advance()?;
        state.snapshot.phase = ConnectionPhase::Disconnected;
        state.snapshot.endpoint = None;
        state.snapshot.remote = false;
        Ok(state.snapshot.clone())
    }

    pub async fn connect(&self, request: ConnectRequest) -> Result<ConnectionSnapshot, ConnectionError> {
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
            result = self.establish(&target, request.secrets, request.expected_backend_id.as_deref(), generation, cancellation.clone()) => result,
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
        &self, target: &transport::Target, secrets: ConnectionSecrets, expected_id: Option<&str>, generation: u64,
        cancellation: CancellationToken,
    ) -> Result<(SessionLease, BackendSnapshot), ConnectionError> {
        let (token, remember) = match secrets {
            ConnectionSecrets::None => (None, false),
            ConnectionSecrets::Saved => {
                let endpoint = target.endpoint.clone();
                let token = self
                    .access_secrets(generation, &cancellation, move |store| {
                        store.read(&endpoint).map_err(|_| ConnectionError::SecretsUnavailable)
                    })
                    .await?
                    .ok_or(ConnectionError::SecretsUnavailable)?;
                (Some(token), false)
            }
            ConnectionSecrets::Bearer { token, remember } => (Some(Zeroizing::new(token)), remember),
        };
        let client = dokime::client::Client::connect(
            target.inner.clone().with_settings(self.preferences()),
            token.as_ref().map(|token| token.as_str()),
            expected_id,
        )
        .await
        .map_err(ConnectionError::from)?;
        let backend = transport::backend(client.backend());
        let lease = SessionLease { generation: generation.to_string(), client, cancellation, preferences: Arc::clone(&self.preferences) };
        if remember && let Some(token) = token {
            let endpoint = target.endpoint.clone();
            self.access_secrets(generation, &lease.cancellation, move |store| {
                store.write(&endpoint, token.as_str()).map_err(|_| ConnectionError::SecretsUnavailable)
            })
            .await?;
        }
        Ok((lease, backend))
    }

    async fn access_secrets<T: Send + 'static>(
        &self, generation: u64, cancellation: &CancellationToken,
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
            if cancellation.is_cancelled() || state.lock().expect("connection state poisoned").generation != generation {
                return Err(ConnectionError::Superseded);
            }
            operation(store.as_ref())
        })
        .await
        .map_err(|_| ConnectionError::SecretsUnavailable)?
    }
}

impl SessionState {
    fn advance(&mut self) -> Result<(), ConnectionError> {
        let generation = self.generation.checked_add(1).ok_or(ConnectionError::GenerationExhausted)?;
        self.cancellation.cancel();
        if let Some(lease) = self.lease.take() {
            lease.client.close();
        }
        self.generation = generation;
        self.cancellation = CancellationToken::new();
        self.snapshot.generation = generation.to_string();
        self.profile = None;
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
    pub(crate) fn client(&self) -> dokime::client::Client {
        let preferences = *self.preferences.lock().expect("client preferences poisoned");
        self.client.with_timeout(std::time::Duration::from_millis(preferences.request_timeout_ms))
    }

    pub fn generation(&self) -> &str {
        &self.generation
    }

    pub(crate) async fn guard<T>(&self, future: impl Future<Output = T>) -> Result<T, ConnectionError> {
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

    pub async fn execute<T>(&self, request: impl Future<Output = Result<T, tonic::Status>>) -> Result<T, ConnectionError> {
        self.guard(request).await?.map_err(errors::rpc_error)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
