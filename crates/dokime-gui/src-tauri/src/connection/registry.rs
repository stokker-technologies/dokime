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
use super::host::Pairing;
use super::{ConnectionError, ConnectionSnapshot};
use crate::native_session::{BackendSelection, DesktopSessionControl};
use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Path, PathBuf},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct Registry {
    serial: tokio::sync::Mutex<()>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub profiles: Vec<Profile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub identifier: String,
    pub label: String,
    pub endpoint: String,
    pub endpoints: Vec<String>,
    pub expected_backend_id: Option<String>,
    pub certificate_pem: Option<String>,
    pub pairing_file: String,
}

struct Store {
    path: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    fn discover(override_path: Option<&Path>) -> io::Result<Self> {
        if let Some(path) = override_path {
            return Ok(Self { path: path.to_path_buf() });
        }
        let environment = |key: &str| std::env::var_os(key).filter(|value| !value.is_empty()).map(PathBuf::from);
        let base = if cfg!(target_os = "windows") {
            environment("APPDATA")
        } else if cfg!(target_os = "macos") {
            environment("HOME").map(|home| home.join("Library/Application Support"))
        } else {
            environment("XDG_CONFIG_HOME").or_else(|| environment("HOME").map(|home| home.join(".config")))
        }
        .ok_or_else(|| io::Error::other("Read Saved Backends: Configure an Application Data Directory"))?;
        Ok(Self { path: base.join("dokime/remotes") })
    }

    fn load(&self) -> io::Result<Document> {
        let entries = match std::fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Document::default()),
            Err(error) => return Err(error),
        };
        let mut profiles = Vec::new();
        for entry in entries {
            let path = entry?.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("pair") {
                continue;
            }
            let pairing = Pairing::read(&path).map_err(io::Error::other)?;
            let identifier = path
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| io::Error::other("Read Saved Backends: Pairing Filename Must Be UTF-8"))?;
            profiles.push(profile(&pairing, identifier, &path)?);
        }
        profiles.sort_by(|left, right| left.identifier.cmp(&right.identifier));
        Ok(Document { profiles })
    }

    fn import_pairing(&self, pairing: &Pairing, identifier: &str) -> io::Result<Profile> {
        validate_identifier(identifier)?;
        pairing.validate().map_err(io::Error::other)?;
        crate::app::desktop::ensure_private_directory(&self.path).map_err(io::Error::other)?;
        let destination = self.path.join(format!("{identifier}.pair"));
        let bytes = zeroize::Zeroizing::new(serde_json::to_vec_pretty(pairing).map_err(io::Error::other)?);
        let mut staged = tempfile::NamedTempFile::new_in(&self.path)?;
        crate::app::desktop::set_private_file(staged.path()).map_err(io::Error::other)?;
        use std::io::Write;
        staged.write_all(&bytes)?;
        staged.as_file().sync_all()?;
        staged.persist_noclobber(&destination).map_err(io::Error::other)?;
        profile(pairing, identifier, &destination)
    }

    fn remove(&self, identifier: &str) -> io::Result<()> {
        validate_identifier(identifier)?;
        std::fs::remove_file(self.path.join(format!("{identifier}.pair")))
    }
}

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
            let pairing = Pairing::read(&path).map_err(persistence)?;
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

fn persistence(error: impl std::error::Error) -> ConnectionError {
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

pub(crate) fn get<'a>(document: &'a Document, identifier: &str) -> io::Result<&'a Profile> {
    validate_identifier(identifier)?;
    document
        .profiles
        .iter()
        .find(|profile| profile.identifier == identifier)
        .ok_or_else(|| io::Error::other("Connect Saved Backend: Import Its Pairing File Again"))
}

fn profile(pairing: &Pairing, identifier: &str, path: &Path) -> io::Result<Profile> {
    validate_identifier(identifier)?;
    Ok(Profile {
        identifier: identifier.to_owned(),
        label: pairing.label.clone(),
        endpoint: pairing.endpoints[0].clone(),
        endpoints: pairing.endpoints.clone(),
        expected_backend_id: Some(pairing.backend_identifier.clone()),
        certificate_pem: Some(pairing.certificate_pem.clone()),
        pairing_file: std::path::absolute(path)?
            .to_str()
            .ok_or_else(|| io::Error::other("Read Saved Backend: Pairing Path Must Be UTF-8"))?
            .to_owned(),
    })
}

fn validate_identifier(identifier: &str) -> io::Result<()> {
    if identifier.is_empty()
        || identifier.len() > 256
        || matches!(identifier, "." | "..")
        || !identifier.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(io::Error::other("Save Backend: Use ASCII Letters, Digits, Dots, Underscores or Hyphens for the Identifier"));
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_inventory_preserves_files_and_excludes_credentials() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let store = Store::discover(Some(root.path()))?;
        let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
        let pairing = Pairing {
            version: 1,
            backend_identifier: "host-identity".into(),
            label: "Remote Fixture".into(),
            endpoints: vec!["https://localhost:7443".into()],
            certificate_pem: certificate.cert.pem(),
            access_key: "fixture-private-access-key".into(),
        };
        let installed = store.import_pairing(&pairing, "lab")?;
        let original = std::fs::read(&installed.pairing_file)?;
        let inventory = store.load()?;
        assert_eq!(inventory.profiles.len(), 1);
        assert_eq!(get(&inventory, "lab")?.expected_backend_id.as_deref(), Some("host-identity"));
        assert!(!serde_json::to_string(&inventory)?.contains(&pairing.access_key));
        assert!(store.import_pairing(&pairing, "lab").is_err());
        assert_eq!(std::fs::read(&installed.pairing_file)?, original);
        assert!(store.import_pairing(&pairing, "../escape").is_err());
        assert!(store.remove("../escape").is_err());
        store.remove("lab")?;
        assert!(store.load()?.profiles.is_empty());
        Ok(())
    }
}
