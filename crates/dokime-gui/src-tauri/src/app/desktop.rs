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
use rand::random;
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
const DESKTOP_PUBLIC_CERTIFICATE_BYTES: u64 = 65_536;
const DESKTOP_REMOTE_KEY_BYTES: u64 = 4_098;
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
    RemoteKey,
    PublicCertificate,
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

    // Prepare: Owning Window, Session Lease, Filename And Private Staging Cache
    ensure_main_window(&window).map_err(prepare)?;
    ensure_generation_token(&generation).map_err(prepare)?;
    ensure_picker_kind(request.kind).map_err(prepare)?;
    let session = state.inner().clone();
    let lease =
        session.lease(&generation).map_err(|error| prepare(format!("Native artifact delivery lost its owning connection: {error:?}")))?;
    let filename = sanitize_filename(&request.filename, request.kind, DESKTOP_REPORT_FILENAME_CHARS as usize).map_err(prepare)?;
    let cache = app.path().app_cache_dir().map_err(|error| prepare(error.to_string()))?.join("documents");
    tauri::async_runtime::spawn_blocking({
        let cache = cache.clone();
        move || ensure_private_directory(&cache)
    })
    .await
    .map_err(|error| prepare(error.to_string()))?
    .map_err(prepare)?;

    // Request, Stream, Write: The Shared Client Pulls And Verifies Into The Cache
    let staged = cache.join(format!("{generation}-{}-{filename}", random::<u64>()));
    let verified = content::fetch_verified(&lease, &request.source, &staged, DESKTOP_FRAME_BYTES as usize).await?;
    let cleanup = tempfile::TempPath::try_from_path(staged).map_err(|error| prepare(error.to_string()))?;
    ensure_document_session(&session, &generation).map_err(prepare)?;

    // Rename Or Open: Hand The Verified File To Its Destination
    match request.action {
        DeliveryAction::Save => {
            let kind = request.kind;
            let explicit_overwrite =
                matches!(request.source, ArtifactSource::ExecutionEvidence { .. } | ArtifactSource::CycleEvidence { .. });
            let selected = tauri::async_runtime::spawn_blocking(move || pick_destination(app, filename, kind, explicit_overwrite))
                .await
                .map_err(|error| prepare(error.to_string()))?
                .map_err(prepare)?;
            let Some((destination, overwrite)) = selected else {
                let _ = tokio::fs::remove_file(&verified.path).await;
                return Ok(None);
            };
            ensure_document_session(&session, &generation).map_err(prepare)?;
            let origin = verified.report_origin();
            move_verified_file(session, generation, verified.path, destination.clone(), overwrite)
                .await
                .map_err(|message| DeliveryError::local(DeliveryStage::Rename, message))?;
            Ok(Some(Delivered { path: destination.to_string_lossy().into_owned(), origin }))
        }
        DeliveryAction::Open => {
            let open = |message: String| DeliveryError::local(DeliveryStage::Open, message);
            let path = verified.path.clone();
            tauri::async_runtime::spawn_blocking(move || set_private_file(&path))
                .await
                .map_err(|error| open(error.to_string()))?
                .map_err(open)?;
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
                let client = reqwest::Client::builder()
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
    }
}

pub fn desktop_set_title(window: WebviewWindow, title: String) -> Result<(), String> {
    set_window_title(&window, &title).map_err(|error| error.to_string())
}

// === Native document delivery helpers === //

pub(super) fn ensure_main_window(window: &WebviewWindow) -> Result<(), String> {
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
        DesktopSaveKind::DiagnosticsBundle => (DESKTOP_DIAGNOSTICS_BUNDLE_BYTES as usize, "Diagnostics bundle"),
        DesktopSaveKind::JsonDocument => (DESKTOP_JSON_DOCUMENT_BYTES as usize, "JSON document"),
        DesktopSaveKind::RemoteKey => (DESKTOP_REMOTE_KEY_BYTES as usize, "Remote key"),
        DesktopSaveKind::PublicCertificate => (DESKTOP_PUBLIC_CERTIFICATE_BYTES as usize, "Public certificate"),
        DesktopSaveKind::SecretArchive => (699_052u64 as usize, "Secret archive"),
        DesktopSaveKind::Artifact => return Err("Artifact is delivered through desktop_artifact_deliver".into()),
    };
    let max_encoded_chars = max_bytes.div_ceil(3).saturating_mul(4);

    let filename = sanitize_filename(&document.filename, document.kind, DESKTOP_REPORT_FILENAME_CHARS as usize)?;
    let bytes = decode_bounded(&document.bytes_base64, max_encoded_chars, max_bytes, label)?;
    if document.kind == DesktopSaveKind::RemoteKey {
        let key = std::str::from_utf8(&bytes).map_err(|_| "Remote key must be UTF-8 text")?.trim();
        if !(32..=4096).contains(&key.len()) || !key.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err("Remote key must contain 32–4096 visible ASCII characters".into());
        }
    }
    if document.kind == DesktopSaveKind::PublicCertificate {
        let certificate = std::str::from_utf8(&bytes).map_err(|_| "Public certificate must be UTF-8 PEM text")?;
        if !certificate.contains("-----BEGIN CERTIFICATE-----") || certificate.contains("PRIVATE KEY") {
            return Err("Public certificate delivery accepts certificate PEM only".into());
        }
    }

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
        DesktopSaveKind::HtmlReport => dialog.add_filter("HTML report", &["html"]),
        // Unreachable while `ensure_picker_kind` gates every picker route; kept
        // truthful so a future route cannot pick up a wrong filter by default.
        DesktopSaveKind::DiagnosticsBundle => dialog.add_filter("Diagnostics bundle", &["zip"]),
        DesktopSaveKind::JsonDocument => dialog.add_filter("JSON document", &["json"]),
        DesktopSaveKind::RemoteKey => dialog.add_filter("Remote key", &["key"]),
        DesktopSaveKind::PublicCertificate => dialog.add_filter("Public certificate", &["pem"]),
        DesktopSaveKind::SecretArchive => dialog.add_filter("Secret Archive", &["b64"]),
        DesktopSaveKind::Artifact => dialog,
    }
}

fn extension_policy(kind: DesktopSaveKind) -> ExtensionPolicy {
    match kind {
        DesktopSaveKind::HtmlReport => ExtensionPolicy::Append(".html"),
        DesktopSaveKind::DiagnosticsBundle => ExtensionPolicy::Require(".zip"),
        DesktopSaveKind::JsonDocument => ExtensionPolicy::Append(".json"),
        DesktopSaveKind::RemoteKey => ExtensionPolicy::Append(".key"),
        DesktopSaveKind::PublicCertificate => ExtensionPolicy::Append(".pem"),
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

#[cfg(not(unix))]
fn restrict_to_owner(_path: &Path, _mode: u32) -> Result<(), String> {
    Ok(())
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
        return Err("Evidence destination must be a regular file".into());
    }
    let message = format!("Replace {} with the verified evidence archive?", destination.display());
    let confirmed = app
        .dialog()
        .message(message)
        .title("Replace Evidence File")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom("Replace".into(), "Cancel".into()))
        .blocking_show();
    Ok(confirmed.then_some((destination, true)))
}

async fn move_verified_file(
    session: ConnectionSession, generation: String, source: PathBuf, destination: PathBuf, overwrite: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _source = tempfile::TempPath::try_from_path(source.clone()).map_err(|error| error.to_string())?;
        let parent = destination.parent().ok_or("destination has no parent directory")?;
        let mut staged = tempfile::Builder::new().prefix(".dokime-").tempfile_in(parent).map_err(|error| error.to_string())?;
        let mut input = std::fs::File::open(source).map_err(|error| error.to_string())?;
        std::io::copy(&mut input, staged.as_file_mut()).map_err(|error| error.to_string())?;
        staged.as_file().sync_all().map_err(|error| error.to_string())?;
        let staged_path = staged.path();
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
    fn reports_require_verified_native_delivery() {
        let document = DesktopSaveDocument {
            kind: DesktopSaveKind::HtmlReport,
            filename: "report.html".into(),
            bytes_base64: BASE64.encode("<html></html>"),
        };
        let error = validate_save_document("7", &document).unwrap_err();
        assert!(error.contains("desktop_artifact_deliver"));
    }

    #[test]
    fn remote_security_documents_are_bounded_and_redacted() {
        let mut document = DesktopSaveDocument {
            kind: DesktopSaveKind::RemoteKey,
            filename: "dokime-remote".into(),
            bytes_base64: BASE64.encode("a".repeat(64)),
        };
        let (filename, bytes, _) = validate_save_document("7", &document).unwrap();
        assert_eq!(filename, "dokime-remote.key");
        assert_eq!(bytes.len(), 64);
        assert!(!format!("{document:?}").contains(&document.bytes_base64));
        document.bytes_base64 = BASE64.encode("short");
        assert!(validate_save_document("7", &document).is_err());
        document.bytes_base64 = BASE64.encode("a".repeat(4099));
        assert!(validate_save_document("7", &document).is_err());
        document.kind = DesktopSaveKind::PublicCertificate;
        document.filename = "dokime-server".into();
        document.bytes_base64 = BASE64.encode("-----BEGIN CERTIFICATE-----\npublic\n-----END CERTIFICATE-----");
        assert_eq!(validate_save_document("7", &document).unwrap().0, "dokime-server.pem");
        document.bytes_base64 = BASE64.encode("-----BEGIN CERTIFICATE-----\n-----BEGIN PRIVATE KEY-----");
        assert!(validate_save_document("7", &document).is_err());
        document.bytes_base64 = "A".repeat((64usize * 1024).div_ceil(3) * 4 + 4);
        assert!(validate_save_document("7", &document).is_err());
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

        assert!(error.contains("JSON document"), "{error}");
    }
}
