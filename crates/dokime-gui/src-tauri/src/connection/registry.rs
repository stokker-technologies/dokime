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
use tauri::State;

// Local Imports
use super::{ConnectionError, ConnectionSession, ConnectionSnapshot};
use crate::native_session::{BackendSelection, DesktopSessionControl};
use dokime::client::profiles::{Document, Profile, Store};

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

pub async fn desktop_backend_add(registry: State<'_, Registry>, profile: Profile) -> Result<Document, ConnectionError> {
    let _serial = registry.serial.lock().await;
    tokio::task::spawn_blocking(move || {
        let store = Store::discover(None).map_err(persistence)?;
        store
            .update(|document| {
                if document.profiles.iter().any(|saved| saved.name == profile.name) {
                    return Err(dokime::client::profiles::Error::Invalid("Backend name already exists".into()));
                }
                document.profiles.push(profile);
                Ok(document.clone())
            })
            .map_err(persistence)
    })
    .await
    .map_err(|_| ConnectionError::BackendUnavailable)?
}

pub async fn desktop_backend_remove(
    registry: State<'_, Registry>, owner: State<'_, DesktopSessionControl>, name: String,
) -> Result<Document, ConnectionError> {
    let _serial = registry.serial.lock().await;
    let removed_name = name.clone();
    let document = tokio::task::spawn_blocking(move || {
        let store = Store::discover(None).map_err(persistence)?;
        store
            .update(|document| {
                dokime::client::profiles::get(document, &name)?;
                document.profiles.retain(|profile| profile.name != name);
                if document.default.as_deref() == Some(name.as_str()) {
                    document.default = None;
                }
                Ok(document.clone())
            })
            .map_err(persistence)
    })
    .await
    .map_err(|_| ConnectionError::BackendUnavailable)??;
    owner.remove_profile(&removed_name).map_err(session_error)?;
    Ok(document)
}

pub async fn desktop_backend_use(
    registry: State<'_, Registry>, owner: State<'_, DesktopSessionControl>, name: String,
) -> Result<ConnectionSnapshot, ConnectionError> {
    let _serial = registry.serial.lock().await;
    Ok(owner.select(BackendSelection::Profile { name }).await.map_err(session_error)?.connection)
}

pub async fn remember(connection: &ConnectionSession, generation: &str, profile: Profile) -> Result<(), ConnectionError> {
    let connection = connection.clone();
    let generation = generation.to_owned();
    tokio::task::spawn_blocking(move || {
        let store = Store::discover(None).map_err(persistence)?;
        connection.with_current(&generation, || {
            store
                .update(|document| {
                    if dokime::client::profiles::get(document, &profile.name)? != &profile {
                        return Err(dokime::client::profiles::Error::Invalid("Backend changed during connection".into()));
                    }
                    document.default = Some(profile.name.clone());
                    Ok(())
                })
                .map_err(persistence)
        })??;
        connection.selected_profile(&generation, profile.name)
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

/* === Private Functions === */

fn persistence(error: dokime::client::profiles::Error) -> ConnectionError {
    ConnectionError::Registry { message: error.to_string() }
}

/* === Tests === */
