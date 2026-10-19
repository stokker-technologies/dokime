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
// ***

//

// Dokime

//

// SPDX-License-Identifier: Apache-2.0

//

// ***
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Third Party Imports
use anyhow::{Context, Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use keyring::{Entry, Error as KeyringError};
use rand::random;

#[cfg(feature = "updater")]
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::{App, AppHandle, Manager, State};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_opener::OpenerExt;

#[cfg(feature = "updater")]
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::oneshot;
use tokio::time::timeout;
use url::Url;

// Local Imports

/* === Constants === */

const CALLBACK_SCHEME: &str = "dokime-test-hub";
const CALLBACK_HOST: &str = "auth";
const CALLBACK_PATH: &str = "/gitlab/callback";
const GITLAB_HOST: &str = "https://gitlab.com";

#[cfg(feature = "updater")]
const GITLAB_PROJECT_ID: &str = "83808544";
const KEYRING_SERVICE: &str = "dokime-test-hub";
const KEYRING_TOKEN_USER: &str = "gitlab-oauth-token";
const OAUTH_REDIRECT_URI: &str = "dokime-test-hub://auth/gitlab/callback";
const OAUTH_SCOPES: &str = "read_user read_api";
const SIGN_IN_TIMEOUT: Duration = Duration::from_secs(180);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct GitlabDesktopState {
    inner: Arc<GitlabDesktopStateInner>,
}

struct GitlabDesktopStateInner {
    config: GitlabConfig,
    pending_auth: Mutex<Option<PendingAuth>>,
}

#[derive(Clone)]
struct GitlabConfig {
    client_id: String,
    gitlab_host: String,

    #[cfg(feature = "updater")]
    project_id: String,

    redirect_uri: String,
}

struct PendingAuth {
    config: GitlabConfig,
    state: String,
    verifier: String,
    sender: oneshot::Sender<Result<GitlabUser>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitlabAuthStatus {
    pub configured: bool,
    pub signed_in: bool,
    pub user: Option<GitlabUser>,
    pub message: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitlabUser {
    // GitLab sends this identifier as a JSON number.
    pub id: u64,
    pub username: String,
    pub name: String,
    pub web_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitlabUpdateCheck {
    pub status: GitlabUpdateStatus,
    pub current_version: String,
    pub version: Option<String>,
    pub body: Option<String>,
    pub date: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitlabInstallResult {
    pub version: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[cfg(feature = "updater")]
#[derive(Deserialize)]
struct GitlabRelease {
    assets: GitlabReleaseAssets,
}

#[cfg(feature = "updater")]
#[derive(Deserialize)]
struct GitlabReleaseAssets {
    links: Vec<GitlabReleaseLink>,
}

#[cfg(feature = "updater")]
#[derive(Deserialize)]
struct GitlabReleaseLink {
    name: String,
    url: String,
    direct_asset_url: Option<String>,
}

/* === Enums === */

#[cfg_attr(not(feature = "updater"), allow(dead_code))]
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GitlabUpdateStatus {
    Available,
    NotConfigured,
    SignedOut,
    UpToDate,
}

/* === Traits === */

/* === Implementations === */

impl GitlabDesktopState {
    fn from_app(app: &App) -> Self {
        Self { inner: Arc::new(GitlabDesktopStateInner { config: GitlabConfig::from_app(app), pending_auth: Mutex::new(None) }) }
    }

    async fn auth_status(&self) -> Result<GitlabAuthStatus> {
        if !self.config().is_configured() {
            return Ok(GitlabAuthStatus {
                configured: false,
                signed_in: false,
                user: None,
                message: Some("GitLab OAuth client ID is not configured.".to_string()),
            });
        }

        // Reading the token means a keyring round trip; keep it off this
        // async task's Tokio worker thread (see `sign_out` above).
        let this = self.clone();
        let token = tauri::async_runtime::spawn_blocking(move || this.token()).await.context("GitLab token lookup task failed")??;
        let Some(token) = token else {
            return Ok(GitlabAuthStatus { configured: true, signed_in: false, user: None, message: None });
        };

        let user = self.fetch_user(&token).await.ok();
        Ok(GitlabAuthStatus { configured: true, signed_in: true, user, message: None })
    }

    async fn sign_in(&self, app: &AppHandle) -> Result<GitlabAuthStatus> {
        let config = self.config();
        if !config.is_configured() {
            return Err(anyhow!("GitLab OAuth client ID is not configured."));
        }

        let state = random_token();
        let verifier = random_token();
        let challenge = pkce_challenge(&verifier);
        let (sender, receiver) = oneshot::channel();

        {
            let mut pending = self.lock_pending_auth()?;
            if let Some(existing) = pending.take() {
                let _ = existing.sender.send(Err(anyhow!("A newer GitLab sign-in was started.")));
            }
            pending.replace(PendingAuth { config: config.clone(), state: state.clone(), verifier, sender });
        }

        let authorize_url = config.authorize_url(&state, &challenge)?;
        let app = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            app.opener().open_url(authorize_url, None::<&str>).context("failed to open GitLab sign-in page")
        })
        .await
        .context("GitLab sign-in page open task failed")??;

        let user =
            timeout(SIGN_IN_TIMEOUT, receiver).await.context("GitLab sign-in timed out")?.context("GitLab sign-in was canceled")??;

        Ok(GitlabAuthStatus { configured: true, signed_in: true, user: Some(user), message: None })
    }

    async fn sign_out(&self) -> Result<GitlabAuthStatus> {
        let configured = self.config().is_configured();
        tauri::async_runtime::spawn_blocking(move || match keyring_entry()?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(GitlabAuthStatus { configured, signed_in: false, user: None, message: None }),
            Err(error) => Err(error).context("failed to delete GitLab token"),
        })
        .await
        .context("GitLab sign-out task failed")?
    }

    #[cfg(feature = "updater")]
    async fn check_for_updates(&self, app: &AppHandle) -> Result<GitlabUpdateCheck> {
        let config = self.config();
        let current_version = app.package_info().version.to_string();
        if !config.is_configured() {
            return Ok(GitlabUpdateCheck {
                status: GitlabUpdateStatus::NotConfigured,
                current_version,
                version: None,
                body: None,
                date: None,
                message: Some("GitLab OAuth client ID is not configured.".to_string()),
            });
        }

        // Reading the token means a keyring round trip; keep it off this
        // async task's Tokio worker thread (see `sign_out` above).
        let this = self.clone();
        let token = tauri::async_runtime::spawn_blocking(move || this.token()).await.context("GitLab token lookup task failed")??;
        let Some(token) = token else {
            return Ok(GitlabUpdateCheck {
                status: GitlabUpdateStatus::SignedOut,
                current_version,
                version: None,
                body: None,
                date: None,
                message: Some("Sign in with GitLab before checking for updates.".to_string()),
            });
        };

        let updater = self.authenticated_updater(app, &token).await?;
        match updater.check().await.context("failed to check for Tauri updates")? {
            Some(update) => Ok(GitlabUpdateCheck {
                status: GitlabUpdateStatus::Available,
                current_version: update.current_version,
                version: Some(update.version),
                body: update.body,
                date: update.date.map(|date| date.to_string()),
                message: None,
            }),
            None => Ok(GitlabUpdateCheck {
                status: GitlabUpdateStatus::UpToDate,
                current_version,
                version: None,
                body: None,
                date: None,
                message: Some(format!("{} is up to date.", crate::PRODUCT)),
            }),
        }
    }

    #[cfg(feature = "updater")]
    async fn install_update(&self, app: &AppHandle) -> Result<GitlabInstallResult> {
        // Same keyring round trip as `check_for_updates`; keep it off the
        // Tokio worker thread.
        let this = self.clone();
        let token = tauri::async_runtime::spawn_blocking(move || this.token())
            .await
            .context("GitLab token lookup task failed")??
            .ok_or_else(|| anyhow!("Sign in with GitLab before installing updates."))?;
        let updater = self.authenticated_updater(app, &token).await?;
        let update =
            updater.check().await.context("failed to check for Tauri updates")?.ok_or_else(|| anyhow!("No update is available."))?;
        let version = update.version.clone();
        let bytes = update.download(|_, _| {}, || {}).await.context("failed to download signed update")?;
        update.install(bytes).context("failed to install signed update")?;
        Ok(GitlabInstallResult { version })
    }

    async fn complete_sign_in(&self, url: Url) {
        let Some((pending, code)) = self.take_matching_pending_auth(&url) else {
            return;
        };

        let result = match exchange_code(&pending.config, &code, &pending.verifier).await {
            Ok(token) => {
                // Storing the token means a keyring round trip; keep it off
                // this spawned task's Tokio worker thread (see `sign_out`
                // above).
                let store_token = token.clone();
                let store_result = match tauri::async_runtime::spawn_blocking(move || {
                    keyring_entry().and_then(|entry| entry.set_password(&store_token).context("failed to store GitLab token"))
                })
                .await
                {
                    Ok(result) => result,
                    Err(error) => Err(anyhow!("GitLab token storage task failed: {error}")),
                };
                match store_result {
                    Ok(()) => self.fetch_user(&token).await,
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        };
        let _ = pending.sender.send(result);
    }

    #[cfg(feature = "updater")]
    async fn authenticated_updater(&self, app: &AppHandle, token: &str) -> Result<tauri_plugin_updater::Updater> {
        let manifest_url = self.latest_manifest_url(token).await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).context("failed to build GitLab authorization header")?,
        );
        app.updater_builder().headers(headers).endpoints(vec![manifest_url])?.build().context("failed to build authenticated updater")
    }

    #[cfg(feature = "updater")]
    async fn latest_manifest_url(&self, token: &str) -> Result<Url> {
        let config = self.config();
        let release = reqwest::Client::new()
            .get(config.latest_release_api_url())
            .bearer_auth(token)
            .send()
            .await
            .context("failed to query latest GitLab release")?
            .error_for_status()
            .context("GitLab latest release request failed")?
            .json::<GitlabRelease>()
            .await
            .context("failed to decode latest GitLab release")?;

        release
            .assets
            .links
            .into_iter()
            .find(|link| link.name == "latest.json")
            .map(|link| link.direct_asset_url.unwrap_or(link.url))
            .ok_or_else(|| anyhow!("latest GitLab release does not include latest.json"))
            .and_then(|url| Url::parse(&url).context("latest.json URL is invalid"))
    }

    async fn fetch_user(&self, token: &str) -> Result<GitlabUser> {
        let config = self.config();
        reqwest::Client::new()
            .get(config.user_api_url())
            .bearer_auth(token)
            .send()
            .await
            .context("failed to query GitLab user")?
            .error_for_status()
            .context("GitLab user request failed")?
            .json::<GitlabUser>()
            .await
            .context("failed to decode GitLab user")
    }

    fn token(&self) -> Result<Option<String>> {
        match keyring_entry()?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(error).context("failed to read GitLab token"),
        }
    }

    fn config(&self) -> GitlabConfig {
        self.inner.config.clone()
    }

    fn lock_pending_auth(&self) -> Result<std::sync::MutexGuard<'_, Option<PendingAuth>>> {
        self.inner.pending_auth.lock().map_err(|_| anyhow!("GitLab auth state lock is poisoned"))
    }

    fn take_matching_pending_auth(&self, url: &Url) -> Option<(PendingAuth, String)> {
        if url.scheme() != CALLBACK_SCHEME || url.host_str() != Some(CALLBACK_HOST) || url.path() != CALLBACK_PATH {
            return None;
        }

        let mut code = None;
        let mut callback_state = None;
        let mut error = None;
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "code" => code = Some(value.to_string()),
                "state" => callback_state = Some(value.to_string()),
                "error" => error = Some(value.to_string()),
                _ => {}
            }
        }

        let mut pending = self.lock_pending_auth().ok()?;
        let pending_auth = pending.take()?;
        if let Some(error) = error {
            let _ = pending_auth.sender.send(Err(anyhow!("GitLab sign-in failed: {error}")));
            return None;
        }
        if callback_state.as_deref() != Some(pending_auth.state.as_str()) {
            let _ = pending_auth.sender.send(Err(anyhow!("GitLab sign-in state did not match.")));
            return None;
        }
        let Some(code) = code else {
            let _ = pending_auth.sender.send(Err(anyhow!("GitLab sign-in callback did not include a code.")));
            return None;
        };

        Some((pending_auth, code))
    }
}

impl GitlabConfig {
    fn from_app(app: &App) -> Self {
        let plugin = app.config().plugins.0.get("gitlab");
        Self {
            client_id: string_field(plugin, "oauthClientId").unwrap_or_default(),
            gitlab_host: string_field(plugin, "host").unwrap_or_else(|| GITLAB_HOST.to_string()),
            #[cfg(feature = "updater")]
            project_id: string_field(plugin, "projectId").unwrap_or_else(|| GITLAB_PROJECT_ID.to_string()),
            redirect_uri: string_field(plugin, "redirectUri").unwrap_or_else(|| OAUTH_REDIRECT_URI.to_string()),
        }
    }

    fn is_configured(&self) -> bool {
        !self.client_id.trim().is_empty()
    }

    fn authorize_url(&self, state: &str, challenge: &str) -> Result<String> {
        let mut url = Url::parse(&format!("{}/oauth/authorize", self.gitlab_host)).context("GitLab authorize URL is invalid")?;
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", OAUTH_SCOPES)
            .append_pair("state", state)
            .append_pair("code_challenge", challenge)
            .append_pair("code_challenge_method", "S256");
        Ok(url.to_string())
    }

    fn token_url(&self) -> String {
        format!("{}/oauth/token", self.gitlab_host)
    }

    fn user_api_url(&self) -> String {
        format!("{}/api/v4/user", self.gitlab_host)
    }

    #[cfg(feature = "updater")]
    fn latest_release_api_url(&self) -> String {
        format!("{}/api/v4/projects/{}/releases/permalink/latest", self.gitlab_host, self.project_id)
    }
}

/* === Public Functions === */

pub fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let state = GitlabDesktopState::from_app(app);
    app.manage(state.clone());

    let deep_link = app.deep_link();
    deep_link.register_all()?;

    let callback_state = state.clone();
    deep_link.on_open_url(move |record| {
        for url in record.urls() {
            let state = callback_state.clone();
            tauri::async_runtime::spawn(async move {
                state.complete_sign_in(url).await;
            });
        }
    });

    if let Some(urls) = deep_link.get_current()? {
        for url in urls {
            let state = state.clone();
            tauri::async_runtime::spawn(async move {
                state.complete_sign_in(url).await;
            });
        }
    }

    Ok(())
}

pub async fn gitlab_auth_status(state: State<'_, GitlabDesktopState>) -> Result<GitlabAuthStatus, String> {
    state.auth_status().await.map_err(render_error)
}

pub async fn gitlab_sign_in(app: AppHandle, state: State<'_, GitlabDesktopState>) -> Result<GitlabAuthStatus, String> {
    state.sign_in(&app).await.map_err(render_error)
}

pub async fn gitlab_sign_out(state: State<'_, GitlabDesktopState>) -> Result<GitlabAuthStatus, String> {
    state.sign_out().await.map_err(render_error)
}

pub async fn gitlab_check_for_updates(app: AppHandle, state: State<'_, GitlabDesktopState>) -> Result<GitlabUpdateCheck, String> {
    #[cfg(feature = "updater")]
    {
        state.check_for_updates(&app).await.map_err(render_error)
    }
    #[cfg(not(feature = "updater"))]
    {
        let _ = (app, state);
        distribution_managed_update()
    }
}

pub async fn gitlab_install_update(app: AppHandle, state: State<'_, GitlabDesktopState>) -> Result<GitlabInstallResult, String> {
    #[cfg(feature = "updater")]
    {
        state.install_update(&app).await.map_err(render_error)
    }
    #[cfg(not(feature = "updater"))]
    {
        let _ = (app, state);
        distribution_managed_update()
    }
}

/* === Private Functions === */

#[cfg(not(feature = "updater"))]
fn distribution_managed_update<T>() -> Result<T, String> {
    Err("Desktop updates are managed by the cargo-dokime distribution; reinstall that distribution to update this payload.".into())
}

async fn exchange_code(config: &GitlabConfig, code: &str, verifier: &str) -> Result<String> {
    // Encode The Token Exchange Form
    let form = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            ("client_id", config.client_id.as_str()),
            ("code", code),
            ("code_verifier", verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", config.redirect_uri.as_str()),
        ])
        .finish();
    // Exchange The Authorization Code
    let response = reqwest::Client::new()
        .post(config.token_url())
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form)
        .send()
        .await
        .context("failed to exchange GitLab authorization code")?
        .error_for_status()
        .context("GitLab token exchange failed")?
        .json::<TokenResponse>()
        .await
        .context("failed to decode GitLab token response")?;
    Ok(response.access_token)
}

fn keyring_entry() -> Result<Entry> {
    Entry::new(KEYRING_SERVICE, KEYRING_TOKEN_USER).context("failed to access OS keyring")
}

fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn random_token() -> String {
    URL_SAFE_NO_PAD.encode(random::<[u8; 32]>())
}

fn render_error(error: anyhow::Error) -> String {
    error.to_string()
}

fn string_field(plugin: Option<&Value>, field: &str) -> Option<String> {
    plugin?.get(field)?.as_str().map(str::trim).filter(|value| !value.is_empty()).map(ToOwned::to_owned)
}

/* === Tests === */
