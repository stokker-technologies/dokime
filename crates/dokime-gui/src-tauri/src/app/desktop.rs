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

use crate::rpc::protocol::FRAME_BYTES as DESKTOP_FRAME_BYTES;

// Standard Library Imports
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt};

// Third Party Imports
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::Deserialize;
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_opener::OpenerExt;

// Local Imports
// ***

//

// Dokime

//

// SPDX-License-Identifier: Apache-2.0

//

// ***
use crate::app::content::{self, DeliveryError, DeliveryStage};
use crate::connection::ConnectionSession;

/* === Constants === */

const DESKTOP_DIAGNOSTICS_BUNDLE_BYTES: u64 = 33_554_432;
const DESKTOP_DOCUMENT_TIMEOUT_MS: u64 = 30_000;
const DESKTOP_JSON_DOCUMENT_BYTES: u64 = 33_554_432;
const DESKTOP_REPORT_FILENAME_CHARS: u64 = 128;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSaveDocument {
    pub kind: DesktopSaveKind,
    pub filename: String,
    pub bytes_base64: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactDeliveryRequest {
    pub source: ArtifactSource,
    pub kind: DesktopSaveKind,
    pub filename: String,
    pub action: DeliveryAction,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DesktopSaveKind {
    HtmlReport,
    DiagnosticsBundle,
    JsonDocument,
    SecretArchive,
    Artifact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExtensionPolicy {
    Append(&'static str),
    Require(&'static str),
    Free,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum ArtifactSource {
    Artifact { artifact_id: String },
    ExecutionFile { execution_id: String, relative_path: String },
    ExecutionEvidence { execution_id: String },
    CycleEvidence { cycle_id: String },
    CycleReport { cycle_id: String },
    ExecutionReport { execution_id: String },
}

// Where a delivery landed, and for a report whether it was sealed or rendered from current evidence.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivered {
    pub path: String,
    pub origin: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryAction {
    Save,
    Open,
}

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for DesktopSaveDocument {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("DesktopSaveDocument").field("kind", &self.kind).field("filename", &self.filename).finish_non_exhaustive()
    }
}

/* === Public Functions === */

pub async fn desktop_open_path(app: AppHandle, path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || app.opener().open_path(path, None::<&str>).map_err(|error| error.to_string()))
        .await
        .map_err(|error| error.to_string())?
}

pub async fn desktop_open_target(app: AppHandle, target: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || app.opener().open_url(target, None::<&str>).map_err(|error| error.to_string()))
        .await
        .map_err(|error| error.to_string())?
}

pub async fn desktop_pairing_create(
    window: WebviewWindow, state: State<'_, ConnectionSession>, app: AppHandle,
) -> Result<Option<PathBuf>, crate::Error> {
    use crate::connection::ConnectionError;
    use dokime::operations::remote::{ReadListener, RevokePairedClient};
    let local = |message: String| ConnectionError::Registry { message };
    ensure_main_window(&window).map_err(local)?;
    let status = state.status();
    let lease = state.lease(&status.generation)?;
    let client = lease.client();
    let listener = client.call(ReadListener {}).await?;
    let settings = listener.settings.ok_or_else(|| local("Listener settings unavailable".into()))?;
    let enable = listener.listening_address.is_none();
    let confirm_app = app.clone();
    let confirmed = if enable {
        tauri::async_runtime::spawn_blocking(move || {
            let warning = concat!(
                "Enable Remote Access on the saved port and export a secret access file? ",
                "Anyone with this file has full control. Transfer it privately."
            );
            confirm_app
                .dialog()
                .message(warning)
                .title("Export Remote Access")
                .kind(MessageDialogKind::Warning)
                .buttons(MessageDialogButtons::OkCancelCustom("Export".into(), "Cancel".into()))
                .blocking_show()
        })
        .await
        .map_err(|_| ConnectionError::BackendUnavailable)?
    } else {
        true
    };
    if !confirmed {
        return Ok(None);
    }
    let now = time::OffsetDateTime::now_utc();
    let mut label = format!(
        "Remote {}-{:02}-{:02} {:02}-{:02}-{:02}",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
    if settings.address.parse::<std::net::SocketAddr>().is_ok_and(|address| address.ip().is_loopback()) {
        label.push_str(" (Same PC)");
    }
    let filename = sanitize_filename(&label, DesktopSaveKind::Artifact, 128).map_err(local)?;
    let selected = tauri::async_runtime::spawn_blocking(move || {
        let picked = app
            .dialog()
            .file()
            .set_file_name(format!("{filename}.dokime-remote"))
            .add_filter("Dokime Remote Access File", &["dokime-remote"])
            .blocking_save_file();
        let Some(picked) = picked else { return Ok(None) };
        let path = picked.into_path().map_err(|error| error.to_string())?;
        let path = if path.extension().is_none() { path.with_extension("dokime-remote") } else { path };
        if path.extension().and_then(|value| value.to_str()) != Some("dokime-remote") {
            return Err("Remote access files require the .dokime-remote extension".into());
        }
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if !metadata.is_file() || metadata.file_type().is_symlink() {
                    return Err("Pairing destination must be a regular file".into());
                }
                let confirmed = app
                    .dialog()
                    .message(format!("Replace {}?", path.display()))
                    .title("Replace Remote Access File")
                    .kind(MessageDialogKind::Warning)
                    .buttons(MessageDialogButtons::OkCancelCustom("Replace".into(), "Cancel".into()))
                    .blocking_show();
                if !confirmed {
                    return Ok(None);
                }
                Ok(Some((path, true)))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Some((path, false))),
            Err(error) => Err(error.to_string()),
        }
    })
    .await
    .map_err(|_| ConnectionError::BackendUnavailable)?
    .map_err(local)?;
    let Some((path, overwrite)) = selected else { return Ok(None) };
    state.with_current(&status.generation, || ())?;
    let client = lease.client();
    let mut created = create_remote_access(&client, label, enable).await?;
    let bytes = serde_json::to_vec_pretty(&created.pairing).map(zeroize::Zeroizing::new);
    if let Some(key) = created.pairing.access_key.as_mut() {
        zeroize::Zeroize::zeroize(key);
    }
    let session = state.inner().clone();
    let generation = status.generation;
    let destination = path.clone();
    let saved = match bytes {
        Ok(bytes) => tauri::async_runtime::spawn_blocking(move || {
            use std::io::Write;
            let parent = destination.parent().ok_or("Pairing destination has no parent")?;
            let mut staged = tempfile::Builder::new().prefix(".dokime-remote-").tempfile_in(parent).map_err(|error| error.to_string())?;
            set_private_file(staged.path())?;
            staged.write_all(&bytes).map_err(|error| error.to_string())?;
            staged.as_file().sync_all().map_err(|error| error.to_string())?;
            session
                .with_current(&generation, || {
                    if overwrite {
                        dokime::harmos::filesystem::publication::replace(staged.path(), &destination)
                    } else {
                        dokime::harmos::filesystem::publication::install(staged.path(), &destination)
                    }
                })
                .map_err(|error| format!("{error:?}"))?
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|_| "Pairing file writer failed".to_owned())
        .and_then(|result| result),
        Err(_) => Err("Pairing file could not be encoded".into()),
    };
    if let Err(message) = saved {
        if let Some(paired) = created.client {
            client
                .call(RevokePairedClient { identifier: paired.identifier })
                .await
                .map_err(|_| local(format!("{message}; the new paired client could not be revoked. Revoke it in Paired Clients.")))?;
        }
        return Err(local(message).into());
    }
    Ok(Some(path))
}

pub async fn create_remote_access(
    client: &dokime::client::Client, label: String, enable: bool,
) -> Result<dokime::domain::model::remote::CreatedPairing, crate::Error> {
    use dokime::operations::remote::{CreatePairing, ReadListener, UpdateListener};
    if enable {
        let mut settings = client.call(ReadListener {}).await?.settings.ok_or_else(|| crate::connection::ConnectionError::Registry {
            message: "Listener settings unavailable".into(),
        })?;
        settings.enabled = true;
        client.call(UpdateListener { settings: Some(settings) }).await?;
    }
    Ok(client.call(CreatePairing { label }).await?)
}

pub async fn desktop_save_document(
    window: WebviewWindow, state: State<'_, ConnectionSession>, app: AppHandle, generation: String, document: DesktopSaveDocument,
) -> Result<Option<String>, String> {
    ensure_main_window(&window)?;
    ensure_picker_kind(document.kind)?;
    let (filename, bytes, kind) = validate_save_document(&generation, &document)?;
    let session = state.inner().clone();
    ensure_document_session(&session, &generation)?;
    let selected = tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog().file().set_file_name(filename);
        let dialog = apply_dialog_filter(dialog, kind);
        dialog.blocking_save_file().map(|path| path.into_path().map_err(|error| error.to_string()))
    })
    .await
    .map_err(|error| error.to_string())?;
    let Some(path) = selected.transpose()? else {
        return Ok(None);
    };
    let path = ensure_selected_extension(path, kind)?;
    ensure_document_session(&session, &generation)?;
    write_document(&session, &generation, path.clone(), bytes, false).await?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

pub async fn desktop_pick_secret_archive(
    window: WebviewWindow, state: State<'_, ConnectionSession>, app: AppHandle, generation: String,
) -> Result<Option<String>, String> {
    ensure_main_window(&window)?;
    let session = state.inner().clone();
    ensure_document_session(&session, &generation)?;
    tauri::async_runtime::spawn_blocking(move || {
        let picked = app.dialog().file().add_filter("Secret Archive", &["b64"]).blocking_pick_file();
        let Some(picked) = picked else { return Ok(None) };
        ensure_document_session(&session, &generation)?;
        let path = picked.into_path().map_err(|error| error.to_string())?;
        let maximum = 699_052u64 as usize;
        let bytes = dokime::harmos::filesystem::input::read(&path, maximum).map_err(|_| "Secret archive is unreadable or oversized")?;
        ensure_document_session(&session, &generation)?;
        Ok(Some(BASE64.encode(bytes)))
    })
    .await
    .map_err(|_| "Secret archive picker failed".to_owned())?
}

pub async fn desktop_save_diagnostics_bundle(
    window: WebviewWindow, state: State<'_, ConnectionSession>, app: AppHandle, generation: String, document: DesktopSaveDocument,
) -> Result<String, String> {
    ensure_main_window(&window)?;
    if document.kind != DesktopSaveKind::DiagnosticsBundle {
        return Err("Native diagnostics delivery accepts only the diagnostics_bundle kind".into());
    }
    let (filename, bytes, _) = validate_save_document(&generation, &document)?;
    let session = state.inner().clone();
    ensure_document_session(&session, &generation)?;
    let downloads = app.path().download_dir().map_err(|error| format!("The Downloads folder could not be resolved: {error}"))?;
    let path = downloads.join(filename);
    // `dokime::harmos::filesystem::publication::write` stages beside the destination and
    // renames into place, so a partially written archive is never visible under
    // the final name and a missing Downloads folder is created first.
    write_document(&session, &generation, path.clone(), bytes, false).await?;
    Ok(path.to_string_lossy().into_owned())
}

pub async fn desktop_artifact_deliver(
    window: WebviewWindow, state: State<'_, ConnectionSession>, app: AppHandle, generation: String, request: ArtifactDeliveryRequest,
) -> Result<Option<Delivered>, DeliveryError> {
    let prepare = |message: String| DeliveryError::local(DeliveryStage::Prepare, message);

    // Prepare The Destination Before Requesting Content
    ensure_main_window(&window).map_err(prepare)?;
    ensure_generation_token(&generation).map_err(prepare)?;
    ensure_picker_kind(request.kind).map_err(prepare)?;
    let session = state.inner().clone();
    let filename = sanitize_filename(&request.filename, request.kind, DESKTOP_REPORT_FILENAME_CHARS as usize).map_err(prepare)?;
    if request.action == DeliveryAction::Save {
        let picker_app = app.clone();
        let picker_filename = filename.clone();
        let kind = request.kind;
        let picked = async move {
            tauri::async_runtime::spawn_blocking(move || pick_destination(picker_app, picker_filename, kind, true))
                .await
                .map_err(|error| error.to_string())?
        };
        return save_verified(session, generation, request.source, filename, picked).await;
    }
    let lease = session.lease(&generation).map_err(|error| prepare(format!("{error:?}")))?;
    ensure_document_session(&session, &generation).map_err(prepare)?;
    let directory = app.state::<dokime::paths::ApplicationPaths>().cache().join("documents");
    let cache = directory.clone();
    tauri::async_runtime::spawn_blocking(move || ensure_private_directory(&cache))
        .await
        .map_err(|error| prepare(error.to_string()))?
        .map_err(prepare)?;
    let staged = tempfile::Builder::new()
        .prefix(".dokime-")
        .suffix(&format!("-{filename}"))
        .tempfile_in(directory)
        .map_err(|error| prepare(error.to_string()))?
        .into_temp_path();
    // The Download API Creates Its Own Output File; Keep The Unique Name And Cleanup Guard.
    std::fs::remove_file(&staged).map_err(|error| prepare(error.to_string()))?;

    // Stream And Verify Beside The Chosen Destination Or In The Open Cache
    let verified = content::fetch_verified(&lease, &request.source, &staged, DESKTOP_FRAME_BYTES as usize).await?;
    let cleanup = staged;
    ensure_document_session(&session, &generation).map_err(prepare)?;

    // Open The Verified Cache File
    let open = |message: String| DeliveryError::local(DeliveryStage::Open, message);
    let path = verified.path.clone();
    tauri::async_runtime::spawn_blocking(move || set_private_file(&path)).await.map_err(|error| open(error.to_string()))?.map_err(open)?;
    ensure_document_session(&session, &generation).map_err(prepare)?;
    // The opener call itself forks/execs a launcher process; run it
    // on the blocking pool so it never occupies a Tokio worker
    // thread other in-flight async commands need.
    let path = verified.path.to_string_lossy().into_owned();
    let launched = path.clone();
    let automation = (std::env::var("TAURI_WEBVIEW_AUTOMATION").as_deref() == Ok("true"))
        .then(|| std::env::var("DOKIME_AUTOMATION_REPORT_OPENER").ok())
        .flatten();
    if let Some(endpoint) = automation {
        let endpoint = url::Url::parse(&endpoint).map_err(|error| open(error.to_string()))?;
        if endpoint.scheme() != "http"
            || endpoint.host_str() != Some("127.0.0.1")
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || endpoint.path() != "/open"
        {
            return Err(open("The automation report opener must be an owned loopback endpoint.".to_owned()));
        }
        ensure_document_session(&session, &generation).map_err(prepare)?;
        let client = crate::app::http::client()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| open(error.to_string()))?;
        let response = client
            .post(endpoint)
            .json(&serde_json::json!({ "path": launched }))
            .timeout(std::time::Duration::from_millis(DESKTOP_DOCUMENT_TIMEOUT_MS))
            .send()
            .await
            .map_err(|error| open(error.to_string()))?;
        response.error_for_status().map_err(|error| open(error.to_string()))?;
    } else {
        tauri::async_runtime::spawn_blocking(move || {
            let opener = app.opener();
            let opened = session.with_current(&generation, || opener.open_path(launched, None::<&str>));
            let opened = opened.map_err(|error| format!("Native report delivery lost its owning connection: {error:?}"))?;
            opened.map_err(|error| error.to_string())
        })
        .await
        .map_err(|error| open(error.to_string()))?
        .map_err(open)?;
    }
    cleanup.keep().map_err(|error| open(error.to_string()))?;
    Ok(Some(Delivered { path, origin: verified.report_origin() }))
}

// This Workflow Owns Picker Ordering, Adjacent Staging And Fenced Publication.
// Tests Supply A Picker Result; Download And Publication Stay On The Real Path.
pub async fn save_verified(
    session: ConnectionSession, generation: String, source: ArtifactSource, filename: String,
    picked: impl std::future::Future<Output = Result<Option<(PathBuf, bool)>, String>>,
) -> Result<Option<Delivered>, DeliveryError> {
    let prepare = |message: String| DeliveryError::local(DeliveryStage::Prepare, message);
    let lease = session.lease(&generation).map_err(|error| prepare(format!("{error:?}")))?;
    let Some((destination, overwrite)) = picked.await.map_err(prepare)? else { return Ok(None) };
    ensure_document_session(&session, &generation).map_err(prepare)?;
    let directory = destination.parent().ok_or_else(|| prepare("Destination has no parent".into()))?;
    let staged = tempfile::Builder::new()
        .prefix(".dokime-")
        .suffix(&format!("-{filename}"))
        .tempfile_in(directory)
        .map_err(|error| prepare(error.to_string()))?
        .into_temp_path();
    std::fs::remove_file(&staged).map_err(|error| prepare(error.to_string()))?;
    let verified = content::fetch_verified(&lease, &source, &staged, DESKTOP_FRAME_BYTES as usize).await?;
    ensure_document_session(&session, &generation).map_err(prepare)?;
    let origin = verified.report_origin();
    move_verified_file(session, generation, verified.path, destination.clone(), overwrite)
        .await
        .map_err(|message| DeliveryError::local(DeliveryStage::Rename, message))?;
    Ok(Some(Delivered {
        path: destination.to_string_lossy().into_owned(),
        origin,
    }))
}

pub fn desktop_set_title(window: WebviewWindow, title: String) -> Result<(), String> {
    set_window_title(&window, &title).map_err(|error| error.to_string())
}

// === Native document delivery helpers === //

pub(crate) fn ensure_main_window<R: tauri::Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    if window.label() == "main" { Ok(()) } else { Err("Native document delivery is restricted to the main window".into()) }
}

pub(super) fn ensure_generation_token(generation: &str) -> Result<(), String> {
    if generation.is_empty() || !generation.chars().all(|value| value.is_ascii_digit()) {
        return Err("Native document delivery generation is invalid".into());
    }
    Ok(())
}

pub(super) fn ensure_private_directory(path: &Path) -> Result<(), String> {
    protect(
        path,
        true,
        |path| dokime::harmos::filesystem::publication::create_dir_all(path).map_err(|error| error.to_string()),
        restrict_to_owner,
    )
}

pub(super) fn set_private_file(path: &Path) -> Result<(), String> {
    protect(path, false, |_| Ok(()), restrict_to_owner)
}

/* === Private Functions === */

fn protect(
    path: &Path, directory: bool, create: impl FnOnce(&Path) -> Result<(), String>, restrict: impl FnOnce(&Path, u32) -> Result<(), String>,
) -> Result<(), String> {
    if directory {
        create(path)?;
    }
    restrict(path, if directory { 0o700 } else { 0o600 })
}

fn set_window_title(window: &WebviewWindow, title: &str) -> tauri::Result<()> {
    let title = title.trim();
    if title.is_empty() {
        return Ok(());
    }
    window.set_title(title)
}

fn validate_save_document(generation: &str, document: &DesktopSaveDocument) -> Result<(String, Vec<u8>, DesktopSaveKind), String> {
    ensure_generation_token(generation)?;
    let (max_bytes, label) = match document.kind {
        DesktopSaveKind::HtmlReport => return Err("Report HTML is delivered through desktop_artifact_deliver".into()),
        DesktopSaveKind::DiagnosticsBundle => (DESKTOP_DIAGNOSTICS_BUNDLE_BYTES as usize, "Diagnostics Bundle"),
        DesktopSaveKind::JsonDocument => (DESKTOP_JSON_DOCUMENT_BYTES as usize, "JSON Document"),
        DesktopSaveKind::SecretArchive => (699_052u64 as usize, "Secret archive"),
        DesktopSaveKind::Artifact => return Err("Artifact is delivered through desktop_artifact_deliver".into()),
    };
    let max_encoded_chars = max_bytes.div_ceil(3).saturating_mul(4);

    let filename = sanitize_filename(&document.filename, document.kind, DESKTOP_REPORT_FILENAME_CHARS as usize)?;
    let bytes = decode_bounded(&document.bytes_base64, max_encoded_chars, max_bytes, label)?;
    Ok((filename, bytes, document.kind))
}

fn ensure_picker_kind(kind: DesktopSaveKind) -> Result<(), String> {
    if kind == DesktopSaveKind::DiagnosticsBundle {
        return Err("Diagnostics bundle is delivered through desktop_save_diagnostics_bundle".into());
    }
    Ok(())
}

fn apply_dialog_filter(
    dialog: tauri_plugin_dialog::FileDialogBuilder<tauri::Wry>, kind: DesktopSaveKind,
) -> tauri_plugin_dialog::FileDialogBuilder<tauri::Wry> {
    match kind {
        DesktopSaveKind::HtmlReport => dialog.add_filter("HTML Report", &["html"]),
        // Unreachable while `ensure_picker_kind` gates every picker route; kept
        // truthful so a future route cannot pick up a wrong filter by default.
        DesktopSaveKind::DiagnosticsBundle => dialog.add_filter("Diagnostics Bundle", &["zip"]),
        DesktopSaveKind::JsonDocument => dialog.add_filter("JSON Document", &["json"]),
        DesktopSaveKind::SecretArchive => dialog.add_filter("Secret Archive", &["b64"]),
        DesktopSaveKind::Artifact => dialog,
    }
}

fn extension_policy(kind: DesktopSaveKind) -> ExtensionPolicy {
    match kind {
        DesktopSaveKind::HtmlReport => ExtensionPolicy::Append(".html"),
        DesktopSaveKind::DiagnosticsBundle => ExtensionPolicy::Require(".zip"),
        DesktopSaveKind::JsonDocument => ExtensionPolicy::Append(".json"),
        DesktopSaveKind::SecretArchive => ExtensionPolicy::Append(".b64"),
        DesktopSaveKind::Artifact => ExtensionPolicy::Free,
    }
}

fn decode_bounded(encoded: &str, max_encoded_chars: usize, max_bytes: usize, label: &str) -> Result<Vec<u8>, String> {
    if encoded.len() > max_encoded_chars {
        return Err(format!("{label} bytes exceed the native delivery limit before decoding"));
    }
    let bytes = BASE64.decode(encoded).map_err(|error| format!("{label} bytes are not valid base64: {error}"))?;
    if bytes.len() > max_bytes {
        return Err(format!("{label} exceeds the {max_bytes}-byte native delivery limit"));
    }
    Ok(bytes)
}

fn ensure_document_session(session: &ConnectionSession, generation: &str) -> Result<(), String> {
    session.lease(generation).map(|_| ()).map_err(|error| format!("Native document delivery lost its owning connection: {error:?}"))
}

fn sanitize_filename(filename: &str, kind: DesktopSaveKind, maximum: usize) -> Result<String, String> {
    if maximum < 5 {
        return Err("Document filename bound cannot preserve its extension".into());
    }
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err("Document filename is empty".into());
    }
    if trimmed.chars().count() > maximum.saturating_mul(2) {
        return Err("Document filename is too long".into());
    }
    let mut safe = trimmed
        .chars()
        .map(|value| match value {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            value if value.is_control() => '_',
            value => value,
        })
        .collect::<String>();
    while safe.ends_with('.') || safe.ends_with(' ') {
        safe.pop();
    }
    if safe.is_empty() || safe == "." || safe == ".." {
        return Err("Document filename is invalid".into());
    }
    // The extension is settled before the length bound so truncation can
    // neither destroy a required suffix nor manufacture one.
    Ok(match extension_policy(kind) {
        ExtensionPolicy::Append(suffix) => {
            if !safe.to_ascii_lowercase().ends_with(suffix) {
                safe.push_str(suffix);
            }
            bound_with_suffix(safe, suffix, maximum)
        }
        ExtensionPolicy::Require(suffix) => {
            let lowered = safe.to_ascii_lowercase();
            if !lowered.ends_with(suffix) || lowered.len() == suffix.len() {
                return Err(format!("Document filename must name a {suffix} file"));
            }
            bound_with_suffix(safe, suffix, maximum)
        }
        ExtensionPolicy::Free if safe.chars().count() > maximum => safe.chars().take(maximum).collect(),
        ExtensionPolicy::Free => safe,
    })
}

fn bound_with_suffix(safe: String, suffix: &str, maximum: usize) -> String {
    if safe.chars().count() <= maximum {
        return safe;
    }
    let prefix = safe.chars().take(maximum - suffix.chars().count()).collect::<String>();
    format!("{prefix}{suffix}")
}

fn ensure_selected_extension(path: PathBuf, kind: DesktopSaveKind) -> Result<PathBuf, String> {
    let expected = match extension_policy(kind) {
        ExtensionPolicy::Append(suffix) | ExtensionPolicy::Require(suffix) => suffix.trim_start_matches('.'),
        ExtensionPolicy::Free => return Ok(path),
    };
    match path.extension().and_then(|extension| extension.to_str()) {
        None => Ok(path.with_extension(expected)),
        Some(extension) if extension.eq_ignore_ascii_case(expected) => Ok(path),
        Some(_) => Err(format!("Selected path must use the .{expected} extension")),
    }
}

#[cfg(unix)]
fn restrict_to_owner(path: &Path, mode: u32) -> Result<(), String> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|error| error.to_string())
}

#[cfg(windows)]
fn restrict_to_owner(path: &Path, mode: u32) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::{
            Foundation::{HLOCAL, LocalFree},
            Security::{
                Authorization::{
                    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1, SE_FILE_OBJECT, SetNamedSecurityInfoW,
                },
                DACL_SECURITY_INFORMATION, GetSecurityDescriptorDacl, PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
            },
        },
        core::{BOOL, PCWSTR},
    };
    let rule = if mode == 0o700 { "D:P(A;OICI;FA;;;OW)" } else { "D:P(A;;FA;;;OW)" };
    let rule: Vec<u16> = rule.encode_utf16().chain(Some(0)).collect();
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // The Descriptor Owns The ACL Until SetNamedSecurityInfo Has Copied It.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(PCWSTR(rule.as_ptr()), SDDL_REVISION_1, &mut descriptor, None)
            .map_err(|error| error.to_string())?;
        let result = (|| {
            let mut present = BOOL::default();
            let mut defaulted = BOOL::default();
            let mut acl = std::ptr::null_mut();
            GetSecurityDescriptorDacl(descriptor, &mut present, &mut acl, &mut defaulted).map_err(|error| error.to_string())?;
            if !present.as_bool() || acl.is_null() {
                return Err("Private file ACL is absent".into());
            }
            SetNamedSecurityInfoW(
                PCWSTR(path.as_ptr()),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(acl),
                None,
            )
            .ok()
            .map_err(|error| error.to_string())
        })();
        let _ = LocalFree(Some(HLOCAL(descriptor.0)));
        result
    }
}

#[cfg(not(any(unix, windows)))]
fn restrict_to_owner(_path: &Path, _mode: u32) -> Result<(), String> {
    Err("Private file permissions are unavailable on this platform".into())
}

async fn write_document(session: &ConnectionSession, generation: &str, path: PathBuf, bytes: Vec<u8>, private: bool) -> Result<(), String> {
    let session = session.clone();
    let generation = generation.to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = zeroize::Zeroizing::new(bytes);
        ensure_document_session(&session, &generation)?;
        if private && let Some(parent) = path.parent() {
            ensure_private_directory(parent)?;
        }
        dokime::harmos::filesystem::publication::write(&path, &bytes).map_err(|error| error.to_string())?;
        if private {
            set_private_file(&path)?;
        }
        ensure_document_session(&session, &generation)
    })
    .await
    .map_err(|error| error.to_string())?
}

fn pick_destination(
    app: AppHandle, filename: String, kind: DesktopSaveKind, explicit_overwrite: bool,
) -> Result<Option<(PathBuf, bool)>, String> {
    let dialog = app.dialog().file().set_file_name(filename);
    let dialog = apply_dialog_filter(dialog, kind);
    let Some(path) = dialog.blocking_save_file() else {
        return Ok(None);
    };
    let path = path.into_path().map_err(|error| error.to_string())?;
    let destination = ensure_selected_extension(path, kind)?;
    if !explicit_overwrite {
        return Ok(Some((destination, true)));
    }

    let metadata = match std::fs::symlink_metadata(&destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Some((destination, false))),
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Destination must be a regular file".into());
    }
    let message = format!("Replace {} with the verified file?", destination.display());
    let confirmed = app
        .dialog()
        .message(message)
        .title("Replace File")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom("Replace".into(), "Cancel".into()))
        .blocking_show();
    Ok(confirmed.then_some((destination, true)))
}

pub async fn move_verified_file(
    session: ConnectionSession, generation: String, source: PathBuf, destination: PathBuf, overwrite: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _source = tempfile::TempPath::try_from_path(source.clone()).map_err(|error| error.to_string())?;
        let input = std::fs::OpenOptions::new().read(true).write(true).open(&source).map_err(|error| error.to_string())?;
        input.sync_all().map_err(|error| error.to_string())?;
        drop(input);
        let staged_path = &source;
        let published = session.with_current(&generation, || {
            if overwrite {
                dokime::harmos::filesystem::publication::replace(staged_path, &destination)
            } else {
                dokime::harmos::filesystem::publication::install(staged_path, &destination)
            }
        });
        let published = published.map_err(|error| format!("Native delivery lost its owning connection: {error:?}"))?;
        published.map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_validation_error_retains_native_rpc_reason() {
        let status = tonic::Status::invalid_argument("fixture-endpoint-rejection");
        let expected = status.message().to_owned();
        let error = crate::Error::from(dokime::client::Error::Rpc(status));
        let encoded = serde_json::to_value(error).unwrap();
        assert_eq!(encoded["kind"], "grpc");
        assert_eq!(encoded["detail"]["code"], 3);
        assert_eq!(encoded["detail"]["message"], expected);
    }

    #[test]
    fn reports_require_verified_native_delivery() {
        let document = DesktopSaveDocument {
            kind: DesktopSaveKind::HtmlReport,
            filename: "report.html".into(),
            bytes_base64: BASE64.encode("<html></html>"),
        };
        let error = validate_save_document("7", &document).unwrap_err();
        assert!(error.contains("desktop_artifact_deliver"));
    }

    fn json_document(filename: &str, bytes: &[u8]) -> DesktopSaveDocument {
        serde_json::from_value(serde_json::json!({
            "kind": "json_document",
            "filename": filename,
            "bytesBase64": BASE64.encode(bytes),
        }))
        .unwrap()
    }

    #[test]
    fn json_document_saves_through_the_picker() {
        let bytes = br#"{"groups":[]}"#;
        let document = json_document("Cycle-A-3-traceability.json", bytes);

        ensure_picker_kind(document.kind).unwrap();
        let (filename, decoded, kind) = validate_save_document("7", &document).unwrap();

        assert_eq!(kind, DesktopSaveKind::JsonDocument);
        assert_eq!(filename, "Cycle-A-3-traceability.json");
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn json_document_names_a_json_file() {
        let (filename, _, _) = validate_save_document("7", &json_document("trace:ability", b"{}")).unwrap();
        assert_eq!(filename, "trace_ability.json");

        let kind = DesktopSaveKind::JsonDocument;
        assert_eq!(ensure_selected_extension(PathBuf::from("/tmp/trace"), kind).unwrap(), PathBuf::from("/tmp/trace.json"));
        assert_eq!(ensure_selected_extension(PathBuf::from("/tmp/trace.JSON"), kind).unwrap(), PathBuf::from("/tmp/trace.JSON"));
        assert!(ensure_selected_extension(PathBuf::from("/tmp/trace.txt"), kind).is_err());
    }

    #[test]
    fn json_document_stays_bounded() {
        let mut document = json_document("trace.json", b"{}");
        document.bytes_base64 = "A".repeat((DESKTOP_JSON_DOCUMENT_BYTES as usize).div_ceil(3) * 4 + 4);

        let error = validate_save_document("7", &document).unwrap_err();

        assert!(error.contains("JSON Document"), "{error}");
    }
}
