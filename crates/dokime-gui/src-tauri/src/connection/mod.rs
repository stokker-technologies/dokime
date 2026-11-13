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
pub mod host;
pub mod registry;
pub(crate) mod transport;

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

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct ConnectionSession {
    preferences: Arc<Mutex<dokime_sdk::v1::ClientPreferences>>,
    inner: Arc<Mutex<SessionState>>,
}

struct SessionState {
    generation: u64,
    cancellation: CancellationToken,
    snapshot: ConnectionSnapshot,
    lease: Option<SessionLease>,
    profile: Option<String>,
    retired: Vec<host::Session>,
}

#[derive(Clone)]
pub struct SessionLease {
    preferences: Arc<Mutex<dokime_sdk::v1::ClientPreferences>>,
    client: host::Session,
    cancellation: CancellationToken,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for ConnectionSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionSession {
    pub(crate) fn set_preferences(&self, preferences: dokime_sdk::v1::ClientPreferences) {
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

    fn new() -> Self {
        let settings = host::desktop_settings();
        Self {
            preferences: Arc::new(Mutex::new(dokime_sdk::v1::ClientPreferences {
                connect_timeout_ms: settings.connect_timeout.as_millis() as u64,
                request_timeout_ms: settings.services.read_timeout.as_millis() as u64,
            })),
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
                retired: Vec::new(),
            })),
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

    pub(crate) async fn join_retired(&self) {
        let retired = std::mem::take(&mut self.inner.lock().expect("connection state poisoned").retired);
        for session in retired {
            session.join().await;
        }
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
        self.join_retired().await;
        let result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => Err(ConnectionError::Superseded),
            result = self.establish(&target, request.expected_backend_id.as_deref(), cancellation.clone()) => result,
        };
        let mut state = self.inner.lock().expect("connection state poisoned");
        if state.generation != generation {
            return Err(ConnectionError::Superseded);
        }
        match result {
            Ok((lease, backend)) => {
                state.snapshot.endpoint = Some(lease.client.endpoint().to_owned());
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
        &self, target: &transport::Target, expected_id: Option<&str>, cancellation: CancellationToken,
    ) -> Result<(SessionLease, BackendSnapshot), ConnectionError> {
        let mut settings = host::desktop_settings();
        let preferences = *self.preferences.lock().expect("client preferences poisoned");
        settings.connect_timeout = std::time::Duration::from_millis(preferences.connect_timeout_ms);
        settings.services.read_timeout = std::time::Duration::from_millis(preferences.request_timeout_ms);
        let client = host::Session::connect_target(&target.target, &settings).await.map_err(|error| {
            tracing::warn!(%error, "Connect to Host Failed");
            ConnectionError::from(error)
        })?;
        let backend = transport::backend(&client);
        if expected_id.is_some_and(|expected| expected != backend.backend_id) {
            client.cancel();
            return Err(ConnectionError::IdentityChanged);
        }
        Ok((
            SessionLease {
                preferences: self.preferences.clone(),
                client,
                cancellation,
            },
            backend,
        ))
    }
}

impl SessionState {
    fn advance(&mut self) -> Result<(), ConnectionError> {
        let generation = self.generation.checked_add(1).ok_or(ConnectionError::GenerationExhausted)?;
        self.cancellation.cancel();
        if let Some(lease) = self.lease.take() {
            lease.client.cancel();
            self.retired.push(lease.client);
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
            lease.client.cancel();
        }
    }
}

impl SessionLease {
    pub(crate) fn client(&self) -> host::Session {
        let timeout_ms = self.preferences.lock().expect("client preferences poisoned").request_timeout_ms;
        self.client.with_timeout(std::time::Duration::from_millis(timeout_ms))
    }

    pub(crate) async fn guard<T>(&self, future: impl Future<Output = T>) -> Result<T, ConnectionError> {
        let result = tokio::select! {
            biased;
            _ = self.cancellation.cancelled() => return Err(ConnectionError::Superseded),
            result = future => result,
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
