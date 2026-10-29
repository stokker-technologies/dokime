/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Backend Registry Commands

use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

use super::{ConnectionError, ConnectionSession, ConnectionSnapshot};
use crate::native_session::{BackendSelection, DesktopSessionControl};
use dokime::client::profiles::{Document, Profile, Store};

#[derive(Default)]
pub struct Registry {
    serial: tokio::sync::Mutex<()>,
}

pub async fn load() -> Result<Document, ConnectionError> {
    tokio::task::spawn_blocking(|| Store::discover(None).and_then(|store| store.load()).map_err(persistence))
        .await
        .map_err(|_| ConnectionError::BackendUnavailable)?
}

pub async fn desktop_backends_list() -> Result<Document, ConnectionError> {
    load().await
}

pub async fn desktop_backend_add(registry: State<'_, Registry>, profile: Profile) -> Result<Document, ConnectionError> {
    let _serial = registry.serial.lock().await;
    let endpoint =
        url::Url::parse(&profile.endpoint).map_err(|_| ConnectionError::InvalidTarget { message: "Invalid lab endpoint".into() })?;
    if endpoint.scheme() != "http"
        || profile.ca_certificate_pem.is_some()
        || profile.expected_backend_id.is_some()
        || profile.secret != dokime::client::profiles::Secret::None
    {
        return Err(ConnectionError::InvalidTarget {
            message: "Secure backends require a pairing file".into(),
        });
    }
    tokio::task::spawn_blocking(move || {
        let store = Store::discover(None).map_err(persistence)?;
        store
            .update(|document| {
                if document.profiles.iter().any(|saved| saved.identifier == profile.identifier) {
                    return Err(dokime::client::profiles::Error::Invalid("Backend identifier already exists".into()));
                }
                document.profiles.push(profile);
                Ok(document.clone())
            })
            .map_err(persistence)
    })
    .await
    .map_err(|_| ConnectionError::BackendUnavailable)?
}

pub async fn desktop_backend_import_pairing(
    window: WebviewWindow, app: AppHandle, registry: State<'_, Registry>, connection: State<'_, ConnectionSession>,
    identifier: Option<String>,
) -> Result<Option<Document>, ConnectionError> {
    crate::app::desktop::ensure_main_window(&window).map_err(|message| ConnectionError::InvalidTarget { message })?;
    let selected =
        tokio::task::spawn_blocking(move || app.dialog().file().add_filter("Dokime Pairing File", &["dokime-pair"]).blocking_pick_file())
            .await
            .map_err(|_| ConnectionError::BackendUnavailable)?;
    let Some(selected) = selected else { return Ok(None) };
    let path = selected.into_path().map_err(|error| ConnectionError::Registry { message: error.to_string() })?;
    let _serial = registry.serial.lock().await;
    let _secrets = connection.secret_operations.lock().await;
    let secrets = connection.secrets.clone();
    tokio::task::spawn_blocking(move || {
        let pairing = dokime::client::pairing::read(&path).map_err(persistence)?;
        let identifier = identifier.as_deref().unwrap_or(&pairing.backend_identifier);
        let store = Store::discover(None).map_err(persistence)?;
        store.import_pairing(&pairing, identifier, secrets.as_ref()).map_err(persistence)?;
        store.load().map(Some).map_err(persistence)
    })
    .await
    .map_err(|_| ConnectionError::BackendUnavailable)?
}

pub async fn desktop_backend_remove(
    registry: State<'_, Registry>, owner: State<'_, DesktopSessionControl>, identifier: String,
) -> Result<Document, ConnectionError> {
    let _serial = registry.serial.lock().await;
    let removed_identifier = identifier.clone();
    let document = tokio::task::spawn_blocking(move || {
        let store = Store::discover(None).map_err(persistence)?;
        store
            .update(|document| {
                dokime::client::profiles::get(document, &identifier)?;
                document.profiles.retain(|profile| profile.identifier != identifier);
                if document.default.as_deref() == Some(identifier.as_str()) {
                    document.default = None;
                }
                Ok(document.clone())
            })
            .map_err(persistence)
    })
    .await
    .map_err(|_| ConnectionError::BackendUnavailable)??;
    owner.remove_profile(&removed_identifier).map_err(session_error)?;
    Ok(document)
}

pub async fn desktop_backend_use(
    registry: State<'_, Registry>, owner: State<'_, DesktopSessionControl>, identifier: String,
) -> Result<ConnectionSnapshot, ConnectionError> {
    let _serial = registry.serial.lock().await;
    Ok(owner.select(BackendSelection::Profile { identifier }).await.map_err(session_error)?.connection)
}

pub async fn remember(connection: &ConnectionSession, generation: &str, profile: Profile) -> Result<(), ConnectionError> {
    let connection = connection.clone();
    let generation = generation.to_owned();
    tokio::task::spawn_blocking(move || {
        let store = Store::discover(None).map_err(persistence)?;
        connection.with_current(&generation, || {
            store
                .update(|document| {
                    if dokime::client::profiles::get(document, &profile.identifier)? != &profile {
                        return Err(dokime::client::profiles::Error::Invalid("Backend changed during connection".into()));
                    }
                    document.default = Some(profile.identifier.clone());
                    Ok(())
                })
                .map_err(persistence)
        })??;
        connection.selected_profile(&generation, profile.identifier)
    })
    .await
    .map_err(|_| ConnectionError::BackendUnavailable)?
}

pub fn session_error(error: crate::native_session::DesktopSessionError) -> ConnectionError {
    match error {
        crate::native_session::DesktopSessionError::Connection { error } => error,
        _ => ConnectionError::BackendUnavailable,
    }
}

fn persistence(error: dokime::client::profiles::Error) -> ConnectionError {
    ConnectionError::Registry { message: error.to_string() }
}
