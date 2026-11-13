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
use super::desktop::{ArtifactSource, ensure_generation_token, ensure_main_window, ensure_private_directory, set_private_file};
use crate::connection::ConnectionSession;
use crate::rpc::protocol::FRAME_BYTES as DESKTOP_FRAME_BYTES;

/* === Constants === */

const DESKTOP_MEDIA_CACHE_BYTES: u64 = 4_294_967_296;
const DESKTOP_MEDIA_RANGE_BYTES: u64 = 4_194_304;

// Even An Incompressible RGBA PNG Fits Within One Native Range Response.
const IMAGE_PREVIEW_EDGE: u32 = 960;
const IMAGE_DECODE_BYTES: u64 = 128 * 1024 * 1024;
const MEDIA_CACHE_HEADER: &str = "preview-v1\n";
pub const MEDIA_SCHEME: &str = "dokime-media";

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
        move |context: tauri::UriSchemeContext<'_, tauri::Wry>, request: http::Request<Vec<u8>>, responder: tauri::UriSchemeResponder| {
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
    window: WebviewWindow, state: State<'_, ConnectionSession>, app: AppHandle, generation: String, source: ArtifactSource,
) -> Result<String, String> {
    ensure_main_window(&window)?;
    ensure_generation_token(&generation)?;
    let lease = state.inner().lease(&generation).map_err(|error| format!("Native media preview lost its owning connection: {error:?}"))?;
    let key = cache_key(&source)?;
    let media_root = media_root(&app);
    let cache_dir = media_root.join(&generation);
    let cache_dir_for_dir = cache_dir.clone();
    tauri::async_runtime::spawn_blocking(move || ensure_private_directory(&cache_dir_for_dir))
        .await
        .map_err(|error| error.to_string())??;
    let destination = cache_dir.join(&key);
    let prepared =
        tokio::fs::read_to_string(destination.with_extension("type")).await.ok().is_some_and(|value| cached_content_type(&value).is_some());
    if !prepared || !tokio::fs::try_exists(&destination).await.unwrap_or(false) {
        // Old Cache Entries Contain Originals, Which Must Never Reach The Image Element.
        let _ = tokio::fs::remove_file(&destination).await;
        let _ = tokio::fs::remove_file(destination.with_extension("type")).await;
        let verified = content::fetch_verified(&lease, &source, &destination, DESKTOP_FRAME_BYTES as usize)
            .await
            .map_err(|error| error.to_string())?;
        let content_type = verified.content_type;
        // Sniff Images Even When An Execution File Has No Declared Media Type.
        let path = destination.clone();
        let result = tauri::async_runtime::spawn_blocking(move || prepare_image(&path)).await.map_err(|error| error.to_string())?;
        let image_type = match result {
            Ok(image_type) => image_type,
            Err(error) => {
                let _ = tokio::fs::remove_file(&destination).await;
                return Err(error);
            }
        };
        write_content_type(&destination, if image_type { Some("image/png") } else { content_type.as_deref() }).await?;
        let private_path = destination.clone();
        tauri::async_runtime::spawn_blocking(move || set_private_file(&private_path)).await.map_err(|error| error.to_string())??;
    }
    evict_lru(&cache_dir, DESKTOP_MEDIA_CACHE_BYTES).await?;
    Ok(format!("{generation}/{key}"))
}

pub async fn desktop_media_release(_state: State<'_, ConnectionSession>, app: AppHandle, generation: String) -> Result<(), String> {
    ensure_generation_token(&generation)?;
    release_with(&media_root(&app), &generation, tokio::fs::remove_dir_all).await
}

pub(crate) async fn release_all(app: &AppHandle) {
    let media_root = media_root(app);
    let _ = tokio::fs::remove_dir_all(&media_root).await;
}

/* === Private Functions === */

fn media_root(app: &AppHandle) -> PathBuf {
    app.state::<crate::paths::ApplicationPaths>().cache().join("media")
}

async fn release_with<F>(media_root: &Path, generation: &str, remove: impl FnOnce(PathBuf) -> F) -> Result<(), String>
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
        ArtifactSource::ExecutionEvidence { .. } | ArtifactSource::CycleEvidence { .. } => {
            Err("evidence bundles must be saved, not previewed as media".into())
        }
        ArtifactSource::CycleReport { .. } | ArtifactSource::ExecutionReport { .. } => {
            Err("reports are saved or opened, not previewed as media".into())
        }
        ArtifactSource::Artifact { artifact_id } => Ok(format!("art-{}", cache_identifier(artifact_id)?)),
        ArtifactSource::ExecutionFile { execution_id, relative_path } => {
            let id = cache_identifier(execution_id)?;
            let digest = Sha256::digest(relative_path.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            Ok(format!("exec-{id}-{digest}"))
        }
    }
}

// Host Identifiers Are Opaque. Only Digests Become Local Cache Path Segments.
fn cache_identifier(identifier: &str) -> Result<String, String> {
    if identifier.is_empty() {
        return Err("Prepare Media: The Host Content Identifier Is Empty; Refresh Its Inventory".into());
    }
    Ok(Sha256::digest(identifier.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect())
}

fn parse_cache_key(key: &str) -> Result<String, String> {
    let digest = |value: &str| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit());
    if let Some(id) = key.strip_prefix("art-")
        && digest(id)
    {
        return Ok(format!("art-{}", id.to_ascii_lowercase()));
    }
    if let Some(rest) = key.strip_prefix("exec-")
        && let Some((id, content)) = rest.split_once('-')
        && digest(id)
        && digest(content)
    {
        return Ok(format!("exec-{}-{}", id.to_ascii_lowercase(), content.to_ascii_lowercase()));
    }
    Err("Prepare Media: Invalid Cache Key; Reload Its Preview".into())
}

fn parse_request_path(path: &str) -> Option<(String, String)> {
    // Tauri Encodes The Entire File Path, Including Its Separating Slash.
    let decoded = percent_encoding::percent_decode_str(path).decode_utf8().ok()?;
    let trimmed = decoded.strip_prefix('/')?;
    let mut segments = trimmed.split('/');
    let generation = segments.next().filter(|value| !value.is_empty())?;
    let key = segments.next().filter(|value| !value.is_empty())?;
    if segments.next().is_some() {
        return None;
    }
    Some((generation.to_owned(), key.to_owned()))
}

fn prepare_image(path: &Path) -> Result<bool, String> {
    let mut reader =
        image::ImageReader::open(path).map_err(|error| error.to_string())?.with_guessed_format().map_err(|error| error.to_string())?;
    if reader.format().is_none() {
        return Ok(false);
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(32_768);
    limits.max_image_height = Some(32_768);
    limits.max_alloc = Some(IMAGE_DECODE_BYTES);
    reader.limits(limits);
    let image = reader.decode().map_err(|error| error.to_string())?;
    let preview = image.thumbnail(IMAGE_PREVIEW_EDGE, IMAGE_PREVIEW_EDGE);
    let mut output =
        tempfile::NamedTempFile::new_in(path.parent().ok_or("media cache has no parent")?).map_err(|error| error.to_string())?;
    preview.write_to(output.as_file_mut(), image::ImageFormat::Png).map_err(|error| error.to_string())?;
    output.persist(path).map_err(|error| error.to_string())?;
    Ok(true)
}

async fn write_content_type(destination: &Path, content_type: Option<&str>) -> Result<(), String> {
    let value = content_type.map(str::trim).filter(|value| !value.is_empty()).unwrap_or("application/octet-stream");
    tokio::fs::write(destination.with_extension("type"), format!("{MEDIA_CACHE_HEADER}{value}")).await.map_err(|error| error.to_string())
}

fn cached_content_type(value: &str) -> Option<&str> {
    value.strip_prefix(MEDIA_CACHE_HEADER).map(str::trim).filter(|value| !value.is_empty())
}

async fn evict_lru(cache_dir: &Path, max_bytes: u64) -> Result<(), String> {
    let Ok(mut reader) = tokio::fs::read_dir(cache_dir).await else {
        return Ok(());
    };
    let mut entries = Vec::new();
    while let Some(entry) = reader.next_entry().await.map_err(|error| error.to_string())? {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "type") {
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

async fn evict<T: Ord, F>(mut entries: Vec<(PathBuf, T, u64)>, max_bytes: u64, mut remove: impl FnMut(PathBuf) -> F)
where
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

fn resolve_range(range_header: Option<&str>, len: u64, maximum: u64) -> RangeOutcome {
    let Some(value) = range_header else {
        return if len <= maximum {
            RangeOutcome::Full
        } else {
            RangeOutcome::Partial {
                start: 0,
                end: maximum.saturating_sub(1).min(len.saturating_sub(1)),
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
    let max_end = start.saturating_add(maximum - 1);
    RangeOutcome::Partial {
        start,
        end: requested_end.min(len - 1).min(max_end),
    }
}

async fn read_range(path: &Path, start: u64, end: u64) -> std::io::Result<Vec<u8>> {
    let mut file = tokio::fs::File::open(path).await?;
    if file.metadata().await?.len() == 0 {
        return Ok(Vec::new());
    }
    file.seek(std::io::SeekFrom::Start(start)).await?;
    let mut buffer = vec![0_u8; (end - start + 1) as usize];
    file.read_exact(&mut buffer).await?;
    Ok(buffer)
}

fn status_only(status: http::StatusCode) -> http::Response<Vec<u8>> {
    http::Response::builder().status(status).body(Vec::new()).unwrap_or_else(|_| {
        let mut response = http::Response::new(Vec::new());
        *response.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
        response
    })
}

async fn try_respond(app: &AppHandle, request: &http::Request<Vec<u8>>) -> Result<http::Response<Vec<u8>>, http::StatusCode> {
    let (generation, raw_key) = parse_request_path(request.uri().path()).ok_or(http::StatusCode::BAD_REQUEST)?;
    if generation.is_empty() || !generation.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(http::StatusCode::BAD_REQUEST);
    }
    let key = parse_cache_key(&raw_key).map_err(|_| http::StatusCode::BAD_REQUEST)?;
    // A stale generation (disconnected, superseded) cannot read a previous
    // connection's cache, even one still on disk pending eviction.
    app.state::<ConnectionSession>().lease(&generation).map_err(|_| http::StatusCode::NOT_FOUND)?;
    let path = media_root(app).join(&generation).join(&key);
    let metadata = tokio::fs::metadata(&path).await.map_err(|_| http::StatusCode::NOT_FOUND)?;
    let len = metadata.len();
    let content_type = tokio::fs::read_to_string(path.with_extension("type"))
        .await
        .ok()
        .and_then(|value| cached_content_type(&value).map(str::to_owned))
        .ok_or(http::StatusCode::NOT_FOUND)?;
    let range_header = request.headers().get(http::header::RANGE).and_then(|value| value.to_str().ok());
    match resolve_range(range_header, len, DESKTOP_MEDIA_RANGE_BYTES) {
        RangeOutcome::Unsatisfiable => http::Response::builder()
            .status(http::StatusCode::RANGE_NOT_SATISFIABLE)
            .header(http::header::CONTENT_RANGE, format!("bytes */{len}"))
            .body(Vec::new())
            .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR),
        RangeOutcome::Full => {
            let bytes = read_range(&path, 0, len.saturating_sub(1)).await.map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;
            http::Response::builder()
                .status(http::StatusCode::OK)
                .header(http::header::CONTENT_TYPE, content_type)
                .header(http::header::ACCEPT_RANGES, "bytes")
                .header(http::header::CONTENT_LENGTH, bytes.len())
                .body(bytes)
                .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)
        }
        RangeOutcome::Partial { start, end } => {
            let bytes = read_range(&path, start, end).await.map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;
            http::Response::builder()
                .status(http::StatusCode::PARTIAL_CONTENT)
                .header(http::header::CONTENT_TYPE, content_type)
                .header(http::header::ACCEPT_RANGES, "bytes")
                .header(http::header::CONTENT_RANGE, format!("bytes {start}-{end}/{len}"))
                .header(http::header::CONTENT_LENGTH, bytes.len())
                .body(bytes)
                .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(feature = "testing")]
pub fn prepare_image_preview(path: &Path) -> Result<bool, String> {
    prepare_image(path)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tauri_encoded_media_paths() {
        let key = cache_key(&ArtifactSource::Artifact {
            artifact_id: "opaque/host/content".into(),
        })
        .unwrap();
        assert_ne!(key, cache_key(&ArtifactSource::Artifact { artifact_id: "other-content".into() }).unwrap());
        let execution = cache_key(&ArtifactSource::ExecutionFile {
            execution_id: "imported-execution".into(),
            relative_path: "images/front.png".into(),
        })
        .unwrap();
        assert_eq!(parse_cache_key(&execution).unwrap(), execution);
        assert!(cache_key(&ArtifactSource::Artifact { artifact_id: String::new() }).is_err());
        for path in [format!("/12%2F{key}"), format!("/12/{key}")] {
            let (generation, parsed) = parse_request_path(&path).unwrap();
            assert_eq!(generation, "12");
            assert_eq!(parse_cache_key(&parsed).unwrap(), key);
        }
        assert!(parse_request_path("/12%2Fkey%2Fextra").is_none());
        assert!(parse_cache_key(&parse_request_path("/12%2F..").unwrap().1).is_err());
        assert_eq!(cached_content_type("image/png"), None);
        assert_eq!(cached_content_type("preview-v1\nimage/png"), Some("image/png"));
    }
}
