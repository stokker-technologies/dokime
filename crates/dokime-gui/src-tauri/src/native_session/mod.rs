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
pub mod launch;
mod owner;

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

pub fn desktop_session_transport<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    state: tauri::State<'_, crate::connection::ConnectionSession>,
    generation: String,
) -> Result<crate::connection::BrowserAccess, crate::connection::ConnectionError> {
    use tauri::Manager;
    let url = window
        .url()
        .map_err(|_| crate::connection::ConnectionError::PermissionDenied)?;
    if window.label() != "main" || !trusted_application_url(&url, window.app_handle().config()) {
        return Err(crate::connection::ConnectionError::PermissionDenied);
    }
    state.browser_access(&generation)
}

pub(crate) fn trusted_application_url(url: &url::Url, config: &tauri::Config) -> bool {
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    if cfg!(debug_assertions)
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
    fn credentials_require_application_origin() {
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
