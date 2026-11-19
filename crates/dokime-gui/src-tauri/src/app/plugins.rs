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
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// Third Party Imports
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State, WebviewWindow};

// Local Imports
use super::desktop::{ensure_generation_token, ensure_main_window};
use crate::connection::{ConnectionSession, SessionLease};
use dokime_client_internal::{
    FilesUploadsAppendOptions, FilesUploadsCreateOptions, FilesUploadsDeleteOptions, FilesUploadsPublishOptions, PluginEntry,
};
use dokime_sdk::{FileMetadata, Upload};
use sha2::{Digest, Sha256};
const INSTALL_ARCHIVE_MAX_BYTES: u64 = 512 * 1024 * 1024;

/* === Constants === */

// Each progress message re-renders the webview; bound how often one is sent.
// A native drop is installed right after the webview sees it; an older one is stale.
const DROP_VALIDITY: Duration = Duration::from_secs(30);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The latest native file drop per window, recorded by the shell so a installation
// request names a dropped bundle without the webview passing a path.
#[derive(Default)]
pub struct DroppedFiles(Mutex<Option<FileDrop>>);

struct FileDrop {
    window: String,
    paths: Vec<PathBuf>,
    at: Instant,
}

/* === Enums === */

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum InstallProgress {
    Reading,
    Uploading { sent: u64, total: u64 },
    Installing,
}

/* === Traits === */

/* === Implementations === */

impl DroppedFiles {
    pub fn record(&self, window: &str, paths: &[PathBuf]) {
        let drop = FileDrop {
            window: window.to_owned(),
            paths: paths.to_vec(),
            at: Instant::now(),
        };
        *self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(drop);
    }

    fn take(&self, window: &str) -> Result<PathBuf, String> {
        let drop = self.0.lock().map_err(|_| "The dropped plugin file is unavailable")?.take();
        let Some(drop) = drop.filter(|drop| drop.window == window && drop.at.elapsed() < DROP_VALIDITY) else {
            return Err("No plugin bundle was dropped on this window".into());
        };
        match drop.paths.as_slice() {
            [path] if path.extension().is_some_and(|extension| extension == "dokime") => Ok(path.clone()),
            _ => Err("Drop exactly one .dokime plugin bundle".into()),
        }
    }
}

/* === Public Functions === */

// Install the bundle just dropped on the window, or pick one natively; `None` when the operator cancels the picker.
pub async fn desktop_plugin_install_file<R: tauri::Runtime>(
    window: WebviewWindow<R>, state: State<'_, ConnectionSession>, dropped: State<'_, DroppedFiles>, app: AppHandle<R>, generation: String,
    drop: bool, progress: Channel<InstallProgress>,
) -> Result<Option<String>, String> {
    // Keep The Native Command Small Even When Called Outside The Bindings Adapter.
    Box::pin(async move {
        let lease = admit(&window, &state, &generation)?;
        let path = if drop {
            dropped.take(window.label())?
        } else {
            let Some(path) = lease.guard(pick_bundle(&app)).await.map_err(|error| format!("{error:?}"))?? else {
                return Ok(None);
            };
            path
        };

        let _ = progress.send(InstallProgress::Reading);
        let reading = move || harmos_runtime::filesystem::input::read(&path, INSTALL_ARCHIVE_MAX_BYTES as usize);
        let read = tauri::async_runtime::spawn_blocking(reading).await.map_err(text)?;
        let bytes = read.map_err(|error| format!("The plugin bundle could not be read: {error}"))?;
        install(&lease, bytes, progress, None).await.map(|installed| Some(installed.name))
    })
    .await
}

/* === Private Functions === */

pub(super) fn admit<R: tauri::Runtime>(
    window: &WebviewWindow<R>, state: &ConnectionSession, generation: &str,
) -> Result<SessionLease, String> {
    ensure_main_window(window)?;
    ensure_generation_token(generation)?;
    state.lease(generation).map_err(|error| format!("Native plugin installation lost its owning connection: {error:?}"))
}

async fn pick_bundle<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<Option<PathBuf>, String> {
    let dialog = app.try_state::<tauri_plugin_dialog::Dialog<R>>().ok_or("The plugin file picker is unavailable")?;
    let (selected, selection) = tokio::sync::oneshot::channel();
    dialog.file().add_filter("Plugin Bundle", &["dokime"]).pick_file(move |path| {
        // The Owning Command May Have Been Cancelled While The Picker Was Open.
        let _ = selected.send(path);
    });
    let Some(path) = selection.await.map_err(|_| "The plugin file picker closed without a result")? else {
        return Ok(None);
    };
    path.into_path().map(Some).map_err(text)
}

pub async fn install(
    lease: &SessionLease, bytes: Vec<u8>, progress: Channel<InstallProgress>, expected_digest: Option<&str>,
) -> Result<PluginEntry, String> {
    let size_bytes = bytes.len() as u64;
    let hashing = move || (Sha256::digest(&bytes).to_vec(), bytes);
    let (digest, bytes) = tauri::async_runtime::spawn_blocking(hashing).await.map_err(text)?;
    let sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    if expected_digest.is_some_and(|expected| !expected.eq_ignore_ascii_case(&sha256)) {
        return Err("Install Plugin: Downloaded Bundle Does Not Match Its Release SHA-256".into());
    }
    let session = lease.client();
    let client = session.client();
    let name = format!("plugin_{:032x}", rand::random::<u128>());
    let metadata = FileMetadata {
        id: name.clone(),
        media_type: "application/vnd.dokime.plugin".into(),
        expected_length: Some(size_bytes),
        expected_sha256: Some(digest.clone()),
        ..Default::default()
    };
    let options = FilesUploadsCreateOptions {
        request_id: Some(format!("{name}/upload")),
        context: session.call_options().map_err(text)?,
    };
    let created = session.execute(client.files.uploads.create_with(metadata, options)).await.map_err(text)?;
    let mut upload: Upload = created.upload.ok_or("Install Plugin: Host Omitted Its Upload")?;
    let mut published = false;
    let result = async {
        let chunk_bytes = usize::try_from(created.chunk_bytes).map_err(text)?;
        if chunk_bytes == 0 || created.chunk_bytes >= created.message_bytes {
            return Err("Install Plugin: Host Returned Invalid Transfer Limits".into());
        }
        let mut offset = 0;
        for chunk in bytes.chunks(chunk_bytes) {
            let options = FilesUploadsAppendOptions {
                request_id: Some(format!("{name}/append/{offset}")),
                context: session.call_options().map_err(text)?,
            };
            let reply =
                session.execute(client.files.uploads.append_with(upload.clone(), offset, chunk.to_vec(), options)).await.map_err(text)?;
            upload = reply.upload.ok_or("Install Plugin: Host Omitted Upload Progress")?;
            offset += chunk.len() as u64;
            if upload.accepted_length != offset {
                return Err("Install Plugin: Host Accepted a Different Byte Offset".into());
            }
            let _ = progress.send(InstallProgress::Uploading { sent: offset, total: size_bytes });
        }
        let options = FilesUploadsPublishOptions {
            request_id: Some(format!("{name}/publish")),
            context: session.call_options().map_err(text)?,
        };
        let reply =
            session.execute(client.files.uploads.publish_with(upload.clone(), size_bytes, digest.clone(), options)).await.map_err(text)?;
        published = true;
        let file = reply.upload.ok_or("Install Plugin: Host Omitted Its Published File")?;
        if file.length != size_bytes || file.sha256 != digest {
            return Err("Install Plugin: Published File Length or SHA-256 Changed".into());
        }
        let _ = progress.send(InstallProgress::Installing);
        session.install_plugin(&name, file).await.map_err(text)
    }
    .await;
    if result.is_err() && !published {
        let options = FilesUploadsDeleteOptions {
            request_id: Some(format!("{name}/abandon")),
            context: session.call_options().map_err(text)?,
        };
        session
            .execute(client.files.uploads.delete_with(upload, options))
            .await
            .map_err(|error| format!("Abandon Plugin Upload After {result:?}: {error}"))?;
    }
    result
}

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_drop_ownership() {
        let dropped = DroppedFiles::default();
        assert!(dropped.take("main").is_err(), "nothing was dropped");

        dropped.record("main", &[PathBuf::from("/bundles/plugin-obc.dokime")]);
        assert!(dropped.take("other").is_err(), "a drop belongs to its window");
        dropped.record("main", &[PathBuf::from("/bundles/plugin-obc.dokime")]);
        assert_eq!(dropped.take("main").unwrap(), PathBuf::from("/bundles/plugin-obc.dokime"));
        assert!(dropped.take("main").is_err(), "a drop is consumed by one installation");

        dropped.record("main", &[PathBuf::from("/bundles/a.dokime"), PathBuf::from("/bundles/b.dokime")]);
        assert!(dropped.take("main").is_err(), "several files are refused");
        dropped.record("main", &[PathBuf::from("/bundles/readme.txt")]);
        assert!(dropped.take("main").is_err(), "a non-bundle is refused");
    }
}
