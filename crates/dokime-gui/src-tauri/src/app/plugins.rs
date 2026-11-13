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
use dokime_protocol::client;
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
        install(&lease, bytes, progress, None).await.map(|installed| Some(installed.install_id))
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

pub(super) async fn install(
    lease: &SessionLease, bytes: Vec<u8>, progress: Channel<InstallProgress>, expected_digest: Option<&str>,
) -> Result<client::PluginInstallation, String> {
    let size_bytes = bytes.len() as u64;
    let hashing = move || {
        let digest = Sha256::digest(&bytes);
        (bytes, digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>())
    };
    let (bytes, sha256) = tauri::async_runtime::spawn_blocking(hashing).await.map_err(text)?;
    if expected_digest.is_some_and(|expected| !expected.eq_ignore_ascii_case(&sha256)) {
        return Err("The downloaded plugin does not match the release SHA-256".into());
    }
    let header = client::InstallPluginFrame {
        frame: Some(client::InstallPluginFrameFrame {
            kind: Some(client::install_plugin_frame_frame::Kind::Header(client::InstallPluginHeader { size_bytes, sha256 })),
        }),
    };
    let (sender, receiver) = tokio::sync::mpsc::channel(2);
    sender.send(header).await.map_err(text)?;
    let uploading = async move {
        let mut offset = 0;
        for chunk in bytes.chunks(64 * 1024) {
            let frame = client::InstallPluginFrame {
                frame: Some(client::InstallPluginFrameFrame {
                    kind: Some(client::install_plugin_frame_frame::Kind::Chunk(client::InstallPluginChunk {
                        offset,
                        data: chunk.to_vec(),
                    })),
                }),
            };
            sender.send(frame).await.map_err(text)?;
            offset += chunk.len() as u64;
            let _ = progress.send(InstallProgress::Uploading { sent: offset, total: size_bytes });
        }
        let _ = progress.send(InstallProgress::Installing);
        Ok::<_, String>(())
    };
    let session = lease.client();
    let installing = async { session.install_plugin(tokio_stream::wrappers::ReceiverStream::new(receiver)).await.map_err(text) };
    let (_, installed) = lease.guard(async { tokio::try_join!(uploading, installing) }).await.map_err(|error| format!("{error:?}"))??;
    Ok(installed)
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
