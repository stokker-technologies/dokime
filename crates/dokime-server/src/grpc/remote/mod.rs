/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Persistent Remote Listener Ownership And Rollback

/* === Modules === */

mod operations;
pub(super) mod sessions;
mod store;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{path::PathBuf, sync::Arc};

// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use zeroize::Zeroizing;

// Local Imports
use super::{Config, Error, Server, Service, TlsIdentity};
use dokime::RuntimeHandle;
use dokime::domain::model::remote::{GeneratedAccessKey, RemoteListenerSettings, RemoteListenerStatus, RemoteSessionPage};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Controller {
    service: Service,
    path: PathBuf,
    sessions: Arc<sessions::SessionLog>,
    state: Mutex<State>,
}

struct State {
    document: store::Document,
    server: Option<Server>,
    error: Option<String>,
    closing: bool,
}

/* === Enums === */

enum Change {
    Settings(RemoteListenerSettings),
    Key,
    Certificate(Vec<String>),
}

/* === Traits === */

/* === Implementations === */

impl State {
    fn status(&self) -> RemoteListenerStatus {
        RemoteListenerStatus {
            settings: Some(self.document.settings.clone()),
            listening_address: self.server.as_ref().map(|server| server.address().to_string()),
            certificate_pem: self.document.identity.as_ref().map(|identity| identity.certificate.clone()),
            key_configured: self.document.key_digest.is_some(),
            error: self.error.clone(),
        }
    }
}

impl Controller {
    pub(super) async fn start(&self) {
        let mut state = self.state.lock().await;
        state.closing = false;
        if state.server.is_some() {
            return;
        }
        let started = match self.config(&state.document) {
            Ok(Some(config)) => self.service.listen_with(config).await.map(Some),
            Ok(None) => Ok(None),
            Err(error) => Err(error),
        };
        match started {
            Ok(server) => {
                state.server = server;
                state.error = None;
            }
            Err(error) => {
                tracing::error!(%error, "saved remote listener could not start");
                state.error = Some(error.to_string());
            }
        }
    }

    pub(super) async fn stop(&self) -> Result<(), Error> {
        let mut state = self.state.lock().await;
        state.closing = true;
        let sessions = self.sessions.clone();
        tokio::task::spawn_blocking(move || sessions.flush()).await?;
        if let Some(server) = state.server.take() {
            server.detach().await?;
        }
        Ok(())
    }

    fn sessions(&self, cursor: Option<&str>, limit: u32) -> Result<RemoteSessionPage, Error> {
        self.sessions.list(cursor, limit).map_err(Error::Configuration)
    }

    async fn status(&self) -> RemoteListenerStatus {
        self.state.lock().await.status()
    }

    fn config(&self, document: &store::Document) -> Result<Option<Config>, Error> {
        let settings = &document.settings;
        let address = store::validate(settings)?;
        if !settings.enabled {
            return Ok(None);
        }
        let digest = if settings.authentication_enabled {
            Some(document.key_digest.ok_or(Error::Configuration("generate a remote access key before enabling authentication"))?)
        } else {
            None
        };
        let tls = if settings.tls_enabled {
            let identity =
                document.identity.as_ref().ok_or(Error::Configuration("configure certificate identities before enabling TLS"))?;
            Some(TlsIdentity::from_pem(identity.certificate.as_bytes().to_vec(), identity.private_key.as_bytes().to_vec()))
        } else {
            None
        };
        Ok(Some(Config::managed_remote(address, tls, digest, self.service.message_limit())))
    }

    async fn persist(&self, document: &store::Document) -> Result<(), Error> {
        let document = document.clone();
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || store::write(&path, &document)).await?
    }

    async fn change(self: &Arc<Self>, change: Change) -> Result<GeneratedAccessKey, Error> {
        // Own The Whole Transition After Client Cancellation, Including Rollback.
        let controller = self.clone();
        tokio::spawn(async move { controller.apply(change).await }).await?
    }

    async fn apply(&self, change: Change) -> Result<GeneratedAccessKey, Error> {
        let mut state = self.state.lock().await;
        if state.closing {
            return Err(harmos::Error::Closing { domain: "remote listener management" }.into());
        }
        let mut candidate = state.document.clone();
        let mut access_key = Zeroizing::new(String::new());
        match change {
            Change::Settings(settings) => {
                store::validate(&settings)?;
                if settings == candidate.settings && state.error.is_none() && state.server.is_some() == settings.enabled {
                    return Ok(GeneratedAccessKey { access_key: String::new(), listener: Some(state.status()) });
                }
                let mut settings = settings;
                if let Some(identity) = &candidate.identity {
                    if identity.names != settings.certificate_names {
                        return Err(Error::Configuration("certificate identities changed; explicitly regenerate the certificate first"));
                    }
                } else if settings.tls_enabled || !settings.certificate_names.is_empty() {
                    // TLS Never Waits On Typed Names; The Host's Own Identities Seed The Certificate.
                    if settings.certificate_names.is_empty() {
                        settings.certificate_names = store::host_names(store::validate(&settings)?);
                    }
                    candidate.identity = Some(store::Identity::generate(settings.certificate_names.clone())?);
                }
                candidate.settings = settings;
            }
            Change::Key => {
                let random = Zeroizing::new(rand::random::<[u8; 32]>());
                use std::fmt::Write;
                for byte in random.iter() {
                    write!(access_key, "{byte:02x}").expect("writing to a string succeeds");
                }
                candidate.key_digest = Some(Sha256::digest(access_key.as_bytes()).into());
            }
            Change::Certificate(names) => {
                candidate.settings.certificate_names = names;
                let address = store::validate(&candidate.settings)?;
                if candidate.settings.certificate_names.is_empty() {
                    candidate.settings.certificate_names = store::host_names(address);
                }
                candidate.identity = Some(store::Identity::generate(candidate.settings.certificate_names.clone())?);
            }
        }
        let next = self.config(&candidate)?;
        let previous = match &state.server {
            Some(server) => self.config(&state.document)?.map(|config| config.with_bind_addr(server.address())),
            None => None,
        };

        // Detach Only Remote Transport; Runtime Work And Native Access Retain Ownership.
        if let Some(server) = state.server.take()
            && let Err(error) = server.detach().await
        {
            return self.restore(&mut state, previous, error).await;
        }
        let server = match next {
            Some(config) => match self.service.listen_with(config).await {
                Ok(server) => Some(server),
                Err(error) => return self.restore(&mut state, previous, error).await,
            },
            None => None,
        };
        if let Err(mut error) = self.persist(&candidate).await {
            if let Some(server) = server
                && let Err(cleanup) = server.detach().await
            {
                error = Error::RemoteRollback { change: Box::new(error), restoration: Box::new(cleanup) };
            }
            // An Atomic Rename May Have Succeeded Before A Directory Sync Failed.
            if let Err(restoration) = self.persist(&state.document).await {
                let failure = Error::RemoteRollback { change: Box::new(error), restoration: Box::new(restoration) };
                return self.restore(&mut state, previous, failure).await;
            }
            return self.restore(&mut state, previous, error).await;
        }

        state.document = candidate;
        state.server = server;
        state.error = None;
        tracing::info!(
            enabled = state.document.settings.enabled,
            tls = state.document.settings.tls_enabled,
            authentication = state.document.settings.authentication_enabled,
            "remote listener settings applied"
        );
        Ok(GeneratedAccessKey { access_key: access_key.to_string(), listener: Some(state.status()) })
    }

    async fn restore(&self, state: &mut State, previous: Option<Config>, change: Error) -> Result<GeneratedAccessKey, Error> {
        if let Some(config) = previous {
            match self.service.listen_with(config).await {
                Ok(server) => state.server = Some(server),
                Err(restoration) => {
                    let error = Error::RemoteRollback { change: Box::new(change), restoration: Box::new(restoration) };
                    state.error = Some(error.to_string());
                    return Err(error);
                }
            }
        }
        state.error = Some(change.to_string());
        Err(change)
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, service: Service) -> Result<Arc<Controller>, Error> {
    let path = runtime.application_settings().application_root.join("remote").join("access.json");
    let read_path = path.clone();
    let document = tokio::task::spawn_blocking(move || store::read(&read_path)).await??;
    let sessions = service.sessions();
    let log = sessions.clone();
    let log_path = path.with_file_name("sessions.json");
    tokio::task::spawn_blocking(move || log.attach(log_path)).await?;
    spawn_session_flush(Arc::downgrade(&sessions));
    let controller =
        Arc::new(Controller { service, path, sessions, state: Mutex::new(State { document, server: None, error: None, closing: false }) });
    operations::register(runtime, controller.clone())?;
    Ok(controller)
}

/* === Private Functions === */

// Calls Only Mark The Log; Writes Are Batched So Authentication Never Waits On Storage.
fn spawn_session_flush(sessions: std::sync::Weak<sessions::SessionLog>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            let Some(log) = sessions.upgrade() else { return };
            if tokio::task::spawn_blocking(move || log.flush()).await.is_err() {
                return;
            }
        }
    });
}

/* === Tests === */
