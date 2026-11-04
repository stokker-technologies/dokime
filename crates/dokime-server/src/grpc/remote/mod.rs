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

pub(super) mod operations;
pub(super) mod pairing;
pub(super) mod sessions;
mod store;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{path::PathBuf, sync::Arc};

// Third Party Imports
use tokio::sync::Mutex;

// Local Imports
use super::{Config, Error, Server, Service, TlsIdentity};
use dokime::RuntimeHandle;
use dokime::domain::model::remote::{RemoteListenerSettings, RemoteListenerStatus, RemoteSessionPage};

/* === Constants === */

const SERVER_REMOTE_HEARTBEAT_MS: u64 = 5_000;
const SERVER_REMOTE_NAMES: u64 = 32;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Controller {
    runtime: RuntimeHandle,
    service: Service,
    path: PathBuf,
    sessions: Arc<sessions::SessionLog>,
    state: Mutex<State>,
    clients: Arc<pairing::Registry>,
    refresh_stop: tokio::sync::watch::Sender<bool>,
    refresh_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

struct State {
    document: store::Document,
    server: Option<Server>,
    bound: Option<Config>,
    error: Option<String>,
    closing: bool,
}

/* === Enums === */

enum Change {
    Settings(RemoteListenerSettings),
    Certificate(Vec<String>),
    Refresh,
}

/* === Traits === */

/* === Implementations === */

impl State {
    fn status(&self) -> RemoteListenerStatus {
        RemoteListenerStatus {
            settings: Some(self.document.settings.clone()),
            listening_address: self.server.as_ref().map(|server| server.address().to_string()),
            certificate_pem: self.document.identity.as_ref().map(|identity| identity.certificate.clone()),
            key_configured: !self.document.clients.is_empty(),
            error: self.error.clone(),
            endpoints: self
                .server
                .as_ref()
                .map(|server| server.address())
                .or_else(|| store::validate(&self.document.settings).ok())
                .map(|address| store::host_endpoints(address, SERVER_REMOTE_NAMES as usize))
                .unwrap_or_default(),
        }
    }
}

impl Controller {
    pub(super) async fn start(self: &Arc<Self>) {
        let mut task = self.refresh_task.lock().await;
        let mut state = self.state.lock().await;
        state.closing = false;
        if task.is_none() {
            self.refresh_stop.send_replace(false);
            let mut stopping = self.refresh_stop.subscribe();
            let weak = Arc::downgrade(self);
            let mut shutdown = self.runtime.watch_state();
            *task = Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(SERVER_REMOTE_HEARTBEAT_MS));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                loop {
                    tokio::select! {
                        biased;
                        _ = stopping.wait_for(|stopping| *stopping) => return,
                        _ = shutdown.wait_until(|snapshot| snapshot.state().shutting_down) => return,
                        _ = interval.tick() => {}
                    }
                    let Some(controller) = weak.upgrade() else { return };
                    if let Err(error) = controller.apply(Change::Refresh).await {
                        tracing::error!(%error, "remote certificate names could not be refreshed");
                    }
                    if let Err(error) = controller.flush_clients().await {
                        tracing::error!(%error, "paired client timestamps could not be saved");
                    }
                }
            }));
        }
        drop(task);
        if state.server.is_some() {
            return;
        }
        if state.document.settings.enabled {
            let mut candidate = state.document.clone();
            let refreshed = async {
                refresh_identity(&mut candidate)?;
                self.write_security(&candidate).await
            }
            .await;
            if let Err(error) = refreshed {
                state.error = Some(error.to_string());
                return;
            }
            state.document = candidate;
        }
        let started = match self.config(&state.document) {
            Ok(Some(config)) => {
                let bound = config.clone();
                match self.service.listen_with(config).await {
                    Ok(server) => {
                        state.bound = Some(bound);
                        Ok(Some(server))
                    }
                    Err(error) => Err(error),
                }
            }
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
        let mut task = self.refresh_task.lock().await;
        self.refresh_stop.send_replace(true);
        if let Some(task) = task.as_mut() {
            task.await?;
        }
        task.take();
        self.flush_clients().await?;
        let mut state = self.state.lock().await;
        state.closing = true;
        let sessions = self.sessions.clone();
        tokio::task::spawn_blocking(move || sessions.flush()).await?;
        if let Some(server) = state.server.as_mut() {
            server.detach_listener().await?;
            state.server = None;
        }
        state.bound = None;
        Ok(())
    }

    fn sessions(&self, cursor: Option<&str>, limit: u32) -> Result<RemoteSessionPage, Error> {
        self.sessions.list(cursor, limit).map_err(Error::Configuration)
    }

    async fn status(&self) -> RemoteListenerStatus {
        let mut status = self.state.lock().await.status();
        match self.runtime.read_application_settings().await {
            Ok(snapshot) => {
                if status.listening_address.is_none() {
                    status.endpoints = store::validate(&snapshot.settings.listener)
                        .ok()
                        .map(|address| store::host_endpoints(address, SERVER_REMOTE_NAMES as usize))
                        .unwrap_or_default();
                }
                status.settings = Some(snapshot.settings.listener);
            }
            Err(error) => status.error = Some(error.to_string()),
        }
        status
    }

    fn config(&self, document: &store::Document) -> Result<Option<Config>, Error> {
        let settings = &document.settings;
        let address = store::validate(settings)?;
        if !settings.enabled {
            return Ok(None);
        }
        let policy = if settings.authentication_enabled {
            super::access::Policy::Clients(self.clients.clone())
        } else {
            super::access::Policy::Open
        };
        let tls = if settings.tls_enabled {
            let identity =
                document.identity.as_ref().ok_or(Error::Configuration("configure certificate identities before enabling TLS"))?;
            Some(TlsIdentity::from_pem(identity.certificate.as_bytes().to_vec(), identity.private_key.as_bytes().to_vec()))
        } else {
            None
        };
        Ok(Some(Config::managed_remote(address, tls, policy, self.service.message_limit())))
    }

    async fn write_security(&self, document: &store::Document) -> Result<(), Error> {
        let document = document.clone();
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || store::write(&path, &document)).await?
    }

    async fn persist(
        &self, document: &store::Document, previous: &store::Document, snapshot: dokime::domain::model::settings::wire::SettingsSnapshot,
    ) -> Result<(), Error> {
        let saved_listener = document.settings.clone();
        let result = async {
            self.write_security(document).await?;
            if snapshot.settings.listener != saved_listener {
                let mut settings = snapshot.settings;
                settings.listener = saved_listener;
                self.runtime
                    .update_application_settings(dokime::domain::model::settings::wire::UpdateSettings {
                        expected_revision: snapshot.revision,
                        settings,
                    })
                    .await?;
            }
            Ok::<_, Error>(())
        }
        .await;
        if let Err(change) = result {
            return match self.write_security(previous).await {
                Ok(()) => Err(change),
                Err(restoration) => Err(Error::RemoteRollback {
                    change: Box::new(change),
                    restoration: Box::new(restoration),
                }),
            };
        }
        Ok(())
    }

    async fn change(self: &Arc<Self>, change: Change) -> Result<RemoteListenerStatus, Error> {
        // Own The Whole Transition After Client Cancellation, Including Rollback.
        let controller = self.clone();
        tokio::spawn(async move {
            let result = controller.apply(change).await;
            if let Err(error) = &result {
                controller.state.lock().await.error = Some(error.to_string());
            }
            result
        })
        .await?
    }

    async fn apply(&self, change: Change) -> Result<RemoteListenerStatus, Error> {
        let mut state = self.state.lock().await;
        if state.closing {
            return Err(dokime::harmos::Error::Closing { domain: "remote listener management" }.into());
        }
        let snapshot = self.runtime.read_application_settings().await?;
        let mut candidate = state.document.clone();
        candidate.settings = snapshot.settings.listener.clone();
        candidate.clients = self.clients.records();
        match change {
            Change::Settings(settings) => {
                store::validate(&settings)?;
                if settings == state.document.settings && state.error.is_none() && state.server.is_some() == settings.enabled {
                    return Ok(state.status());
                }
                candidate.settings = settings;
                refresh_identity(&mut candidate)?;
            }
            Change::Certificate(names) => {
                candidate.settings.certificate_names = names;
                let detected = store::host_names(store::validate(&candidate.settings)?, SERVER_REMOTE_NAMES as usize);
                let names = certificate_names(&candidate.settings.certificate_names, detected)?;
                match &mut candidate.identity {
                    Some(identity) => identity.reissue(names)?,
                    None => candidate.identity = Some(store::Identity::generate(names)?),
                }
            }
            Change::Refresh => {
                // Address And Access Changes Remain Pending Until Their Explicit Commit.
                if listener_changed(&state.document.settings, &candidate.settings) || !candidate.settings.enabled {
                    return Ok(state.status());
                }
                let previous_names = candidate.identity.as_ref().map(|identity| identity.names.clone());
                refresh_identity(&mut candidate)?;
                if previous_names == candidate.identity.as_ref().map(|identity| identity.names.clone())
                    && state.server.is_some() == candidate.settings.enabled
                {
                    return Ok(state.status());
                }
            }
        }
        let next = self.config(&candidate)?;
        let rebind =
            listener_changed(&state.document.settings, &candidate.settings) || state.server.is_some() != candidate.settings.enabled;
        let previous = state.bound.clone();
        if let Err(error) = self.persist(&candidate, &state.document, snapshot).await {
            state.error = Some(error.to_string());
            return Err(error);
        }
        if !rebind {
            if let (Some(server), Some(identity)) = (&state.server, next.as_ref().and_then(|config| config.tls.as_ref()))
                && let Err(change) = server.reload_certificate(identity)
            {
                return match self.persist(&state.document, &candidate, self.runtime.read_application_settings().await?).await {
                    Ok(()) => Err(change),
                    Err(restoration) => Err(Error::RemoteRollback {
                        change: Box::new(change),
                        restoration: Box::new(restoration),
                    }),
                };
            }
            state.document = candidate;
            state.bound = next;
            state.error = None;
            return Ok(state.status());
        }
        state.document = candidate.clone();

        // Detach Only Remote Transport; Runtime Work And Native Access Retain Ownership.
        if let Some(server) = state.server.take()
            && let Err(error) = server.detach().await
        {
            return self.restore(&mut state, previous, error).await;
        }
        state.bound = None;
        let next_bound = next.clone();
        let server = match next {
            Some(config) => match self.service.listen_with(config).await {
                Ok(server) => Some(server),
                Err(error) => return self.restore(&mut state, previous, error).await,
            },
            None => None,
        };
        state.document = candidate;
        state.server = server;
        state.bound = next_bound;
        state.error = None;
        tracing::info!(
            enabled = state.document.settings.enabled,
            tls = state.document.settings.tls_enabled,
            authentication = state.document.settings.authentication_enabled,
            "remote listener settings applied"
        );
        Ok(state.status())
    }

    async fn restore(&self, state: &mut State, previous: Option<Config>, change: Error) -> Result<RemoteListenerStatus, Error> {
        if let Some(config) = previous {
            let bound = config.clone();
            match self.service.listen_with(config).await {
                Ok(server) => {
                    state.server = Some(server);
                    state.bound = Some(bound);
                }
                Err(restoration) => {
                    let error = Error::RemoteRollback {
                        change: Box::new(change),
                        restoration: Box::new(restoration),
                    };
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

pub(super) async fn activate(runtime: &RuntimeHandle, service: Service) -> Result<Arc<Controller>, Error> {
    let path = runtime.application_root().join("remote").join("access.json");
    let read_path = path.clone();
    let settings = runtime.read_application_settings().await?.settings.listener;
    let document = tokio::task::spawn_blocking(move || store::read(&read_path, settings)).await??;
    let sessions = service.sessions();
    let log = sessions.clone();
    let log_path = path.with_file_name("sessions.json");
    tokio::task::spawn_blocking(move || log.attach(log_path)).await?;
    spawn_session_flush(Arc::downgrade(&sessions), std::time::Duration::from_millis(SERVER_REMOTE_HEARTBEAT_MS));
    let clients = Arc::new(pairing::Registry::new(document.clients.clone()));
    let controller = Arc::new(Controller {
        runtime: runtime.clone(),
        service,
        path,
        sessions,
        clients,
        refresh_stop: tokio::sync::watch::channel(false).0,
        refresh_task: Mutex::new(None),
        state: Mutex::new(State {
            document,
            server: None,
            bound: None,
            error: None,
            closing: false,
        }),
    });
    Ok(controller)
}

/* === Private Functions === */

fn refresh_identity(document: &mut store::Document) -> Result<(), Error> {
    if !document.settings.tls_enabled {
        return Ok(());
    }
    let detected = store::host_names(store::validate(&document.settings)?, SERVER_REMOTE_NAMES as usize);
    refresh_identity_for_names(document, detected)
}

fn refresh_identity_for_names(document: &mut store::Document, detected: Vec<String>) -> Result<(), Error> {
    let required = certificate_names(&document.settings.certificate_names, detected)?;
    if document
        .identity
        .as_ref()
        .is_some_and(|identity| required.iter().all(|name| identity.names.iter().any(|covered| covered.eq_ignore_ascii_case(name))))
    {
        return Ok(());
    }
    let mut names = required;
    if let Some(identity) = &document.identity {
        for name in &identity.names {
            if names.len() < SERVER_REMOTE_NAMES as usize && !names.iter().any(|covered| covered.eq_ignore_ascii_case(name)) {
                names.push(name.clone());
            }
        }
    }
    match &mut document.identity {
        Some(identity) if identity.names == names => {}
        Some(identity) => identity.reissue(names)?,
        None => document.identity = Some(store::Identity::generate(names)?),
    }
    Ok(())
}

fn listener_changed(previous: &RemoteListenerSettings, next: &RemoteListenerSettings) -> bool {
    previous.enabled != next.enabled
        || previous.address != next.address
        || previous.tls_enabled != next.tls_enabled
        || previous.authentication_enabled != next.authentication_enabled
}

fn certificate_names(configured: &[String], advertised: Vec<String>) -> Result<Vec<String>, Error> {
    let mut names: Vec<String> = Vec::new();
    for name in configured.iter().chain(advertised.iter()) {
        if !names.iter().any(|covered| covered.eq_ignore_ascii_case(name)) {
            names.push(name.clone());
        }
    }
    if names.len() > SERVER_REMOTE_NAMES as usize {
        return Err(Error::Configuration("configured and advertised certificate names exceed the listener name limit"));
    }
    Ok(names)
}

// Calls Only Mark The Log; Writes Are Batched So Authentication Never Waits On Storage.
fn spawn_session_flush(sessions: std::sync::Weak<sessions::SessionLog>, heartbeat: std::time::Duration) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(heartbeat);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_preserves_covered_names_across_address_flaps_and_reordering() {
        let mut document = store::Document::initial();
        document.settings.tls_enabled = true;
        document.settings.certificate_names = vec!["lab.example".into()];
        let hosts = vec!["192.168.1.10".into(), "192.168.1.20".into()];
        refresh_identity_for_names(&mut document, hosts.clone()).unwrap();
        let certificate = document.identity.as_ref().unwrap().certificate.clone();
        let names = document.identity.as_ref().unwrap().names.clone();
        for detected in [vec![hosts[0].clone()], vec![hosts[1].clone(), hosts[0].clone()], hosts] {
            refresh_identity_for_names(&mut document, detected).unwrap();
            assert_eq!(document.identity.as_ref().unwrap().certificate, certificate);
            assert_eq!(document.identity.as_ref().unwrap().names, names);
        }
        document.settings.certificate_names = vec!["LAB.EXAMPLE".into()];
        refresh_identity_for_names(&mut document, vec!["192.168.1.10".into()]).unwrap();
        assert_eq!(document.identity.as_ref().unwrap().certificate, certificate);
        let ca = document.identity.as_ref().unwrap().ca_certificate.clone();
        refresh_identity_for_names(&mut document, vec!["192.168.1.30".into()]).unwrap();
        let identity = document.identity.as_ref().unwrap();
        assert_ne!(identity.certificate, certificate);
        assert_eq!(identity.ca_certificate, ca);
        assert!(identity.names.contains(&"192.168.1.10".into()));
        document.settings.certificate_names.push("new.example".into());
        refresh_identity_for_names(&mut document, vec!["192.168.1.30".into()]).unwrap();
        assert!(document.identity.as_ref().unwrap().names.contains(&"new.example".into()));
    }

    #[test]
    fn certificate_changes_do_not_require_rebinding() {
        let previous = store::Document::initial().settings;
        let mut next = previous.clone();
        next.certificate_names.push("lab.example".into());
        assert!(!listener_changed(&previous, &next));
        next.address = "127.0.0.1:8765".into();
        assert!(listener_changed(&previous, &next));
        next = previous.clone();
        next.tls_enabled = !previous.tls_enabled;
        assert!(listener_changed(&previous, &next));
        next = previous.clone();
        next.authentication_enabled = !previous.authentication_enabled;
        assert!(listener_changed(&previous, &next));
        next = previous.clone();
        next.enabled = !previous.enabled;
        assert!(listener_changed(&previous, &next));
    }
}
