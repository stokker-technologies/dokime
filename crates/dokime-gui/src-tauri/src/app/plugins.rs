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
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// Third Party Imports
use tauri::ipc::Channel;
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

// Local Imports
use super::desktop::{ensure_generation_token, ensure_main_window};
use crate::connection::{ConnectionSession, SessionLease};
use dokime::domain::model::plugins::wire::STAGE_ARCHIVE_MAX_BYTES;

/* === Constants === */

const STAGE_DEADLINE: Duration = Duration::from_secs(300);
// Each progress message re-renders the webview; bound how often one is sent.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
// A native drop is staged right after the webview sees it; an older one is stale.
const DROP_VALIDITY: Duration = Duration::from_secs(30);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The latest native file drop per window, recorded by the shell so a staging
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
pub enum StageProgress {
    Reading,
    Uploading { sent: u64, total: u64 },
    Inspecting,
}

/* === Traits === */

/* === Implementations === */

impl DroppedFiles {
    pub fn record(&self, window: &str, paths: &[PathBuf]) {
        let drop = FileDrop { window: window.to_owned(), paths: paths.to_vec(), at: Instant::now() };
        *self.0.lock().expect("dropped files poisoned") = Some(drop);
    }

    fn take(&self, window: &str) -> Result<PathBuf, String> {
        let drop = self.0.lock().expect("dropped files poisoned").take();
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

// Stage the bundle just dropped on the window, or pick one natively; `None` when the operator cancels the picker.
pub async fn desktop_plugin_stage_file(
    window: WebviewWindow, state: State<'_, ConnectionSession>, dropped: State<'_, DroppedFiles>, app: AppHandle, generation: String,
    drop: bool, progress: Channel<StageProgress>,
) -> Result<Option<String>, String> {
    let lease = admit(&window, &state, &generation)?;
    let path = if drop {
        dropped.take(window.label())?
    } else {
        let picking = move || pick_bundle(app);
        let Some(path) = tauri::async_runtime::spawn_blocking(picking).await.map_err(text)?? else {
            return Ok(None);
        };
        path
    };

    let _ = progress.send(StageProgress::Reading);
    let reading = move || dokime::harmos::filesystem::input::read(&path, STAGE_ARCHIVE_MAX_BYTES as usize);
    let read = tauri::async_runtime::spawn_blocking(reading).await.map_err(text)?;
    let bytes = read.map_err(|error| format!("The plugin bundle could not be read: {error}"))?;
    stage(&lease, bytes, progress, None).await.map(|staged| Some(staged.install_id))
}

/* === Private Functions === */

pub(super) fn admit(window: &WebviewWindow, state: &ConnectionSession, generation: &str) -> Result<SessionLease, String> {
    ensure_main_window(window)?;
    ensure_generation_token(generation)?;
    state.lease(generation).map_err(|error| format!("Native plugin staging lost its owning connection: {error:?}"))
}

fn pick_bundle(app: AppHandle) -> Result<Option<PathBuf>, String> {
    let dialog = app.dialog().file().add_filter("Plugin bundle", &["dokime"]);
    let Some(path) = dialog.blocking_pick_file() else {
        return Ok(None);
    };
    path.into_path().map(Some).map_err(text)
}

pub(super) async fn stage(
    lease: &SessionLease, bytes: Vec<u8>, progress: Channel<StageProgress>, expected_digest: Option<&str>,
) -> Result<dokime::domain::model::plugins::wire::PluginInstallation, String> {
    // Hash Once, Off The Async Workers
    let started = Instant::now();
    let size_bytes = bytes.len();
    let hashing = move || {
        let sha256 = dokime::client::plugin_bundle_digest(&bytes);
        (bytes, sha256)
    };
    let (bytes, sha256) = tauri::async_runtime::spawn_blocking(hashing).await.map_err(text)?;
    if expected_digest.is_some_and(|expected| !expected.eq_ignore_ascii_case(&sha256)) {
        return Err("The downloaded plugin does not match the release SHA-256".into());
    }

    // Stream The Advertised Chunk Size Over The Shared Client
    let frame_limit = NonZeroUsize::new(crate::rpc::MAX_FRAME_BYTES).expect("desktop frame limit");
    let client = lease.client().with_frame_limit(frame_limit).with_timeout(STAGE_DEADLINE);
    let mut reported: Option<Instant> = None;
    let report = move |sent: u64, total: u64| {
        if sent == total {
            let _ = progress.send(StageProgress::Inspecting);
            return;
        }
        if reported.is_some_and(|last| last.elapsed() < PROGRESS_INTERVAL) {
            return;
        }
        reported = Some(Instant::now());
        let _ = progress.send(StageProgress::Uploading { sent, total });
    };
    let staging = client.plugins_stage_bundle(bytes, sha256, report);
    let staged = lease.guard(staging).await.map_err(|error| format!("{error:?}"))?;
    let staged = staged.map_err(text)?;
    tracing::info!(size_bytes, elapsed_ms = started.elapsed().as_millis(), "plugin bundle staged from the desktop");
    Ok(staged)
}

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drop_stages_once_and_only_a_single_bundle() {
        let dropped = DroppedFiles::default();
        assert!(dropped.take("main").is_err(), "nothing was dropped");

        dropped.record("main", &[PathBuf::from("/bundles/obc-plugin.dokime")]);
        assert!(dropped.take("other").is_err(), "a drop belongs to its window");
        dropped.record("main", &[PathBuf::from("/bundles/obc-plugin.dokime")]);
        assert_eq!(dropped.take("main").unwrap(), PathBuf::from("/bundles/obc-plugin.dokime"));
        assert!(dropped.take("main").is_err(), "a drop is consumed by one staging");

        dropped.record("main", &[PathBuf::from("/bundles/a.dokime"), PathBuf::from("/bundles/b.dokime")]);
        assert!(dropped.take("main").is_err(), "several files are refused");
        dropped.record("main", &[PathBuf::from("/bundles/readme.txt")]);
        assert!(dropped.take("main").is_err(), "a non-bundle is refused");
    }
}
