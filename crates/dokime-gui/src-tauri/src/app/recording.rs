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

// Third Party Imports
use tauri::WebviewWindow;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

// Local Imports
use super::content;
use super::desktop::ensure_main_window;

/* === Constants === */

const DESKTOP_MP4_PROBE_BYTES: u64 = 4_194_304;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// === Private Functions: MP4 (ISO BMFF) box walking === //

//

// A byte-for-byte port of the box reader/writer pair in

// `fix-recording-duration.ts`; offsets are the ISO/IEC 14496-12 layouts for

// `mvhd`/`tkhd`/`mdhd` version 0 (32-bit) and version 1 (64-bit) fields.

struct Mp4Box {
    box_type: [u8; 4],
    end: usize,
    payload: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn desktop_recording_finish(
    window: WebviewWindow, bridge: tauri::State<'_, crate::rpc::Bridge>, generation: String, duration_ms: u64, media_type: String,
) -> Result<String, String> {
    ensure_main_window(&window)?;
    let generation_value: u64 = generation.parse().map_err(|_| "Native recording generation is invalid".to_string())?;
    let recording = bridge
        .take_recording(window.label(), generation_value)
        .await
        .map_err(|error| format!("Native recording is no longer available: {error:?}"))?;
    if is_mp4(&media_type) {
        patch_mp4_duration_in_place(&recording.path, duration_ms, DESKTOP_MP4_PROBE_BYTES).await?;
    }
    let uploaded = content::push_verified(&recording.lease, &recording.path, recording.header, DESKTOP_FRAME_BYTES as usize)
        .await
        .inspect_err(|_| {
            let path = recording.path.clone();
            tauri::async_runtime::spawn(async move {
                let _ = tokio::fs::remove_file(&path).await;
            });
        })?;
    let _ = tokio::fs::remove_file(&recording.path).await;
    Ok(uploaded.artifact_id)
}

pub(crate) async fn patch_mp4_duration_in_place(path: &std::path::Path, duration_ms: u64, probe_bytes: u64) -> Result<bool, String> {
    if duration_ms == 0 {
        return Ok(false);
    }
    let file = tokio::fs::OpenOptions::new().read(true).write(true).open(path).await.map_err(|error| error.to_string())?;
    let file_len = file.metadata().await.map_err(|error| error.to_string())?.len();
    patch_duration(file, file_len, duration_ms, probe_bytes).await
}

/* === Private Functions === */

fn is_mp4(media_type: &str) -> bool {
    let lower = media_type.to_ascii_lowercase();
    lower.starts_with("video/mp4") || lower.starts_with("audio/mp4")
}

async fn patch_duration(
    mut file: impl tokio::io::AsyncRead + tokio::io::AsyncWrite + tokio::io::AsyncSeek + Unpin, file_len: u64, duration_ms: u64,
    probe_bytes: u64,
) -> Result<bool, String> {
    let probe_len = probe_bytes.min(file_len) as usize;
    let mut buffer = vec![0_u8; probe_len];
    file.read_exact(&mut buffer).await.map_err(|error| error.to_string())?;
    if !fix_mp4_duration(&mut buffer, duration_ms) {
        return Ok(false);
    }
    file.seek(std::io::SeekFrom::Start(0)).await.map_err(|error| error.to_string())?;
    file.write_all(&buffer).await.map_err(|error| error.to_string())?;
    file.flush().await.map_err(|error| error.to_string())?;
    Ok(true)
}

fn read_box(data: &[u8], offset: usize) -> Option<Mp4Box> {
    if offset + 8 > data.len() {
        return None;
    }
    let mut size = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as u64;
    let box_type: [u8; 4] = data[offset + 4..offset + 8].try_into().unwrap();
    let mut payload = offset + 8;
    if size == 1 {
        if offset + 16 > data.len() {
            return None;
        }
        size = u64::from_be_bytes(data[offset + 8..offset + 16].try_into().unwrap());
        payload = offset + 16;
    } else if size == 0 {
        size = (data.len() - offset) as u64;
    }
    let end = offset as u64 + size;
    if end > data.len() as u64 || end <= offset as u64 {
        return None;
    }
    Some(Mp4Box { box_type, end: end as usize, payload })
}

fn find_box(data: &[u8], start: usize, end: usize, target: &[u8; 4]) -> Option<Mp4Box> {
    let mut pos = start;
    while pos < end {
        let candidate = read_box(data, pos)?;
        if &candidate.box_type == target {
            return Some(candidate);
        }
        pos = candidate.end;
    }
    None
}

fn fix_mp4_duration(data: &mut [u8], duration_ms: u64) -> bool {
    let Some(moov) = find_box(data, 0, data.len(), b"moov") else {
        return false;
    };
    let Some(mvhd) = find_box(data, moov.payload, moov.end, b"mvhd") else {
        return false;
    };
    let Some(movie_timescale) = patch_mvhd_duration(data, mvhd.payload, duration_ms) else {
        return false;
    };
    let mut pos = moov.payload;
    while pos < moov.end {
        let Some(candidate) = read_box(data, pos) else {
            break;
        };
        if &candidate.box_type == b"trak" {
            patch_trak_duration(data, candidate.payload, candidate.end, movie_timescale, duration_ms);
        }
        pos = candidate.end;
    }
    true
}

fn duration_ticks_u32(duration_ms: u64, timescale: u32) -> u32 {
    ((duration_ms as f64 / 1000.0) * timescale as f64).round().min(u32::MAX as f64) as u32
}

fn duration_ticks_u64(duration_ms: u64, timescale: u32) -> u64 {
    ((duration_ms as f64 / 1000.0) * timescale as f64).round() as u64
}

fn patch_mvhd_duration(data: &mut [u8], payload: usize, duration_ms: u64) -> Option<u32> {
    let version = data[payload];
    if version == 0 {
        let timescale = u32::from_be_bytes(data[payload + 12..payload + 16].try_into().unwrap());
        if timescale == 0 {
            return None;
        }
        let duration = duration_ticks_u32(duration_ms, timescale);
        data[payload + 16..payload + 20].copy_from_slice(&duration.to_be_bytes());
        Some(timescale)
    } else {
        let timescale = u32::from_be_bytes(data[payload + 20..payload + 24].try_into().unwrap());
        if timescale == 0 {
            return None;
        }
        let duration = duration_ticks_u64(duration_ms, timescale);
        data[payload + 24..payload + 32].copy_from_slice(&duration.to_be_bytes());
        Some(timescale)
    }
}

fn patch_trak_duration(data: &mut [u8], start: usize, end: usize, movie_timescale: u32, duration_ms: u64) {
    let mut pos = start;
    while pos < end {
        let Some(candidate) = read_box(data, pos) else {
            return;
        };
        if &candidate.box_type == b"tkhd" {
            patch_tkhd_duration(data, candidate.payload, movie_timescale, duration_ms);
        } else if &candidate.box_type == b"mdia" {
            patch_mdia_duration(data, candidate.payload, candidate.end, duration_ms);
        }
        pos = candidate.end;
    }
}

fn patch_tkhd_duration(data: &mut [u8], payload: usize, movie_timescale: u32, duration_ms: u64) {
    let version = data[payload];
    if version == 0 {
        let duration = duration_ticks_u32(duration_ms, movie_timescale);
        data[payload + 20..payload + 24].copy_from_slice(&duration.to_be_bytes());
    } else {
        let duration = duration_ticks_u64(duration_ms, movie_timescale);
        data[payload + 28..payload + 36].copy_from_slice(&duration.to_be_bytes());
    }
}

fn patch_mdia_duration(data: &mut [u8], start: usize, end: usize, duration_ms: u64) {
    let Some(mdhd) = find_box(data, start, end, b"mdhd") else {
        return;
    };
    let payload = mdhd.payload;
    let version = data[payload];
    if version == 0 {
        let timescale = u32::from_be_bytes(data[payload + 12..payload + 16].try_into().unwrap());
        if timescale == 0 {
            return;
        }
        let duration = duration_ticks_u32(duration_ms, timescale);
        data[payload + 16..payload + 20].copy_from_slice(&duration.to_be_bytes());
    } else {
        let timescale = u32::from_be_bytes(data[payload + 20..payload + 24].try_into().unwrap());
        if timescale == 0 {
            return;
        }
        let duration = duration_ticks_u64(duration_ms, timescale);
        data[payload + 24..payload + 32].copy_from_slice(&duration.to_be_bytes());
    }
}

/* === Tests === */
