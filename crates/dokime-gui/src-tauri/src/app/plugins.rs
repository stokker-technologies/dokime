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
use std::str::FromStr;
use std::time::{Duration, Instant};

// Third Party Imports
use tauri::ipc::{Channel, InvokeBody, JavaScriptChannelId, Request};
use tauri::{AppHandle, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

// Local Imports
use super::desktop::{ensure_generation_token, ensure_main_window};
use crate::connection::{ConnectionSession, SessionLease};
use dokime_domain::model::plugins::wire::STAGE_ARCHIVE_MAX_BYTES;

/* === Constants === */

pub const GENERATION_HEADER: &str = "dokime-plugin-generation";
pub const PROGRESS_HEADER: &str = "dokime-plugin-progress";
const STAGE_DEADLINE: Duration = Duration::from_secs(300);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

/* === Public Functions === */

// Pick a bundle with the native dialog and stage it; `None` when the operator cancels.
pub async fn desktop_plugin_stage_file(
    window: WebviewWindow, state: State<'_, ConnectionSession>, app: AppHandle, generation: String, progress: Channel<StageProgress>,
) -> Result<Option<String>, String> {
    let lease = admit(&window, &state, &generation)?;
    let picking = move || pick_bundle(app);
    let Some(path) = tauri::async_runtime::spawn_blocking(picking).await.map_err(text)?? else {
        return Ok(None);
    };

    let _ = progress.send(StageProgress::Reading);
    let reading = move || harmos::filesystem::input::read(&path, STAGE_ARCHIVE_MAX_BYTES as usize);
    let read = tauri::async_runtime::spawn_blocking(reading).await.map_err(text)?;
    let bytes = read.map_err(|error| format!("The plugin bundle could not be read: {error}"))?;
    stage(&lease, bytes, progress).await.map(Some)
}

// Stage a dropped bundle whose bytes cross the bridge as one raw invoke body.
pub async fn desktop_plugin_stage_bytes(
    window: WebviewWindow, state: State<'_, ConnectionSession>, request: Request<'_>,
) -> Result<String, String> {
    let generation = header(&request, GENERATION_HEADER)?;
    let lease = admit(&window, &state, generation)?;
    let channel = header(&request, PROGRESS_HEADER)?;
    let channel = JavaScriptChannelId::from_str(channel).map_err(|_| "Native plugin staging has an invalid progress channel")?;
    let progress = channel.channel_on(window.as_ref().clone());
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("Native plugin staging expects a raw bundle body".into());
    };
    stage(&lease, bytes.clone(), progress).await
}

/* === Private Functions === */

fn admit(window: &WebviewWindow, state: &ConnectionSession, generation: &str) -> Result<SessionLease, String> {
    ensure_main_window(window)?;
    ensure_generation_token(generation)?;
    state.lease(generation).map_err(|error| format!("Native plugin staging lost its owning connection: {error:?}"))
}

fn header<'a>(request: &'a Request<'_>, name: &str) -> Result<&'a str, String> {
    let value = request.headers().get(name).ok_or_else(|| format!("Native plugin staging is missing {name}"))?;
    value.to_str().map_err(|_| format!("Native plugin staging has an invalid {name}"))
}

fn pick_bundle(app: AppHandle) -> Result<Option<PathBuf>, String> {
    let dialog = app.dialog().file().add_filter("Plugin bundle", &["dokime"]);
    let Some(path) = dialog.blocking_pick_file() else {
        return Ok(None);
    };
    path.into_path().map(Some).map_err(text)
}

async fn stage(lease: &SessionLease, bytes: Vec<u8>, progress: Channel<StageProgress>) -> Result<String, String> {
    // Hash Once, Off The Async Workers
    let started = Instant::now();
    let size_bytes = bytes.len();
    let hashing = move || {
        let sha256 = dokime_client::plugin_bundle_digest(&bytes);
        (bytes, sha256)
    };
    let (bytes, sha256) = tauri::async_runtime::spawn_blocking(hashing).await.map_err(text)?;

    // Stream Large Frames Over The Shared Client
    let frame_limit = NonZeroUsize::new(crate::rpc::MAX_FRAME_BYTES).expect("desktop frame limit");
    let client = lease.client().with_frame_limit(frame_limit).with_timeout(STAGE_DEADLINE);
    let report = move |sent: u64, total: u64| {
        let _ = progress.send(StageProgress::Uploading { sent, total });
        if sent == total {
            let _ = progress.send(StageProgress::Inspecting);
        }
    };
    let staging = client.plugins_stage_bundle(bytes, sha256, report);
    let staged = lease.guard(staging).await.map_err(|error| format!("{error:?}"))?;
    let staged = staged.map_err(text)?;
    tracing::info!(size_bytes, elapsed_ms = started.elapsed().as_millis(), "plugin bundle staged from the desktop");
    Ok(staged.install_id)
}

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/* === Tests === */
