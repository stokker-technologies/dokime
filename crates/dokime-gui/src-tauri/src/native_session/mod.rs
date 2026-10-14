/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod backend;
mod contract;
mod diagnostics;
pub mod launch;
mod owner;
pub mod storage;

/* === Re-Exports === */
pub use contract::*;
pub use owner::DesktopSession;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct DesktopSessionControl(DesktopSession);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl DesktopSessionControl {
    pub fn active(owner: DesktopSession) -> Self {
        Self(owner)
    }
    pub async fn shutdown(&self) -> Result<(), DesktopSessionError> {
        self.0.shutdown().await?;
        Ok(())
    }
    pub fn status(&self) -> DesktopSessionSnapshot {
        self.0.status()
    }
}

/* === Public Functions === */
pub async fn desktop_session_select(
    state: tauri::State<'_, DesktopSessionControl>,
    selection: BackendSelection,
) -> Result<DesktopSessionSnapshot, DesktopSessionError> {
    state.0.select(selection).await
}

pub async fn desktop_session_disconnect(
    state: tauri::State<'_, DesktopSessionControl>,
) -> Result<DesktopSessionSnapshot, DesktopSessionError> {
    state.0.disconnect().await
}

pub fn desktop_session_status(
    state: tauri::State<'_, DesktopSessionControl>,
) -> Result<DesktopSessionSnapshot, DesktopSessionError> {
    Ok(state.0.status())
}

/// Development only: stops the owned backend, deletes the contents of its data
/// directory and starts it again on the empty root. `directory` is the path
/// the operator confirmed; anything but this desktop's data root is refused.
pub async fn desktop_reset_local_storage(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, DesktopSessionControl>,
    directory: String,
) -> Result<DesktopSessionSnapshot, DesktopSessionError> {
    use tauri::Manager;

    // Development Builds And The Main Window Only
    let unavailable = |message: &str| DesktopSessionError::StorageReset {
        reason: StorageResetReason::DevelopmentOnly,
        message: message.to_owned(),
    };
    if !tauri::is_dev() {
        return Err(unavailable(
            "Local storage can be reset only from a development build.",
        ));
    }
    let url = window
        .url()
        .map_err(|error| unavailable(&error.to_string()))?;
    if window.label() != "main" || !trusted_application_url(&url, window.app_handle().config()) {
        return Err(unavailable(
            "Only the main application window may reset local storage.",
        ));
    }

    // Resolve The Data Root The Owned Backend Opens
    let failed = |message: String| DesktopSessionError::StorageReset {
        reason: StorageResetReason::Failed,
        message,
    };
    let application =
        crate::app::settings::application_root(&window.app_handle().config().identifier)
            .map_err(|error| failed(error.to_string()))?;
    let root = storage::StorageRoot::resolve(application).map_err(failed)?;
    tracing::warn!(directory, "local storage reset requested");
    state
        .0
        .reset_local_storage(std::path::Path::new(&directory), root)
        .await
}

pub(crate) fn trusted_application_url(url: &url::Url, config: &tauri::Config) -> bool {
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    // Only a shell that serves its frontend from the development server trusts
    // that origin; a debug build with embedded assets loads them like a release.
    if tauri::is_dev()
        && let Some(dev_url) = &config.build.dev_url
    {
        return url.origin() == dev_url.origin();
    }
    matches!(
        (url.scheme(), url.host_str()),
        ("tauri", Some("localhost")) | ("http" | "https", Some("tauri.localhost"))
    ) && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
}

/// Opens the failed owned start's log with the system handler. The WebView
/// names no path: the shell opens only the log its own session recorded.
pub async fn desktop_open_backend_log(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, DesktopSessionControl>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri::Manager;
    use tauri_plugin_opener::OpenerExt;

    // Trust Only The Main Application Window
    let url = window.url().map_err(|error| error.to_string())?;
    if window.label() != "main" || !trusted_application_url(&url, window.app_handle().config()) {
        return Err("Only the main application window may open the backend log.".into());
    }

    // Open The Recorded Log
    let log = state
        .0
        .startup_log()
        .ok_or("No failed backend start has a log file to open.")?;
    // The opener forks a launcher process; keep it off the async workers.
    tauri::async_runtime::spawn_blocking(move || {
        app.opener()
            .open_path(log.to_string_lossy(), None::<&str>)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

pub fn desktop_profiles_list() -> Result<dokime_client::profiles::Document, String> {
    let store =
        dokime_client::profiles::Store::discover(None).map_err(|error| error.to_string())?;
    store.load().map_err(|error| error.to_string())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    #[test]
    fn navigation_requires_application_origin() {
        let config = tauri::Config::default();
        assert!(super::trusted_application_url(
            &url::Url::parse("tauri://localhost/index.html").unwrap(),
            &config
        ));
        assert!(!super::trusted_application_url(
            &url::Url::parse("https://example.com/").unwrap(),
            &config
        ));
        assert!(!super::trusted_application_url(
            &url::Url::parse("https://tauri.localhost.evil.example/").unwrap(),
            &config
        ));
        assert!(!super::trusted_application_url(
            &url::Url::parse("https://secret@tauri.localhost/").unwrap(),
            &config
        ));
        assert!(!super::trusted_application_url(
            &url::Url::parse("https://tauri.localhost:8443/").unwrap(),
            &config
        ));
    }
}
