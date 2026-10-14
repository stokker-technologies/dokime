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
use std::path::{Path, PathBuf};
// Third Party Imports
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State, WebviewWindow, http};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
// Local Imports
use super::content;
use super::desktop::{
    ArtifactSource, ensure_generation_token, ensure_main_window, ensure_private_directory,
    set_private_file,
};
use crate::connection::ConnectionSession;

/* === Constants === */
pub const MEDIA_SCHEME: &str = "dokime-media";
const MAX_MEDIA_CACHE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MEDIA_RANGE_BYTES: u64 = 4 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
enum RangeOutcome {
    Full,
    Partial { start: u64, end: u64 },
    Unsatisfiable,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn register(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.register_asynchronous_uri_scheme_protocol(
        MEDIA_SCHEME,
        move |context: tauri::UriSchemeContext<'_, tauri::Wry>,
              request: http::Request<Vec<u8>>,
              responder: tauri::UriSchemeResponder| {
            let app = context.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let response = match try_respond(&app, &request).await {
                    Ok(response) => response,
                    Err(status) => status_only(status),
                };
                responder.respond(response);
            });
        },
    )
}

pub async fn desktop_media_prepare(
    window: WebviewWindow,
    state: State<'_, ConnectionSession>,
    app: AppHandle,
    generation: String,
    source: ArtifactSource,
) -> Result<String, String> {
    ensure_main_window(&window)?;
    ensure_generation_token(&generation)?;
    let lease = state
        .inner()
        .lease(&generation)
        .map_err(|error| format!("Native media preview lost its owning connection: {error:?}"))?;
    let key = cache_key(&source)?;
    let media_root = media_root(&app)?;
    let cache_dir = media_root.join(&generation);
    let cache_dir_for_dir = cache_dir.clone();
    tauri::async_runtime::spawn_blocking(move || ensure_private_directory(&cache_dir_for_dir))
        .await
        .map_err(|error| error.to_string())??;
    let destination = cache_dir.join(&key);
    if !tokio::fs::try_exists(&destination).await.unwrap_or(false) {
        let verified = content::fetch_verified(&lease, &source, &destination)
            .await
            .map_err(|error| error.to_string())?;
        write_content_type(&destination, verified.content_type.as_deref()).await?;
        let private_path = destination.clone();
        tauri::async_runtime::spawn_blocking(move || set_private_file(&private_path))
            .await
            .map_err(|error| error.to_string())??;
    }
    evict_lru(&cache_dir, MAX_MEDIA_CACHE_BYTES).await?;
    Ok(format!("{generation}/{key}"))
}

pub async fn desktop_media_release(
    _state: State<'_, ConnectionSession>,
    app: AppHandle,
    generation: String,
) -> Result<(), String> {
    ensure_generation_token(&generation)?;
    release_generation(&media_root(&app)?, &generation).await
}

pub(crate) async fn release_all(app: &AppHandle) {
    if let Ok(media_root) = media_root(app) {
        let _ = tokio::fs::remove_dir_all(&media_root).await;
    }
}

/* === Private Functions === */
fn media_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("media"))
}

async fn release_generation(media_root: &Path, generation: &str) -> Result<(), String> {
    release_with(media_root, generation, |path| {
        tokio::fs::remove_dir_all(path)
    })
    .await
}

async fn release_with<F>(
    media_root: &Path,
    generation: &str,
    remove: impl FnOnce(PathBuf) -> F,
) -> Result<(), String>
where
    F: std::future::Future<Output = std::io::Result<()>>,
{
    match remove(media_root.join(generation)).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn cache_key(source: &ArtifactSource) -> Result<String, String> {
    match source {
        ArtifactSource::Artifact { artifact_id } => {
            let id = dokime::core::ArtifactId::from_uuid(artifact_id.clone())
                .map_err(|_| "invalid artifact id".to_string())?;
            Ok(format!("art-{id}"))
        }
        ArtifactSource::ExecutionFile {
            execution_id,
            relative_path,
        } => {
            let id = dokime::core::TestExecutionId::from_uuid(execution_id.clone())
                .map_err(|_| "invalid execution id".to_string())?;
            let digest = Sha256::digest(relative_path.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            Ok(format!("exec-{id}-{digest}"))
        }
    }
}

fn parse_cache_key(key: &str) -> Result<String, String> {
    if let Some(id) = key.strip_prefix("art-") {
        let id = dokime::core::ArtifactId::from_uuid(id.to_string())
            .map_err(|_| "invalid artifact id".to_string())?;
        return Ok(format!("art-{id}"));
    }
    if let Some(rest) = key.strip_prefix("exec-") {
        let (id, digest) = rest
            .rsplit_once('-')
            .ok_or_else(|| "invalid media key".to_string())?;
        let id = dokime::core::TestExecutionId::from_uuid(id.to_string())
            .map_err(|_| "invalid execution id".to_string())?;
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("invalid media key digest".into());
        }
        return Ok(format!("exec-{id}-{}", digest.to_ascii_lowercase()));
    }
    Err("invalid media key".into())
}

fn parse_request_path(path: &str) -> Option<(&str, &str)> {
    let trimmed = path.strip_prefix('/')?;
    let mut segments = trimmed.split('/');
    let generation = segments.next().filter(|value| !value.is_empty())?;
    let key = segments.next().filter(|value| !value.is_empty())?;
    if segments.next().is_some() {
        return None;
    }
    Some((generation, key))
}

async fn write_content_type(destination: &Path, content_type: Option<&str>) -> Result<(), String> {
    let value = content_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("application/octet-stream");
    tokio::fs::write(destination.with_extension("type"), value)
        .await
        .map_err(|error| error.to_string())
}

async fn evict_lru(cache_dir: &Path, max_bytes: u64) -> Result<(), String> {
    let Ok(mut reader) = tokio::fs::read_dir(cache_dir).await else {
        return Ok(());
    };
    let mut entries = Vec::new();
    while let Some(entry) = reader
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "type")
        {
            continue;
        }
        let metadata = entry.metadata().await.map_err(|error| error.to_string())?;
        if !metadata.is_file() {
            continue;
        }
        let modified = metadata.modified().map_err(|error| error.to_string())?;
        entries.push((path, modified, metadata.len()));
    }
    evict(entries, max_bytes, tokio::fs::remove_file).await;
    Ok(())
}

async fn evict<T: Ord, F>(
    mut entries: Vec<(PathBuf, T, u64)>,
    max_bytes: u64,
    mut remove: impl FnMut(PathBuf) -> F,
) where
    F: std::future::Future<Output = std::io::Result<()>>,
{
    let mut total = entries.iter().map(|(_, _, size)| size).sum::<u64>();
    if total <= max_bytes {
        return;
    }
    entries.sort_by(|left, right| left.1.cmp(&right.1));
    for (path, _, size) in entries {
        if total <= max_bytes {
            break;
        }
        if remove(path.clone()).await.is_ok() {
            total = total.saturating_sub(size);
            let _ = remove(path.with_extension("type")).await;
        }
    }
}

fn resolve_range(range_header: Option<&str>, len: u64) -> RangeOutcome {
    let Some(value) = range_header else {
        return if len <= MEDIA_RANGE_BYTES {
            RangeOutcome::Full
        } else {
            RangeOutcome::Partial {
                start: 0,
                end: MEDIA_RANGE_BYTES
                    .saturating_sub(1)
                    .min(len.saturating_sub(1)),
            }
        };
    };
    let Some(spec) = value.strip_prefix("bytes=") else {
        return RangeOutcome::Unsatisfiable;
    };
    let Some((start_text, end_text)) = spec.split_once('-') else {
        return RangeOutcome::Unsatisfiable;
    };
    let Ok(start) = start_text.parse::<u64>() else {
        return RangeOutcome::Unsatisfiable;
    };
    if start >= len {
        return RangeOutcome::Unsatisfiable;
    }
    let requested_end = if end_text.is_empty() {
        len - 1
    } else {
        match end_text.parse::<u64>() {
            Ok(value) => value,
            Err(_) => return RangeOutcome::Unsatisfiable,
        }
    };
    let max_end = start.saturating_add(MEDIA_RANGE_BYTES - 1);
    RangeOutcome::Partial {
        start,
        end: requested_end.min(len - 1).min(max_end),
    }
}

async fn read_range(path: &Path, start: u64, end: u64) -> std::io::Result<Vec<u8>> {
    let mut file = tokio::fs::File::open(path).await?;
    file.seek(std::io::SeekFrom::Start(start)).await?;
    let mut buffer = vec![0_u8; (end - start + 1) as usize];
    file.read_exact(&mut buffer).await?;
    Ok(buffer)
}

fn status_only(status: http::StatusCode) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .body(Vec::new())
        .unwrap_or_else(|_| {
            let mut response = http::Response::new(Vec::new());
            *response.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
            response
        })
}

async fn try_respond(
    app: &AppHandle,
    request: &http::Request<Vec<u8>>,
) -> Result<http::Response<Vec<u8>>, http::StatusCode> {
    let (generation, raw_key) =
        parse_request_path(request.uri().path()).ok_or(http::StatusCode::BAD_REQUEST)?;
    if generation.is_empty() || !generation.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(http::StatusCode::BAD_REQUEST);
    }
    let key = parse_cache_key(raw_key).map_err(|_| http::StatusCode::BAD_REQUEST)?;
    // A stale generation (disconnected, superseded) cannot read a previous
    // connection's cache, even one still on disk pending eviction.
    app.state::<ConnectionSession>()
        .lease(generation)
        .map_err(|_| http::StatusCode::NOT_FOUND)?;
    let path = media_root(app)
        .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?
        .join(generation)
        .join(&key);
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|_| http::StatusCode::NOT_FOUND)?;
    let len = metadata.len();
    let content_type = tokio::fs::read_to_string(path.with_extension("type"))
        .await
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "application/octet-stream".into());
    let range_header = request
        .headers()
        .get(http::header::RANGE)
        .and_then(|value| value.to_str().ok());
    match resolve_range(range_header, len) {
        RangeOutcome::Unsatisfiable => http::Response::builder()
            .status(http::StatusCode::RANGE_NOT_SATISFIABLE)
            .header(http::header::CONTENT_RANGE, format!("bytes */{len}"))
            .body(Vec::new())
            .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR),
        RangeOutcome::Full => {
            let bytes = read_range(&path, 0, len.saturating_sub(1))
                .await
                .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;
            http::Response::builder()
                .status(http::StatusCode::OK)
                .header(http::header::CONTENT_TYPE, content_type)
                .header(http::header::ACCEPT_RANGES, "bytes")
                .header(http::header::CONTENT_LENGTH, bytes.len())
                .body(bytes)
                .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)
        }
        RangeOutcome::Partial { start, end } => {
            let bytes = read_range(&path, start, end)
                .await
                .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;
            http::Response::builder()
                .status(http::StatusCode::PARTIAL_CONTENT)
                .header(http::header::CONTENT_TYPE, content_type)
                .header(http::header::ACCEPT_RANGES, "bytes")
                .header(
                    http::header::CONTENT_RANGE,
                    format!("bytes {start}-{end}/{len}"),
                )
                .header(http::header::CONTENT_LENGTH, bytes.len())
                .body(bytes)
                .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/* === Tests === */
