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
use regex::Regex;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::{runtime::Error, runtime::Result, execution_runtime::authoring::FileRegexMatch};
use dokime_domain::model::time::Timestamp;

/* === Constants === */
const REGEX_SCAN_LIMIT_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MAX_ARTIFACT_READ_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_EXECUTION_FILE_SNAPSHOT_BYTES: usize = 2 * 1024 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionFileEntry {
    pub artifact_id: String,
    pub relative_path: String,
    pub content_type: Option<String>,
    pub content_kind: ExecutionFileContentKind,
    pub stream_state: ExecutionFileStreamState,
    pub size_bytes: u64,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionFileContent {
    pub relative_path: String,
    pub content_type: Option<String>,
    pub content_kind: ExecutionFileContentKind,
    pub stream_state: ExecutionFileStreamState,
    pub offset: u64,
    pub size_bytes: u64,
    pub bytes: Vec<u8>,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionFileContentKind {
    Text,
    Binary,
    Media,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionFileStreamState {
    Streaming,
    Finalized,
    Interrupted,
}

/* === Traits === */

/* === Implementations === */
impl ExecutionFileContentKind {
    pub fn from_metadata(path: &str, content_type: Option<&str>) -> Self {
        if let Some(content_type) = content_type {
            if content_type.starts_with("text/")
                || matches!(
                    content_type,
                    "application/json" | "application/x-ndjson" | "application/xml"
                )
            {
                return ExecutionFileContentKind::Text;
            }
            if content_type.starts_with("image/")
                || content_type.starts_with("video/")
                || content_type.starts_with("audio/")
            {
                return ExecutionFileContentKind::Media;
            }
            return ExecutionFileContentKind::Binary;
        }
        match extension(path) {
            "txt" | "log" | "md" | "csv" | "json" | "jsonl" | "toml" | "yaml" | "yml" | "xml" => {
                ExecutionFileContentKind::Text
            }
            "png" | "jpg" | "jpeg" | "webp" | "mp4" | "wav" => ExecutionFileContentKind::Media,
            "arrow" | "zip" => ExecutionFileContentKind::Binary,
            _ => ExecutionFileContentKind::Unknown,
        }
    }
}

impl harmos::MemorySize for ExecutionFileEntry {
    fn memory_size(&self) -> usize {
        let timestamp = harmos::MemorySize::memory_size(&self.updated_at);
        if timestamp == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(self.artifact_id.capacity())
            .saturating_add(self.relative_path.capacity())
            .saturating_add(self.content_type.as_ref().map_or(0, String::capacity))
            .saturating_add(timestamp.saturating_sub(std::mem::size_of::<Timestamp>()))
    }
}

impl harmos::MemorySize for ExecutionFileContent {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.relative_path.capacity())
            .saturating_add(self.content_type.as_ref().map_or(0, String::capacity))
            .saturating_add(self.bytes.capacity())
    }
}

/* === Public Functions === */
pub(crate) fn regex_matches(
    content: &ExecutionFileContent,
    pattern: &str,
    first_only: bool,
) -> Result<Vec<FileRegexMatch>> {
    if content.offset != 0 || content.bytes.len() > REGEX_SCAN_LIMIT_BYTES {
        return Err(Error::Operation(
            "execution output regex reads must start at zero and stay within 2 MiB".to_owned(),
        ));
    }
    let regex = Regex::new(pattern)
        .map_err(|error| Error::Operation(format!("invalid regex pattern: {error}")))?;
    let text = std::str::from_utf8(&content.bytes).map_err(|error| {
        Error::Operation(format!(
            "execution output {} is not valid UTF-8: {error}",
            content.relative_path
        ))
    })?;
    let mut matches = Vec::new();
    for captures in regex.captures_iter(text) {
        let Some(full_match) = captures.get(0) else {
            continue;
        };
        matches.push(FileRegexMatch {
            full_match: full_match.as_str().to_owned(),
            captures: (1..captures.len())
                .map(|index| {
                    captures
                        .get(index)
                        .map(|capture| capture.as_str().to_owned())
                })
                .collect(),
            named_captures: regex
                .capture_names()
                .flatten()
                .filter_map(|name| {
                    captures
                        .name(name)
                        .map(|capture| (name.to_owned(), capture.as_str().to_owned()))
                })
                .collect(),
            byte_offset: full_match.start() as u64,
            line_number: Some(
                text[..full_match.start()]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count() as u64
                    + 1,
            ),
        });
        if first_only {
            break;
        }
    }
    Ok(matches)
}

pub(crate) fn validate_producer_path(relative_path: &str) -> Result<()> {
    crate::execution::saved::validate_output_path(relative_path)
        .map_err(|error| Error::Operation(error.to_string()))
}

pub(crate) fn validate_relative_path(relative_path: &str) -> Result<()> {
    crate::execution::saved::validate_relative_output_path(relative_path)
        .map_err(|error| Error::Operation(error.to_string()))
}

pub(crate) fn media_type_for_path(path: &str) -> Option<&'static str> {
    match extension(path) {
        "html" | "htm" => Some("text/html"),
        "json" => Some("application/json"),
        "jsonl" => Some("application/x-ndjson"),
        "txt" | "log" | "md" => Some("text/plain"),
        "csv" => Some("text/csv"),
        "arrow" => Some("application/vnd.apache.arrow.file"),
        "zip" => Some("application/zip"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "mp4" => Some("video/mp4"),
        "wav" => Some("audio/wav"),
        _ => None,
    }
}

/* === Private Functions === */
fn extension(path: &str) -> &str {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
}

/* === Tests === */
