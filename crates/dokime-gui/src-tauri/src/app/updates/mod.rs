/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod plugins;
pub mod policy;
mod sources;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};

#[cfg(feature = "updater")]
use std::time::Duration;

// Third Party Imports
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[cfg(feature = "updater")]
use tauri_plugin_updater::UpdaterExt;

// Local Imports
use self::policy::{Mode, UpdatePolicy};
use super::organization::OrganizationState;
use crate::settings::{Store, UpdateSelection};

/* === Constants === */

#[cfg(feature = "updater")]
const DESKTOP_UPDATE_TIMEOUT_MS: u64 = 300_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct UpdateState(Arc<Shared>);

struct Shared {
    store: Arc<Store>,
    inner: Mutex<Inner>,
}

struct Inner {
    policy: UpdatePolicy,
    generation: u64,
    error: Option<String>,
    last: Option<UpdateCheck>,

    #[cfg(feature = "updater")]
    candidate: Option<Candidate>,
}

#[cfg(feature = "updater")]
struct Candidate {
    id: String,
    generation: u64,
    update: tauri_plugin_updater::Update,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub mode: Mode,
    pub source: Option<String>,
    pub sources: Vec<String>,
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
        Ok(UpdateStatus {
            mode: inner.policy.mode,
            source: inner.policy.source.clone(),
            sources: inner.policy.sources.keys().cloned().collect(),
            last_check: inner.last.clone(),
            error: inner.error.clone(),
        })
    }

    fn snapshot(&self) -> Result<(UpdatePolicy, u64), String> {
        let inner = self.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        Ok((inner.policy.clone(), inner.generation))
    }

    fn configure(&self, selection: UpdateSelection) -> Result<(), String> {
        let mut inner = self.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        let mut policy = inner.policy.clone();
        policy.mode = selection.mode;
        policy.source = selection.source.clone();
        policy.validate()?;
        self.0
            .store
            .update(|settings| {
                settings.updates = Some(selection);
                Ok(())
            })
            .map_err(|error| format!("Update selection could not be saved: {error}"))?;
        inner.policy = policy;
        inner.generation = inner.generation.wrapping_add(1);
        inner.last = None;
        inner.error = None;
        #[cfg(feature = "updater")]
        {
            inner.candidate = None;
        }
        Ok(())
    }

    #[cfg(feature = "updater")]
    async fn check(&self, app: &AppHandle) -> Result<UpdateCheck, String> {
        let (policy, generation) = self.snapshot()?;
        let current_version = app.package_info().version.to_string();
        if policy.mode == Mode::Disabled {
            return Ok(UpdateCheck {
                status: "disabled".into(),
                current_version,
                version: None,
                body: None,
                candidate: None,
            });
        }
        let manifest = sources::resolve(&policy, "release.json").await?;
        let updater = app
            .updater_builder()
            .endpoints(vec![manifest])
            .map_err(|_| "The update manifest URL is invalid")?
            .configure_client(sources::client)
            // The Installer Starts Right After This Hook and Must Outlive the Exiting Desktop.
            .on_before_exit(|| {
                if let Err(error) = dokime_runtime::process::detach_later_children() {
                    tracing::error!(%error, "Detach the Update Installer");
                }
            })
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
                policy::https_url(update.download_url.as_str())?;
                update.timeout = Some(Duration::from_millis(DESKTOP_UPDATE_TIMEOUT_MS));
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
            None => UpdateCheck {
                status: "up_to_date".into(),
                current_version,
                version: None,
                body: None,
                candidate: None,
            },
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
    let store = app.state::<Arc<Store>>().inner().clone();
    let mut policy = app.state::<OrganizationState>().update_policy()?;
    let mut error = None;
    match crate::settings::load(store.path()) {
        Ok(settings) => {
            if let Some(selection) = settings.updates {
                let mut selected = policy.clone();
                selected.mode = selection.mode;
                selected.source = selection.source;
                match selected.validate() {
                    Ok(()) => policy = selected,
                    Err(reason) => {
                        policy.mode = Mode::Disabled;
                        error = Some(reason);
                    }
                }
            }
        }
        Err(_) => {
            policy.mode = Mode::Disabled;
            error = Some("Saved desktop settings could not be read; updates are disabled".into());
        }
    }
    let state = UpdateState(Arc::new(Shared {
        store,
        inner: Mutex::new(Inner {
            policy,
            generation: 0,
            error,
            last: None,
            #[cfg(feature = "updater")]
            candidate: None,
        }),
    }));
    app.manage(state.clone());
    state.automatic(app.handle());
    Ok(())
}

pub async fn desktop_updates_status(state: State<'_, UpdateState>) -> Result<UpdateStatus, String> {
    state.status()
}

pub async fn desktop_updates_configure(
    app: AppHandle, state: State<'_, UpdateState>, selection: UpdateSelection,
) -> Result<UpdateStatus, String> {
    state.configure(selection)?;
    state.automatic(&app);
    state.status()
}

pub async fn desktop_updates_check(app: AppHandle, state: State<'_, UpdateState>) -> Result<UpdateCheck, String> {
    state.check(&app).await
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
        // A Failed Launch Returns Here With the Desktop Still Running; Own Its Children Again.
        update.install(bytes).map_err(|_| {
            if let Err(error) = dokime_runtime::process::own_later_children() {
                tracing::error!(%error, "Own Child Processes After a Failed Update Launch");
            }
            "The verified update could not be installed"
        })?;
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

#[cfg(feature = "testing")]
pub struct UpdateProbe(UpdateState);

#[cfg(feature = "testing")]
impl UpdateProbe {
    pub fn new(path: std::path::PathBuf, policy: UpdatePolicy) -> Self {
        Self(UpdateState(Arc::new(Shared {
            store: Arc::new(Store::new(path)),
            inner: Mutex::new(Inner {
                policy,
                generation: 0,
                error: None,
                last: None,
                #[cfg(feature = "updater")]
                candidate: None,
            }),
        })))
    }
    pub fn configure(&self, selection: UpdateSelection) -> Result<(), String> {
        self.0.configure(selection)
    }
    pub fn generation(&self) -> Result<u64, String> {
        self.0.snapshot().map(|snapshot| snapshot.1)
    }
    pub fn status(&self) -> Result<UpdateStatus, String> {
        self.0.status()
    }
}

#[cfg(feature = "testing")]
pub fn update_client(builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    sources::client(builder)
}

/* === Tests === */
