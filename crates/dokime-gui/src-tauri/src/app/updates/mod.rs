/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod auth;
pub(crate) mod plugins;
mod sources;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

// Third Party Imports
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;
#[cfg(feature = "updater")]
use tauri_plugin_updater::UpdaterExt;

// Local Imports
use super::organization::{
    OrganizationState,
    policy::{Connection, Mode, Source, UpdatePolicy},
};

/* === Constants === */

const SETTINGS_BYTES: usize = 16 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct UpdateState(Arc<Shared>);

struct Shared {
    path: PathBuf,
    accounts: Arc<auth::Accounts>,
    inner: Mutex<Inner>,
}

struct Inner {
    policy: UpdatePolicy,
    generation: u64,
    error: Option<String>,
    last: Option<UpdateCheck>,
    plugin: Option<plugins::Candidate>,
    #[cfg(feature = "updater")]
    candidate: Option<Candidate>,
}

#[cfg(feature = "updater")]
struct Candidate {
    id: String,
    generation: u64,
    update: tauri_plugin_updater::Update,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Selection {
    pub mode: Mode,
    pub source: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub mode: Mode,
    pub source: Option<String>,
    pub sources: Vec<String>,
    pub connection: Option<String>,
    pub host: Option<String>,
    pub can_sign_in: bool,
    pub user: Option<auth::User>,
    pub last_check: Option<UpdateCheck>,
    pub error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub status: String,
    pub current_version: String,
    pub version: Option<String>,
    pub body: Option<String>,
    pub candidate: Option<String>,
}

#[derive(Serialize)]
pub struct InstallResult {
    pub version: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl UpdateState {
    fn status(&self) -> Result<UpdateStatus, String> {
        let inner = self.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        let connection = selected_connection(&inner.policy);
        let account = connection.as_ref().map(|(_, connection)| self.0.accounts.user(connection)).transpose();
        let (user, account_error) = match account {
            Ok(user) => (user.flatten(), None),
            Err(error) => (None, Some(error)),
        };
        Ok(UpdateStatus {
            mode: inner.policy.mode,
            source: inner.policy.source.clone(),
            sources: inner.policy.sources.keys().cloned().collect(),
            connection: connection.as_ref().map(|(name, _)| name.clone()),
            host: connection.as_ref().map(|(_, connection)| connection.host.clone()),
            can_sign_in: inner.policy.mode != Mode::Disabled
                && connection.as_ref().is_some_and(|(_, connection)| !connection.oauth_client_id.is_empty()),
            user,
            last_check: inner.last.clone(),
            error: inner.error.clone().or(account_error),
        })
    }

    fn snapshot(&self) -> Result<(UpdatePolicy, u64), String> {
        let inner = self.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        Ok((inner.policy.clone(), inner.generation))
    }

    fn invalidate(&self) -> Result<(), String> {
        let mut inner = self.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        inner.generation = inner.generation.wrapping_add(1);
        inner.last = None;
        inner.plugin = None;
        #[cfg(feature = "updater")]
        {
            inner.candidate = None;
        }
        Ok(())
    }

    fn configure(&self, selection: Selection) -> Result<(), String> {
        let mut inner = self.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        let mut policy = inner.policy.clone();
        policy.mode = selection.mode;
        policy.source = selection.source.clone();
        policy.validate()?;
        let bytes = serde_json::to_vec(&selection).map_err(|_| "Update selection could not be encoded")?;
        dokime::harmos::filesystem::publication::write(&self.0.path, &bytes).map_err(|_| "Update selection could not be saved")?;
        self.0.accounts.cancel(None);
        inner.policy = policy;
        inner.generation = inner.generation.wrapping_add(1);
        inner.last = None;
        inner.plugin = None;
        inner.error = None;
        #[cfg(feature = "updater")]
        {
            inner.candidate = None;
        }
        Ok(())
    }

    async fn authorization(&self, policy: &UpdatePolicy) -> Result<Option<sources::Authorization>, String> {
        let Some((_, connection)) = selected_connection(policy) else { return Ok(None) };
        let accounts = Arc::clone(&self.0.accounts);
        tokio::task::spawn_blocking(move || accounts.authorization(&connection))
            .await
            .map_err(|_| "Release account lookup failed".to_owned())?
    }

    #[cfg(feature = "updater")]
    async fn check(&self, app: &AppHandle) -> Result<UpdateCheck, String> {
        let (policy, generation) = self.snapshot()?;
        let current_version = app.package_info().version.to_string();
        if policy.mode == Mode::Disabled {
            return Ok(UpdateCheck { status: "disabled".into(), current_version, version: None, body: None, candidate: None });
        }
        let auth = self.authorization(&policy).await?;
        let manifest = sources::resolve(&policy, "latest.json", auth.as_ref()).await?;
        let headers = sources::headers(auth.as_ref(), &manifest)?;
        let updater = app
            .updater_builder()
            .endpoints(vec![manifest])
            .map_err(|_| "The update manifest URL is invalid")?
            .headers(headers)
            .configure_client(sources::client)
            .build()
            .map_err(|_| "The compiled updater is unavailable")?;
        let found = updater.check().await.map_err(|_| "The update manifest could not be checked")?;
        let mut inner = self.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        if inner.generation != generation {
            return Err("The update source changed while checking".into());
        }
        inner.candidate = None;
        let result = match found {
            Some(mut update) => {
                super::organization::policy::https_url(update.download_url.as_str())?;
                update.headers = sources::headers(auth.as_ref(), &update.download_url)?;
                update.timeout = Some(Duration::from_secs(300));
                let id = format!("{:032x}", rand::random::<u128>());
                let result = UpdateCheck {
                    status: "available".into(),
                    current_version,
                    version: Some(update.version.clone()),
                    body: update.body.clone(),
                    candidate: Some(id.clone()),
                };
                inner.candidate = Some(Candidate { id, generation, update });
                result
            }
            None => UpdateCheck { status: "up_to_date".into(), current_version, version: None, body: None, candidate: None },
        };
        inner.last = Some(result.clone());
        inner.error = None;
        Ok(result)
    }

    #[cfg(not(feature = "updater"))]
    async fn check(&self, _app: &AppHandle) -> Result<UpdateCheck, String> {
        Err("This distribution manages desktop updates outside the application".into())
    }

    fn automatic(&self, app: &AppHandle) {
        if self.snapshot().is_ok_and(|(policy, _)| policy.mode == Mode::AutomaticCheck) {
            let state = self.clone();
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let generation = state.snapshot().map(|(_, generation)| generation).ok();
                if let Err(error) = state.check(&app).await {
                    let mut inner = state.0.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    if Some(inner.generation) == generation {
                        inner.error = Some(error);
                    }
                }
            });
        }
    }
}

/* === Public Functions === */

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let root = super::settings::application_root(&app.config().identifier)?.join("release-sources");
    let path = root.join("selection.json");
    let mut policy = app.state::<OrganizationState>().update_policy()?;
    let mut error = None;
    match dokime::harmos::filesystem::input::read(&path, SETTINGS_BYTES) {
        Ok(bytes) => {
            let selection = serde_json::from_slice::<Selection>(&bytes);
            let selected = selection.map_err(|_| "Saved update selection is invalid".to_owned()).and_then(|selection| {
                let mut selected = policy.clone();
                selected.mode = selection.mode;
                selected.source = selection.source;
                selected.validate()?;
                Ok(selected)
            });
            match selected {
                Ok(selected) => policy = selected,
                Err(reason) => {
                    policy.mode = Mode::Disabled;
                    error = Some(reason);
                }
            }
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => {
            policy.mode = Mode::Disabled;
            error = Some("Saved update selection could not be read; updates are disabled".into());
        }
    }
    let accounts = Arc::new(auth::Accounts::open(root.join("accounts.json")));
    let state = UpdateState(Arc::new(Shared {
        path,
        accounts,
        inner: Mutex::new(Inner {
            policy,
            generation: 0,
            error,
            last: None,
            plugin: None,
            #[cfg(feature = "updater")]
            candidate: None,
        }),
    }));
    app.manage(state.clone());
    let deep_link = app.deep_link();
    deep_link.register_all()?;
    let callbacks = Arc::clone(&state.0.accounts);
    deep_link.on_open_url(move |record| {
        for url in record.urls() {
            let accounts = Arc::clone(&callbacks);
            tauri::async_runtime::spawn(async move {
                accounts.callback(url).await;
            });
        }
    });
    state.automatic(app.handle());
    Ok(())
}

pub async fn desktop_updates_status(state: State<'_, UpdateState>) -> Result<UpdateStatus, String> {
    state.status()
}

pub async fn desktop_updates_configure(
    app: AppHandle, state: State<'_, UpdateState>, selection: Selection,
) -> Result<UpdateStatus, String> {
    state.configure(selection)?;
    state.automatic(&app);
    state.status()
}

pub async fn desktop_updates_check(app: AppHandle, state: State<'_, UpdateState>) -> Result<UpdateCheck, String> {
    state.check(&app).await
}

pub async fn desktop_updates_sign_in(app: AppHandle, state: State<'_, UpdateState>) -> Result<UpdateStatus, String> {
    let (policy, generation) = state.snapshot()?;
    if policy.mode == Mode::Disabled {
        return Err("Updates are disabled".into());
    }
    let (_, connection) = selected_connection(&policy).ok_or("The selected source has no OAuth connection")?;
    let pending = state.0.accounts.begin(connection)?;
    let url = pending.url;
    let opened = tokio::task::spawn_blocking(move || app.opener().open_url(url, None::<&str>)).await;
    if !matches!(opened, Ok(Ok(()))) {
        state.0.accounts.cancel(Some(pending.epoch));
        return Err("The sign-in browser could not be opened".into());
    }
    let result = tokio::time::timeout(Duration::from_secs(180), pending.reply).await;
    state.0.accounts.cancel(Some(pending.epoch));
    result.map_err(|_| "Release sign-in timed out")?.map_err(|_| "Release sign-in was cancelled")??;
    if state.snapshot()?.1 != generation {
        return Err("The release connection changed while signing in".into());
    }
    state.invalidate()?;
    state.status()
}

pub async fn desktop_updates_sign_out(state: State<'_, UpdateState>) -> Result<UpdateStatus, String> {
    let (policy, _) = state.snapshot()?;
    let (_, connection) = selected_connection(&policy).ok_or("The selected source has no OAuth connection")?;
    let accounts = Arc::clone(&state.0.accounts);
    state.invalidate()?;
    tokio::task::spawn_blocking(move || accounts.sign_out(&connection)).await.map_err(|_| "Release sign-out failed")??;
    state.status()
}

pub async fn desktop_updates_install(state: State<'_, UpdateState>, candidate: String) -> Result<InstallResult, String> {
    #[cfg(feature = "updater")]
    {
        let (update, generation) = {
            let inner = state.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
            let selected =
                inner.candidate.as_ref().filter(|selected| selected.id == candidate).ok_or("The reviewed update is no longer available")?;
            (selected.update.clone(), selected.generation)
        };
        let bytes = update
            .download(|_, _| {}, || {})
            .await
            .map_err(|_| "The update download failed signature verification or could not be read")?;
        let mut inner = state.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        if inner.generation != generation || inner.candidate.as_ref().is_none_or(|selected| selected.id != candidate) {
            return Err("The reviewed update changed before installation".into());
        }
        update.install(bytes).map_err(|_| "The verified update could not be installed")?;
        inner.candidate = None;
        Ok(InstallResult { version: update.version })
    }
    #[cfg(not(feature = "updater"))]
    {
        let _ = (state, candidate);
        Err("This distribution manages desktop updates outside the application".into())
    }
}

/* === Private Functions === */

fn selected_connection(policy: &UpdatePolicy) -> Option<(String, Connection)> {
    let source = policy.sources.get(policy.source.as_ref()?)?;
    let name = match source {
        Source::Direct { connection, .. } => connection.as_ref()?,
        Source::Gitlab { connection, .. } => connection,
        Source::Github { .. } => return None,
    };
    Some((name.clone(), policy.connections.get(name)?.clone()))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "updater")]
    #[tokio::test]
    async fn downloaded_update_rejects_signature_with_compiled_trust() {
        use rustls::pki_types::PrivatePkcs8KeyDer;
        use std::io::{Read, Write};
        use tauri::test::{mock_builder, mock_context, noop_assets};

        let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let trust = reqwest::Certificate::from_der(certificate.cert.der()).unwrap();
        let key = PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.cert.der().clone()], key.into())
            .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("https://localhost:{}", listener.local_addr().unwrap().port());
        let manifest = serde_json::json!({
            "version": "999.0.0", "url": format!("{origin}/update"), "signature": "bm90IGEgc2lnbmF0dXJl"
        })
        .to_string();
        let runtime = tokio::runtime::Handle::current();
        let server = tokio::task::spawn_blocking(move || {
            let config = Arc::new(config);
            for body in [manifest.as_bytes(), b"untrusted update bytes".as_slice()] {
                let (socket, _) = runtime.block_on(tokio::time::timeout(Duration::from_secs(10), listener.accept())).unwrap().unwrap();
                let socket = socket.into_std().unwrap();
                socket.set_nonblocking(false).unwrap();
                socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                socket.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
                let connection = rustls::ServerConnection::new(Arc::clone(&config)).unwrap();
                let mut stream = rustls::StreamOwned::new(connection, socket);
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    stream.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                    assert!(request.len() < 16384);
                }
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                stream.write_all(body).unwrap();
                stream.flush().unwrap();
            }
        });
        let compiled: serde_json::Value = serde_json::from_str(include_str!("../../../tauri.conf.json")).unwrap();
        let key = compiled["plugins"]["updater"]["pubkey"].as_str().unwrap();
        let mut context = mock_context(noop_assets());
        context.config_mut().plugins.0.insert("updater".into(), compiled["plugins"]["updater"].clone());
        let app = mock_builder().plugin(tauri_plugin_updater::Builder::new().pubkey(key).build()).build(context).unwrap();
        let update = app
            .updater_builder()
            .endpoints(vec![format!("{origin}/manifest").parse().unwrap()])
            .unwrap()
            .configure_client(move |builder| {
                sources::client(builder.add_root_certificate(trust.clone())).timeout(Duration::from_secs(10)).no_proxy()
            })
            .build()
            .unwrap()
            .check()
            .await
            .unwrap()
            .unwrap();
        let mut downloaded = 0;
        let result = update.download(|bytes, _| downloaded += bytes, || {}).await;
        assert_eq!(downloaded, b"untrusted update bytes".len());
        assert!(matches!(result, Err(tauri_plugin_updater::Error::Minisign(_))));
        server.await.unwrap();
    }

    #[test]
    fn invalid_selection_preserves_saved_policy_and_switching_invalidates_review() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("selection.json");
        let mut policy = UpdatePolicy::default();
        policy.sources.insert("public".into(), Source::Github { repository: "example/product".into() });
        let accounts = Arc::new(auth::Accounts::open(root.path().join("accounts.json")));
        let state = UpdateState(Arc::new(Shared {
            path: path.clone(),
            accounts,
            inner: Mutex::new(Inner {
                policy,
                generation: 0,
                error: None,
                last: None,
                plugin: None,
                #[cfg(feature = "updater")]
                candidate: None,
            }),
        }));
        state.configure(Selection { mode: Mode::Manual, source: Some("public".into()) }).unwrap();
        let saved = std::fs::read(&path).unwrap();
        let generation = state.snapshot().unwrap().1;
        assert!(state.configure(Selection { mode: Mode::AutomaticCheck, source: Some("missing".into()) }).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        assert_eq!(state.snapshot().unwrap().1, generation);

        state.configure(Selection { mode: Mode::Disabled, source: None }).unwrap();
        assert_eq!(state.status().unwrap().mode, Mode::Disabled);
        assert_ne!(state.snapshot().unwrap().1, generation);
        assert!(state.status().unwrap().last_check.is_none());
    }
}
