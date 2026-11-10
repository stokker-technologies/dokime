/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Backend Registry Commands

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

// Local Imports
use super::{ConnectionError, ConnectionSnapshot};
use crate::native_session::{BackendSelection, DesktopSessionControl};
use dokime::client::profiles::{Document, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct Registry {
    serial: tokio::sync::Mutex<()>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn load() -> Result<Document, ConnectionError> {
    tokio::task::spawn_blocking(|| Store::discover(None).and_then(|store| store.load()).map_err(persistence))
        .await
        .map_err(|_| ConnectionError::BackendUnavailable)?
}

pub async fn desktop_backends_list() -> Result<Document, ConnectionError> {
    load().await
}

pub async fn desktop_backend_import_pairing(
    window: WebviewWindow, app: AppHandle, registry: State<'_, Registry>, identifier: Option<String>,
) -> Result<Option<Document>, ConnectionError> {
    crate::app::desktop::ensure_main_window(&window).map_err(|message| ConnectionError::InvalidTarget { message })?;
    let picker = app.clone();
    let selected = tokio::task::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .add_filter(format!("{} Remote Access File", picker.state::<crate::app::organization::OrganizationState>().label()), &["pair"])
            .blocking_pick_file()
    })
    .await
    .map_err(|_| ConnectionError::BackendUnavailable)?;
    let Some(selected) = selected else { return Ok(None) };
    let path = selected.into_path().map_err(|error| ConnectionError::Registry { message: error.to_string(), cause: None })?;
    import_path(&app, &registry, path, identifier)
        .await
        .inspect_err(|error| {
            let _ = app.emit("desktop://remote-import-failed", error);
        })
        .map(Some)
}

pub async fn import_path(
    app: &AppHandle, registry: &Registry, path: std::path::PathBuf, identifier: Option<String>,
) -> Result<Document, ConnectionError> {
    let identifier = {
        let _serial = registry.serial.lock().await;
        tokio::task::spawn_blocking(move || {
            let pairing = dokime::client::pairing::read(&path).map_err(persistence)?;
            let identifier = identifier.as_deref().unwrap_or(&pairing.backend_identifier);
            let store = Store::discover(None).map_err(persistence)?;
            store.import_pairing(&pairing, identifier).map_err(persistence)?;
            Ok::<_, ConnectionError>(identifier.to_owned())
        })
        .await
        .map_err(|_| ConnectionError::BackendUnavailable)??
    };
    app.state::<DesktopSessionControl>().select(BackendSelection::Profile { identifier }).await.map_err(session_error)?;
    let _ = app.emit("desktop://remote-imported", ());
    load().await
}

pub async fn desktop_backend_remove(
    registry: State<'_, Registry>, owner: State<'_, DesktopSessionControl>, identifier: String,
) -> Result<Document, ConnectionError> {
    let _serial = registry.serial.lock().await;
    let removed_identifier = identifier.clone();
    let document = tokio::task::spawn_blocking(move || {
        let store = Store::discover(None).map_err(persistence)?;
        store.remove(&identifier).map_err(persistence)?;
        store.load().map_err(persistence)
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

pub fn session_error(error: crate::native_session::DesktopSessionError) -> ConnectionError {
    match error {
        crate::native_session::DesktopSessionError::Connection { error } => error,
        _ => ConnectionError::BackendUnavailable,
    }
}

/* === Private Functions === */

fn persistence(error: dokime::client::profiles::Error) -> ConnectionError {
    use std::error::Error;
    let mut messages = Vec::new();
    let mut source = error.source();
    while let Some(error) = source {
        messages.push(error.to_string());
        source = error.source();
    }
    let mut cause = None;
    for message in messages.into_iter().rev() {
        cause = Some(Box::new(super::contract::RegistryCause { message, cause }));
    }
    ConnectionError::Registry { message: error.to_string(), cause }
}

/* === Tests === */
