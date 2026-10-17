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
use std::path::Path;
// Third Party Imports
use serde::Deserialize;
use serde::Serialize;
// Local Imports
use crate::model::execution::RecordScope;
use crate::model::identity::ArtifactId;
use crate::model::identity::UploadId;
use crate::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub artifact_id: ArtifactId,
    pub upload_id: UploadId,
    pub scope: RecordScope,
    pub filename: String,
    pub content_type: Option<String>,
    pub size_bytes: u64,
    pub sha256: Option<String>,
    pub status: ArtifactStatus,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactUploadInput {
    pub scope: RecordScope,
    pub filename: String,
    pub content_type: Option<String>,
    pub size_bytes: u64,
    pub sha256: Option<String>,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Image,
    Video,
    Audio,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    Staged,
    Complete,
    Cancelled,
    Rejected,
}

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for ArtifactRecord {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.artifact_id), std::mem::size_of_val(&self.artifact_id)),
            (harmos::MemorySize::memory_size(&self.upload_id), std::mem::size_of_val(&self.upload_id)),
            (harmos::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
            (harmos::MemorySize::memory_size(&self.filename), std::mem::size_of_val(&self.filename)),
            (harmos::MemorySize::memory_size(&self.content_type), std::mem::size_of_val(&self.content_type)),
            (harmos::MemorySize::memory_size(&self.sha256), std::mem::size_of_val(&self.sha256)),
            (harmos::MemorySize::memory_size(&self.created_at), std::mem::size_of_val(&self.created_at)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

/* === Public Functions === */
pub fn media_kind_for(filename: &str, content_type: Option<&str>) -> Option<MediaKind> {
    let lower = filename.to_ascii_lowercase();
    let extension = Path::new(&lower).extension().and_then(|extension| extension.to_str());

    if matches!(extension, Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "svg")) {
        return Some(MediaKind::Image);
    }
    if matches!(extension, Some("mp4" | "webm" | "mov" | "avi" | "mkv" | "m4v")) {
        return Some(MediaKind::Video);
    }
    if matches!(extension, Some("wav" | "mp3" | "ogg" | "oga" | "opus" | "flac" | "m4a" | "aac")) {
        return Some(MediaKind::Audio);
    }

    match content_type.map(str::trim) {
        Some(content_type) if content_type.starts_with("image/") => Some(MediaKind::Image),
        Some(content_type) if content_type.starts_with("video/") => Some(MediaKind::Video),
        Some(content_type) if content_type.starts_with("audio/") => Some(MediaKind::Audio),
        _ => None,
    }
}

/* === Private Functions === */

/* === Tests === */
