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
use super::backend::Backend;
use super::storage::StorageRoot;
use super::{BackendOwnership, BackendSelection, DesktopSessionError as Error, DesktopSessionSnapshot, StorageResetReason};
use crate::connection::{ConnectRequest, ConnectionSecrets, ConnectionSession, ConnectionTarget};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct DesktopSession(Arc<Inner>);

struct Inner {
    connection: ConnectionSession,
    state: Mutex<State>,
    status_updates: tokio::sync::watch::Sender<DesktopSessionSnapshot>,
    transition: tokio::sync::Mutex<Resources>,
    local: Option<Result<Backend, String>>,
    // Cancelled once, when the operator exits without finishing.
    abandon: tokio_util::sync::CancellationToken,
}

struct State {
    revision: u64,
    status_revision: u64,
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
    local: Option<dokime_server::host::Session>,
    settings_client: Option<dokime::client::Client>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl DesktopSession {
    pub fn attached(connection: ConnectionSession) -> Self {
        let initial = DesktopSessionSnapshot {
            status_revision: 0,
            ownership: BackendOwnership::None,
            phase: None,
            archived_bytes: None,
            connection: connection.status(),
        };
        let (status_updates, _) = tokio::sync::watch::channel(initial);
        Self(Arc::new(Inner {
            connection,
            state: Mutex::new(State {
                revision: 0,
                status_revision: 0,
                closing: false,
                ownership: BackendOwnership::None,
                phase: None,
                archived: None,
                startup_log: None,
            }),
            status_updates,
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
        self.snapshot(&state)
    }
    pub fn subscribe_status(&self) -> tokio::sync::watch::Receiver<DesktopSessionSnapshot> {
        self.0.status_updates.subscribe()
    }
    fn snapshot(&self, state: &State) -> DesktopSessionSnapshot {
        DesktopSessionSnapshot {
            status_revision: state.status_revision,
            ownership: state.ownership.clone(),
            phase: state.phase.clone(),
            archived_bytes: state.archived,
            connection: self.0.connection.status(),
        }
    }
    fn publish_status(&self, state: &mut State) -> DesktopSessionSnapshot {
        state.status_revision = state.status_revision.saturating_add(1);
        let snapshot = self.snapshot(state);
        self.0.status_updates.send_replace(snapshot.clone());
        snapshot
    }
    fn report_phase(&self, revision: u64, phase: &str) {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.revision == revision {
            state.phase = Some(phase.to_owned());
            state.archived = None;
            self.publish_status(&mut state);
        }
    }
    fn report_archived(&self, revision: u64, bytes: u64) {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if state.revision == revision {
            state.archived = Some(bytes);
            self.publish_status(&mut state);
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
            self.publish_status(&mut state);
        }
    }
    pub async fn select(&self, selection: BackendSelection) -> Result<DesktopSessionSnapshot, Error> {
        let selected = if let BackendSelection::Profile { identifier } = &selection {
            let document = super::super::connection::registry::load().await.map_err(|error| Error::Connection { error })?;
            Some(
                dokime::client::profiles::get(&document, identifier)
                    .map_err(|error| Error::Startup {
                        message: error.to_string(),
                        diagnostics: None,
                    })?
                    .clone(),
            )
        } else {
            None
        };
        let selection = if let Some(profile) = &selected {
            let target = dokime::client::Target::new(&profile.endpoint, profile.ca_certificate_pem.clone())
                .map_err(|error| Error::Connection { error: error.into() })?;
            let target = if target.is_remote() {
                ConnectionTarget::Remote {
                    endpoint: profile.endpoint.clone(),
                    ca_certificate_pem: profile.ca_certificate_pem.clone(),
                }
            } else {
                ConnectionTarget::Local { endpoint: profile.endpoint.clone() }
            };
            let secrets = match profile.secret {
                dokime::client::profiles::Secret::None => ConnectionSecrets::None,
                dokime::client::profiles::Secret::Keyring => ConnectionSecrets::Saved,
            };
            BackendSelection::Attach {
                request: ConnectRequest {
                    target,
                    secrets,
                    expected_backend_id: profile.expected_backend_id.clone(),
                },
            }
        } else {
            selection
        };
        if let BackendSelection::Attach { request } = &selection {
            ConnectionSession::validate_target(request.target.clone()).map_err(|error| Error::Connection { error })?;
        }
        if matches!(selection, BackendSelection::OwnedLocal) {
            self.backend()?;
        }
        let revision = self.begin(false)?;
        let owner = self.clone();
        let result = tokio::spawn(async move { owner.select_owned(revision, selection).await }).await.map_err(|error| Error::Startup {
            message: error.to_string(),
            diagnostics: None,
        })?;
        // Startup is over, one way or another: whatever phase it last
        // reported no longer describes anything a caller should show. The
        // success path already clears it before publishing its snapshot;
        // this covers every way `select_owned` can fail instead, and keeps
        // the failed start's log for the shell to open.
        if let Err(error) = &result {
            self.fail(revision, error);
        }
        if let (Ok(snapshot), Some(profile)) = (&result, selected)
            && let Err(error) = crate::connection::registry::remember(&self.0.connection, &snapshot.connection.generation, profile).await
        {
            if self.current(revision).is_ok() {
                self.0.connection.disconnect().map_err(|error| Error::Connection { error })?;
                let failure = Error::Connection { error };
                self.fail(revision, &failure);
                return Err(failure);
            }
            return Err(Error::Superseded);
        }
        result
    }
    async fn local_preferences(
        &self, resources: &mut Resources,
    ) -> Result<dokime::domain::model::settings::application::ClientPreferences, Error> {
        if resources.local.is_none() {
            let backend = self.backend()?;
            let (local, _) = super::backend::start(&backend, |_| {}).await?;
            resources.local = Some(local);
        }
        let endpoint = resources.local.as_ref().expect("local owner initialized").endpoint();
        let target = dokime::client::Target::local(endpoint).map_err(|error| Error::Connection { error: error.into() })?;
        let client =
            dokime::client::Client::connect(target, None, None).await.map_err(|error| Error::Connection { error: error.into() })?;
        let result = client.settings_read_application_settings(dokime::domain::operations::settings::ReadApplicationSettings {}).await;
        client.close();
        result.map(|snapshot| snapshot.settings.client).map_err(|error| Error::Connection { error: error.into() })
    }
    pub fn remove_profile(&self, identifier: &str) -> Result<(), Error> {
        let mut state = self.0.state.lock().expect("desktop owner poisoned");
        if self.0.connection.disconnect_profile(identifier).map_err(|error| Error::Connection { error })? {
            state.revision = state.revision.checked_add(1).ok_or(Error::Superseded)?;
            state.ownership = BackendOwnership::None;
            state.phase = None;
            self.publish_status(&mut state);
        }
        Ok(())
    }
    pub async fn local_client(&self) -> Result<dokime::client::Client, Error> {
        let mut resources = self.0.transition.lock().await;
        if resources.local.is_none() {
            let backend = self.backend()?;
            let (local, _) = super::backend::start(&backend, |_| {}).await?;
            resources.local = Some(local);
        }
        if let Some(client) = &resources.settings_client {
            return Ok(client.clone());
        }
        let endpoint = resources.local.as_ref().expect("local owner initialized").endpoint();
        let target = dokime::client::Target::local(endpoint).map_err(|error| Error::Connection { error: error.into() })?;
        let client =
            dokime::client::Client::connect(target, None, None).await.map_err(|error| Error::Connection { error: error.into() })?;
        resources.settings_client = Some(client.clone());
        Ok(client)
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
        let result = tokio::spawn(async move { owner.reset_owned(revision, root, None).await }).await.map_err(|error| Error::Startup {
            message: error.to_string(),
            diagnostics: None,
        })?;
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
        let result = tokio::spawn(async move { owner.reset_owned(revision, root, Some(destination)).await }).await.map_err(|error| {
            Error::Startup {
                message: error.to_string(),
                diagnostics: None,
            }
        })?;
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
        self.publish_status(&mut state);
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
            if closing {
                resources.shutdown(&owner.0.abandon).await?;
            }
            owner.current(revision)?;
            if closing {
                let mut state = owner.0.state.lock().expect("desktop owner poisoned");
                state.ownership = BackendOwnership::None;
                owner.0.connection.disconnect().map_err(|error| Error::Connection { error })?;
                owner.publish_status(&mut state);
            }
            Ok(owner.status())
        })
        .await
        .map_err(|error| Error::Shutdown { message: error.to_string() })?
    }
    async fn select_owned(&self, revision: u64, selection: BackendSelection) -> Result<DesktopSessionSnapshot, Error> {
        let mut resources = self.0.transition.lock().await;
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
                if resources.local.is_none() {
                    let backend = self.backend()?;
                    let (process, _) = super::backend::start(&backend, move |phase| self.report_phase(revision, phase)).await?;
                    resources.local = Some(process);
                }
                let endpoint = resources.local.as_ref().expect("local owner initialized").endpoint();
                (
                    ConnectRequest {
                        target: ConnectionTarget::Local { endpoint },
                        secrets: ConnectionSecrets::None,
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
        let local = self.local_preferences(resources).await?;
        self.0.connection.set_preferences(local);
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
                        Some(self.publish_status(&mut state))
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
            .map_err(|error| Error::StorageReset {
                reason: StorageResetReason::Failed,
                message: error.to_string(),
            })??;
        tracing::warn!(archive = %archive.display(), "local storage archived");
        self.clear_and_restart(revision, root, &mut resources).await.map_err(|error| Error::Archived {
            archive: archive.display().to_string(),
            error: Box::new(error),
        })
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
            .map_err(|error| Error::StorageReset {
                reason: StorageResetReason::Failed,
                message: error.to_string(),
            })?
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
        if let Some(client) = self.settings_client.take() {
            client.close_and_wait().await;
        }
        if let Some(local) = self.local.as_mut() {
            tokio::select! {
                result = local.shutdown() => result.map_err(|error| Error::Shutdown { message: error.to_string() })?,
                () = abandon.cancelled() => {
                    tracing::warn!("exiting without finishing the embedded server drain");
                    return Err(Error::Shutdown { message: "The local server drain was interrupted.".to_owned() });
                }
            }
            self.local = None;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn status_watch_publishes_lifecycle_changes_and_ignores_stale_callbacks() {
        let owner = DesktopSession::attached(ConnectionSession::default());
        let mut updates = owner.subscribe_status();
        assert_eq!(updates.borrow_and_update().status_revision, 0);

        let revision = owner.begin(false).unwrap();
        let started = next_status(&mut updates).await;
        assert_eq!(started.status_revision, 1);

        owner.report_phase(revision, "storage");
        let phase = next_status(&mut updates).await;
        assert_eq!(phase.phase.as_deref(), Some("storage"));
        assert_eq!(phase.status_revision, 2);

        owner.report_archived(revision, 4096);
        let archived = next_status(&mut updates).await;
        assert_eq!(archived.archived_bytes, Some(4096));
        assert_eq!(archived.status_revision, 3);

        owner.fail(
            revision,
            &Error::Startup {
                message: "failed".to_owned(),
                diagnostics: None,
            },
        );
        let failed = next_status(&mut updates).await;
        assert_eq!(failed.phase, None);
        assert_eq!(failed.archived_bytes, None);
        assert_eq!(failed.status_revision, 4);

        let next_revision = owner.begin(false).unwrap();
        let restarted = next_status(&mut updates).await;
        assert!(restarted.status_revision > failed.status_revision);
        owner.report_phase(revision, "stale");
        assert!(!updates.has_changed().unwrap());

        owner.report_phase(next_revision, "plugins");
        let latest = next_status(&mut updates).await;
        assert_eq!(latest.phase.as_deref(), Some("plugins"));
        assert!(latest.status_revision > restarted.status_revision);
    }

    async fn next_status(updates: &mut tokio::sync::watch::Receiver<DesktopSessionSnapshot>) -> DesktopSessionSnapshot {
        updates.changed().await.unwrap();
        updates.borrow_and_update().clone()
    }
}
