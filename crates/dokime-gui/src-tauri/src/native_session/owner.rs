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
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

// Third Party Imports

// Local Imports
use super::backend::{Backend, Process};
use super::storage::StorageRoot;
use super::{BackendOwnership, BackendSelection, DesktopSessionError as Error, DesktopSessionSnapshot, StorageResetReason};
use crate::connection::{ConnectRequest, ConnectionCredentials, ConnectionSession, ConnectionTarget};

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
    // Cancelled once, when the operator exits without finishing.
    abandon: tokio_util::sync::CancellationToken,
}

struct State {
    revision: u64,
    closing: bool,
    ownership: BackendOwnership,
    phase: Option<String>,
    archived: Option<u64>,
    // The log the last failed owned start reported; the only file the shell
    // opens on the WebView's behalf.
    startup_log: Option<PathBuf>,
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
                archived: None,
                startup_log: None,
            }),
            transition: tokio::sync::Mutex::new(Resources::default()),
            local: None,
            abandon: tokio_util::sync::CancellationToken::new(),
        }))
    }
    pub fn with_local_backend(connection: ConnectionSession, backend: Backend) -> Self {
        Self::with_local_configuration(connection, Ok(backend))
    }
    pub fn with_local_configuration(connection: ConnectionSession, backend: Result<Backend, String>) -> Self {
        let mut owner = Self::attached(connection);
        Arc::get_mut(&mut owner.0).expect("new owner").local = Some(backend);
        owner
    }
    pub fn status(&self) -> DesktopSessionSnapshot {
        let state = self.0.state.lock().expect("desktop owner poisoned");
        DesktopSessionSnapshot {
            ownership: state.ownership.clone(),
            phase: state.phase.clone(),
            archived_bytes: state.archived,
            connection: self.0.connection.status(),
        }
    }
    fn report_phase(&self, revision: u64, phase: &str) {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.revision == revision {
            state.phase = Some(phase.to_owned());
            state.archived = None;
        }
    }
    fn report_archived(&self, revision: u64, bytes: u64) {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.revision == revision {
            state.archived = Some(bytes);
        }
    }
    pub fn startup_log(&self) -> Option<PathBuf> {
        let state = self.0.state.lock().expect("desktop owner poisoned");
        state.startup_log.clone()
    }
    fn fail(&self, revision: u64, error: &Error) {
        let error = match error {
            Error::Archived { error, .. } => error.as_ref(),
            error => error,
        };
        let log = match error {
            Error::Startup { diagnostics: Some(diagnostics), .. } => diagnostics.log.as_deref().map(PathBuf::from),
            _ => None,
        };
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.revision == revision {
            state.phase = None;
            state.archived = None;
            state.startup_log = log;
        }
    }
    pub async fn select(&self, selection: BackendSelection) -> Result<DesktopSessionSnapshot, Error> {
        // Resolve The Shared Profile Before Replacing An Active Session.
        let selection = match selection {
            BackendSelection::Profile { name } => {
                let store = dokime_client::profiles::Store::discover(None)
                    .map_err(|error| Error::Startup { message: error.to_string(), diagnostics: None })?;
                let document = store.load().map_err(|error| Error::Startup { message: error.to_string(), diagnostics: None })?;
                let profile = document.get(&name).map_err(|error| Error::Startup { message: error.to_string(), diagnostics: None })?;
                let resolved = profile.selection().map_err(|error| Error::Startup { message: error.to_string(), diagnostics: None })?;
                let target = if resolved.target.is_remote() {
                    ConnectionTarget::Remote { endpoint: profile.endpoint.clone(), ca_certificate_pem: profile.ca_certificate_pem.clone() }
                } else {
                    ConnectionTarget::Local { endpoint: profile.endpoint.clone() }
                };
                let credentials = match resolved.credential {
                    dokime_client::profiles::Credential::None => ConnectionCredentials::None,
                    dokime_client::profiles::Credential::Keyring => ConnectionCredentials::Saved,
                };
                BackendSelection::Attach {
                    request: ConnectRequest { target, credentials, expected_backend_id: resolved.expected_backend_id },
                }
            }
            selection => selection,
        };
        if let BackendSelection::Attach { request } = &selection {
            ConnectionSession::validate_target(request.target.clone()).map_err(|error| Error::Connection { error })?;
        }
        if matches!(selection, BackendSelection::OwnedLocal) {
            self.backend()?;
        }
        let revision = self.begin(false)?;
        let owner = self.clone();
        let result = tokio::spawn(async move { owner.select_owned(revision, selection).await })
            .await
            .map_err(|error| Error::Startup { message: error.to_string(), diagnostics: None })?;
        // Startup is over, one way or another: whatever phase it last
        // reported no longer describes anything a caller should show. The
        // success path already clears it before publishing its snapshot;
        // this covers every way `select_owned` can fail instead, and keeps
        // the failed start's log for the shell to open.
        if let Err(error) = &result {
            self.fail(revision, error);
        }
        result
    }
    pub async fn reset_local_storage(&self, confirmed: &Path, root: StorageRoot) -> Result<DesktopSessionSnapshot, Error> {
        // Refuse Before Touching The Running Backend
        if self.status().ownership != BackendOwnership::OwnedLocal {
            return Err(Error::StorageReset {
                reason: StorageResetReason::NotOwned,
                message: "Only a backend this desktop started can have its storage reset.".to_owned(),
            });
        }
        root.verify(confirmed)?;
        self.backend()?;

        // Reset Under A New Revision
        let revision = self.begin(false)?;
        let owner = self.clone();
        let result = tokio::spawn(async move { owner.reset_owned(revision, root, None).await })
            .await
            .map_err(|error| Error::Startup { message: error.to_string(), diagnostics: None })?;
        if let Err(error) = &result {
            self.fail(revision, error);
        }
        result
    }
    pub async fn backup_and_reset_local_storage(&self, root: StorageRoot, destination: PathBuf) -> Result<DesktopSessionSnapshot, Error> {
        // Refuse While Any Backend Is Connected
        self.backend()?;
        if self.status().ownership != BackendOwnership::None {
            return Err(Error::StorageReset {
                reason: StorageResetReason::NotOwned,
                message: "Local storage can be backed up and reset only after the local backend failed to start.".to_owned(),
            });
        }
        root.verify(root.path())?;

        // Back Up And Reset Under A New Revision
        let revision = self.begin(false)?;
        let owner = self.clone();
        let result = tokio::spawn(async move { owner.reset_owned(revision, root, Some(destination)).await })
            .await
            .map_err(|error| Error::Startup { message: error.to_string(), diagnostics: None })?;
        if let Err(error) = &result {
            self.fail(revision, error);
        }
        result
    }
    pub async fn disconnect(&self) -> Result<DesktopSessionSnapshot, Error> {
        self.stop(false).await
    }
    pub async fn shutdown(&self) -> Result<DesktopSessionSnapshot, Error> {
        self.stop(true).await
    }
    // End the owned backend now, without the rest of its orderly stop.
    pub fn abandon(&self) {
        self.0.abandon.cancel();
    }
    fn backend(&self) -> Result<Backend, Error> {
        self.0.local.clone().ok_or(Error::LocalBackendUnavailable)?.map_err(|message| Error::Startup { message, diagnostics: None })
    }
    fn begin(&self, closing: bool) -> Result<u64, Error> {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.closing && !closing {
            return Err(Error::ShuttingDown);
        }
        state.revision = state.revision.checked_add(1).ok_or(Error::ShuttingDown)?;
        state.closing |= closing;
        state.phase = None;
        state.archived = None;
        state.startup_log = None;
        // A closing desktop keeps an established connection until its backend
        // has stopped: the window goes on showing what it showed, and nothing
        // it still asks is answered as superseded. A connection still being
        // made is abandoned at once.
        if !state.closing || state.ownership == BackendOwnership::None {
            state.ownership = BackendOwnership::None;
            self.0.connection.disconnect().map_err(|error| Error::Connection { error })?;
        }
        Ok(state.revision)
    }
    fn current(&self, revision: u64) -> Result<(), Error> {
        if self.0.state.lock().expect("desktop owner poisoned").revision == revision { Ok(()) } else { Err(Error::Superseded) }
    }
    async fn stop(&self, closing: bool) -> Result<DesktopSessionSnapshot, Error> {
        let revision = self.begin(closing)?;
        let owner = self.clone();
        tokio::spawn(async move {
            let mut resources = owner.0.transition.lock().await;
            resources.shutdown(&owner.0.abandon).await?;
            owner.current(revision)?;
            if closing {
                owner.0.state.lock().expect("desktop owner poisoned").ownership = BackendOwnership::None;
                owner.0.connection.disconnect().map_err(|error| Error::Connection { error })?;
            }
            Ok(owner.status())
        })
        .await
        .map_err(|error| Error::Shutdown { message: error.to_string() })?
    }
    async fn select_owned(&self, revision: u64, selection: BackendSelection) -> Result<DesktopSessionSnapshot, Error> {
        let mut resources = self.0.transition.lock().await;
        self.current(revision)?;
        resources.shutdown(&self.0.abandon).await?;
        self.current(revision)?;
        self.establish(revision, selection, &mut resources).await
    }
    async fn establish(
        &self, revision: u64, selection: BackendSelection, resources: &mut Resources,
    ) -> Result<DesktopSessionSnapshot, Error> {
        let (request, ownership) = match selection {
            BackendSelection::Profile { .. } => unreachable!("profile resolved before transition"),
            BackendSelection::Attach { request } => (request, BackendOwnership::Attached),
            BackendSelection::OwnedLocal => {
                let backend = self.backend()?;
                let (process, endpoint) = super::backend::start(&backend, move |phase| self.report_phase(revision, phase)).await?;
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
            resources.shutdown(&self.0.abandon).await?;
            return Err(error);
        }
        let connected = self.0.connection.connect(request).await;
        if let Err(error) = self.current(revision) {
            self.0.connection.disconnect().map_err(|error| Error::Connection { error })?;
            resources.shutdown(&self.0.abandon).await?;
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
                        state.archived = None;
                        Some(DesktopSessionSnapshot {
                            ownership,
                            phase: None,
                            archived_bytes: None,
                            connection: self.0.connection.status(),
                        })
                    } else {
                        None
                    }
                };
                if let Some(snapshot) = snapshot {
                    return Ok(snapshot);
                }
                self.0.connection.disconnect().map_err(|error| Error::Connection { error })?;
                resources.shutdown(&self.0.abandon).await?;
                Err(Error::Superseded)
            }
            Err(error) => {
                resources.shutdown(&self.0.abandon).await?;
                Err(Error::Connection { error })
            }
        }
    }
    async fn reset_owned(&self, revision: u64, root: StorageRoot, backup: Option<PathBuf>) -> Result<DesktopSessionSnapshot, Error> {
        // Stop The Owned Backend
        // The transition lock stays held until the new backend is ready, so
        // no other selection starts a backend on a half-deleted root.
        let mut resources = self.0.transition.lock().await;
        self.current(revision)?;
        self.report_phase(revision, "stopping");
        resources.shutdown(&self.0.abandon).await?;
        self.current(revision)?;

        // Archive The Data Root First, When Asked
        let Some(destination) = backup else {
            return self.clear_and_restart(revision, root, &mut resources).await;
        };
        self.report_phase(revision, "archiving");
        let owner = self.clone();
        let source = root.clone();
        let archive = tokio::task::spawn_blocking(move || source.archive(&destination, |bytes| owner.report_archived(revision, bytes)))
            .await
            .map_err(|error| Error::StorageReset { reason: StorageResetReason::Failed, message: error.to_string() })??;
        tracing::warn!(archive = %archive.display(), "local storage archived");
        self.clear_and_restart(revision, root, &mut resources)
            .await
            .map_err(|error| Error::Archived { archive: archive.display().to_string(), error: Box::new(error) })
    }
    async fn clear_and_restart(
        &self, revision: u64, root: StorageRoot, resources: &mut Resources,
    ) -> Result<DesktopSessionSnapshot, Error> {
        // Delete The Contents Of The Data Root
        self.current(revision)?;
        self.report_phase(revision, "deleting");
        let path = root.path().to_path_buf();
        tracing::warn!(root = %path.display(), "resetting local storage");
        tokio::task::spawn_blocking(move || root.clear())
            .await
            .map_err(|error| Error::StorageReset { reason: StorageResetReason::Failed, message: error.to_string() })?
            .inspect_err(|error| {
                tracing::error!(root = %path.display(), ?error, "local storage reset refused");
            })?;
        tracing::warn!(root = %path.display(), "local storage deleted");

        // Start Again On The Empty Root
        self.current(revision)?;
        self.report_phase(revision, "starting");
        self.establish(revision, BackendSelection::OwnedLocal, resources).await
    }
}

impl Resources {
    async fn shutdown(&mut self, abandon: &tokio_util::sync::CancellationToken) -> Result<(), Error> {
        if let Some(local) = self.local.as_mut() {
            let status = local.shutdown(abandon).await.map_err(|message| Error::Shutdown { message })?;
            // The process has exited either way; a retry has nothing left to wait for.
            self.local = None;
            if !status.success() {
                return Err(Error::Shutdown {
                    message: format!(
                        "The {} backend did not stop cleanly ({status}); its next start seals what it left unfinished.",
                        crate::PRODUCT
                    ),
                });
            }
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
