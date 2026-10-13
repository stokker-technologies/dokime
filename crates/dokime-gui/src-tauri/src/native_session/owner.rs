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
use std::sync::{Arc, Mutex};
// Third Party Imports

// Local Imports
use super::backend::{Backend, Process};
use super::{
    BackendOwnership, BackendSelection, DesktopSessionError as Error, DesktopSessionSnapshot,
};
use crate::connection::{
    ConnectRequest, ConnectionCredentials, ConnectionSession, ConnectionTarget,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct DesktopSession(Arc<Inner>);

struct Inner {
    connection: ConnectionSession,
    state: Mutex<State>,
    transition: tokio::sync::Mutex<Resources>,
    local: Option<Result<Backend, String>>,
}

struct State {
    revision: u64,
    closing: bool,
    ownership: BackendOwnership,
    phase: Option<String>,
}

#[derive(Default)]
struct Resources {
    local: Option<Process>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl DesktopSession {
    pub fn attached(connection: ConnectionSession) -> Self {
        Self(Arc::new(Inner {
            connection,
            state: Mutex::new(State {
                revision: 0,
                closing: false,
                ownership: BackendOwnership::None,
                phase: None,
            }),
            transition: tokio::sync::Mutex::new(Resources::default()),
            local: None,
        }))
    }
    pub fn with_local_backend(connection: ConnectionSession, backend: Backend) -> Self {
        Self::with_local_configuration(connection, Ok(backend))
    }
    pub fn with_local_configuration(
        connection: ConnectionSession,
        backend: Result<Backend, String>,
    ) -> Self {
        let mut owner = Self::attached(connection);
        Arc::get_mut(&mut owner.0).expect("new owner").local = Some(backend);
        owner
    }
    pub fn status(&self) -> DesktopSessionSnapshot {
        let state = self.0.state.lock().expect("desktop owner poisoned");
        DesktopSessionSnapshot {
            ownership: state.ownership.clone(),
            phase: state.phase.clone(),
            connection: self.0.connection.status(),
        }
    }
    fn report_phase(&self, revision: u64, phase: &str) {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.revision == revision {
            state.phase = Some(phase.to_owned());
        }
    }
    fn clear_phase(&self, revision: u64) {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.revision == revision {
            state.phase = None;
        }
    }
    pub async fn select(
        &self,
        selection: BackendSelection,
    ) -> Result<DesktopSessionSnapshot, Error> {
        // Resolve The Shared Profile Before Replacing An Active Session.
        let selection = match selection {
            BackendSelection::Profile { name } => {
                let store = dokime_client::profiles::Store::discover(None).map_err(|error| {
                    Error::Startup {
                        message: error.to_string(),
                    }
                })?;
                let document = store.load().map_err(|error| Error::Startup {
                    message: error.to_string(),
                })?;
                let profile = document.get(&name).map_err(|error| Error::Startup {
                    message: error.to_string(),
                })?;
                let resolved = profile.selection().map_err(|error| Error::Startup {
                    message: error.to_string(),
                })?;
                let target = if resolved.target.is_remote() {
                    ConnectionTarget::Remote {
                        endpoint: profile.endpoint.clone(),
                        ca_certificate_pem: profile.ca_certificate_pem.clone(),
                    }
                } else {
                    ConnectionTarget::Local {
                        endpoint: profile.endpoint.clone(),
                    }
                };
                let credentials = match resolved.credential {
                    dokime_client::profiles::Credential::None => ConnectionCredentials::None,
                    dokime_client::profiles::Credential::Keyring => ConnectionCredentials::Saved,
                };
                BackendSelection::Attach {
                    request: ConnectRequest {
                        target,
                        credentials,
                        expected_backend_id: resolved.expected_backend_id,
                    },
                }
            }
            selection => selection,
        };
        if let BackendSelection::Attach { request } = &selection {
            ConnectionSession::validate_target(request.target.clone())
                .map_err(|error| Error::Connection { error })?;
        }
        if matches!(selection, BackendSelection::OwnedLocal) {
            self.backend()?;
        }
        let revision = self.begin(false)?;
        let owner = self.clone();
        let result = tokio::spawn(async move { owner.select_owned(revision, selection).await })
            .await
            .map_err(|error| Error::Startup {
                message: error.to_string(),
            })?;
        // Startup is over, one way or another: whatever phase it last
        // reported no longer describes anything a caller should show. The
        // success path already clears it before publishing its snapshot;
        // this covers every way `select_owned` can fail instead.
        if result.is_err() {
            self.clear_phase(revision);
        }
        result
    }
    pub async fn disconnect(&self) -> Result<DesktopSessionSnapshot, Error> {
        self.stop(false).await
    }
    pub async fn shutdown(&self) -> Result<DesktopSessionSnapshot, Error> {
        self.stop(true).await
    }
    fn backend(&self) -> Result<Backend, Error> {
        self.0
            .local
            .clone()
            .ok_or(Error::LocalBackendUnavailable)?
            .map_err(|message| Error::Startup { message })
    }
    fn begin(&self, closing: bool) -> Result<u64, Error> {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.closing && !closing {
            return Err(Error::ShuttingDown);
        }
        state.revision = state.revision.checked_add(1).ok_or(Error::ShuttingDown)?;
        state.closing |= closing;
        state.ownership = BackendOwnership::None;
        state.phase = None;
        self.0
            .connection
            .disconnect()
            .map_err(|error| Error::Connection { error })?;
        Ok(state.revision)
    }
    fn current(&self, revision: u64) -> Result<(), Error> {
        if self
            .0
            .state
            .lock()
            .expect("desktop owner poisoned")
            .revision
            == revision
        {
            Ok(())
        } else {
            Err(Error::Superseded)
        }
    }
    async fn stop(&self, closing: bool) -> Result<DesktopSessionSnapshot, Error> {
        let revision = self.begin(closing)?;
        let owner = self.clone();
        tokio::spawn(async move {
            let mut resources = owner.0.transition.lock().await;
            resources.shutdown().await?;
            owner.current(revision)?;
            Ok(owner.status())
        })
        .await
        .map_err(|error| Error::Shutdown {
            message: error.to_string(),
        })?
    }
    async fn select_owned(
        &self,
        revision: u64,
        selection: BackendSelection,
    ) -> Result<DesktopSessionSnapshot, Error> {
        let mut resources = self.0.transition.lock().await;
        self.current(revision)?;
        resources.shutdown().await?;
        self.current(revision)?;
        let (request, ownership) = match selection {
            BackendSelection::Profile { .. } => unreachable!("profile resolved before transition"),
            BackendSelection::Attach { request } => (request, BackendOwnership::Attached),
            BackendSelection::OwnedLocal => {
                let backend = self.backend()?;
                let (process, endpoint) = super::backend::start(&backend, move |phase| {
                    self.report_phase(revision, phase)
                })
                .await
                .map_err(|message| Error::Startup { message })?;
                resources.local = Some(process);
                (
                    ConnectRequest {
                        target: ConnectionTarget::Local { endpoint },
                        credentials: ConnectionCredentials::None,
                        expected_backend_id: None,
                    },
                    BackendOwnership::OwnedLocal,
                )
            }
        };
        if let Err(error) = self.current(revision) {
            resources.shutdown().await?;
            return Err(error);
        }
        let connected = self.0.connection.connect(request).await;
        if let Err(error) = self.current(revision) {
            self.0
                .connection
                .disconnect()
                .map_err(|error| Error::Connection { error })?;
            resources.shutdown().await?;
            return Err(error);
        }
        match connected {
            Ok(_) => {
                let snapshot = {
                    let mut state = self.0.state.lock().expect("desktop owner poisoned");
                    if state.revision == revision {
                        state.ownership = ownership.clone();
                        // Startup is over: whatever phase it last reported no
                        // longer describes anything a caller should show.
                        state.phase = None;
                        Some(DesktopSessionSnapshot {
                            ownership,
                            phase: None,
                            connection: self.0.connection.status(),
                        })
                    } else {
                        None
                    }
                };
                if let Some(snapshot) = snapshot {
                    return Ok(snapshot);
                }
                self.0
                    .connection
                    .disconnect()
                    .map_err(|error| Error::Connection { error })?;
                resources.shutdown().await?;
                Err(Error::Superseded)
            }
            Err(error) => {
                resources.shutdown().await?;
                Err(Error::Connection { error })
            }
        }
    }
}

impl Resources {
    async fn shutdown(&mut self) -> Result<(), Error> {
        if let Some(local) = self.local.as_mut() {
            local
                .shutdown()
                .await
                .map_err(|message| Error::Shutdown { message })?;
            self.local = None;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
