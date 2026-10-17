/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File sources watched during an execution: every matched file becomes one
// execution artifact under the watch's output folder. `FileWatch` streams its
// progress to a listener; `WatchFile` is the same watch over one file for a
// template, settling only when the watch stops, and may carry line matchers
// that turn captured lines into rows and records.

/* === Modules === */
mod file_watch;
mod watch_file;

/* === Re-Exports === */
pub use file_watch::FileWatch;
pub use watch_file::WatchFile;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::content::ArtifactMetadata;
use super::{Access, Endpoint, FOLLOW};
use crate::model::values::ValueType;
use crate::model::telemetry::TelemetryRow;
use crate::model::telemetry::TelemetryStream;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// One ordered include or exclude rule over paths relative to the source root.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileRule {
    // A glob over forward-slash relative paths; `*` stays within one segment, `**` crosses segments.
    #[row(
        description = "A glob over forward-slash relative paths; `*` stays within one segment, `**` crosses segments."
    )]
    pub glob: String,
    pub action: FileRuleAction,
}

// A directory on the backend host.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalWatchSource {
    // An absolute directory path on the backend host.
    #[row(description = "An absolute directory path on the backend host.")]
    pub root: String,
}

// A directory on an FTP server, listed and retrieved by polling.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FtpWatchSource {
    // `ftps://[user@]host[:port]/root` (explicit TLS), or `ftp://` on a trusted bench; never a password.
    #[row(
        description = "`ftps://[user@]host[:port]/root` (explicit TLS), or `ftp://` on a trusted bench; never a password."
    )]
    pub url: String,
    // The host credential store entry holding the password; absent logs in anonymously.
    #[row(
        description = "The host credential store entry holding the password; absent logs in anonymously."
    )]
    pub credential_id: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWatchSource {
    pub payload: Option<FileWatchSourcePayload>,
}

// Watches a source for the lifetime of the stream: until the caller cancels,
// the bound elapses or the execution seals. Each matched file is captured into
// `<output>/<path relative to the root>` and sealed with its length and digest.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWatchRequest {
    // Absent takes the execution of the calling step's context; one of the two is required.
    #[row(
        description = "Absent takes the execution of the calling step's context; one of the two is required."
    )]
    pub execution_id: Option<String>,
    pub source: Option<FileWatchSource>,
    // Ordered rules: the last rule matching a path decides; a path no rule matches is excluded.
    #[row(
        description = "Ordered rules: the last rule matching a path decides; a path no rule matches is excluded."
    )]
    pub rules: Vec<FileRule>,
    // The artifact folder, a relative path, that mirrors the matched structure.
    #[row(
        description = "The artifact folder, a relative path, that mirrors the matched structure."
    )]
    pub output: String,
    pub policy: FileWatchPolicy,
    // Discovery and capture interval: default 1000, 50–60000 milliseconds.
    #[row(description = "Discovery and capture interval: default 1000, 50–60000 milliseconds.")]
    pub poll_interval_ms: Option<u32>,
    // The watch seals and completes after this many seconds; absent runs until cancelled or sealed.
    #[row(
        description = "The watch seals and completes after this many seconds; absent runs until cancelled or sealed."
    )]
    pub maximum_seconds: Option<u32>,
    // Files the watch may track: default 64, maximum 256.
    #[row(description = "Files the watch may track: default 64, maximum 256.")]
    pub maximum_files: Option<u32>,
    // Bytes the watch may capture across its files: default 256 MiB, maximum 4 GiB.
    #[row(
        description = "Bytes the watch may capture across its files: default 256 MiB, maximum 4 GiB."
    )]
    pub maximum_bytes: Option<u64>,
}

// One captured named group read as a column of the matcher's stream.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LineField {
    // The column, and the named capture group that fills it.
    #[row(description = "The column, and the named capture group that fills it.")]
    pub name: String,
    pub kind: LineFieldKind,
}

// A regular expression over captured lines whose named groups publish one row
// to a stream the calling template declares.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LineMatcher {
    pub storage_schema: harmos::schema::graph::Schema,
    pub pattern: String,
    pub stream: String,
    pub fields: Vec<LineField>,
}

// A regular expression over captured lines that records one record of a type
// the calling template declares, its named groups as text details.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LineMarker {
    pub pattern: String,
    pub record_type: String,
}

// Watches one file for the calling step's execution until the instance is
// cancelled, the bound elapses or the execution seals. Its bytes become the
// artifact `artifact`, or `<output>/<file name>` when absent.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchFileRequest {
    // Absent takes the execution of the calling step's context; one of the two is required.
    #[row(
        description = "Absent takes the execution of the calling step's context; one of the two is required."
    )]
    pub execution_id: Option<String>,
    // A local root or FTP URL naming the watched file itself.
    #[row(description = "A local root or FTP URL naming the watched file itself.")]
    pub source: Option<FileWatchSource>,
    // The artifact folder, a relative path.
    #[row(description = "The artifact folder, a relative path.")]
    pub output: String,
    // The artifact name, a relative path; absent names it `<output>/<file name>`.
    #[row(
        description = "The artifact name, a relative path; absent names it `<output>/<file name>`."
    )]
    pub artifact: Option<String>,
    pub policy: FileWatchPolicy,
    // Discovery and capture interval: default 1000, 50–60000 milliseconds.
    #[row(description = "Discovery and capture interval: default 1000, 50–60000 milliseconds.")]
    pub poll_interval_ms: Option<u32>,
    // The watch seals and settles after this many seconds; absent runs until cancelled or sealed.
    #[row(
        description = "The watch seals and settles after this many seconds; absent runs until cancelled or sealed."
    )]
    pub maximum_seconds: Option<u32>,
    // Bytes the watch may capture: default 256 MiB, maximum 4 GiB.
    #[row(description = "Bytes the watch may capture: default 256 MiB, maximum 4 GiB.")]
    pub maximum_bytes: Option<u64>,
    // Stream policy only: each captured line is offered to every matcher and marker.
    #[row(
        description = "Stream policy only: each captured line is offered to every matcher and marker."
    )]
    pub matchers: Vec<LineMatcher>,
    pub markers: Vec<LineMarker>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchedArtifact {
    pub artifact: String,
    pub size_bytes: u64,
}

// What a watch that settled on its own sealed, and where its output skips.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchFileOutcome {
    pub artifacts: Vec<WatchedArtifact>,
    pub gaps: Vec<FileWatchGap>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWatchProgress {
    // The path relative to the source root.
    #[row(description = "The path relative to the source root.")]
    pub path: String,
    // Bytes captured into the output so far.
    #[row(description = "Bytes captured into the output so far.")]
    pub offset: u64,
    pub phase: FileWatchPhase,
}

// Source bytes the output does not contain, or a change in what it mirrors.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWatchGap {
    pub path: String,
    // The output offset at which the gap sits.
    #[row(description = "The output offset at which the gap sits.")]
    pub offset: u64,
    pub reason: FileWatchGapReason,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWatchSealed {
    pub path: String,
    pub artifact: Option<ArtifactMetadata>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWatchCompleted {
    pub artifacts: Vec<ArtifactMetadata>,
}

// Progress, gap and seal frames while watching, then one Completed frame when
// the bound elapses, the execution seals the watch or a start snapshot is taken.
// A cancelled watch still seals what it captured, without a final frame.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWatchFrame {
    pub payload: Option<FileWatchFramePayload>,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileRuleAction {
    Unspecified,
    Include,
    Exclude,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineFieldKind {
    Unspecified,
    Text,
    Integer,
    Number,
    Boolean,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum FileWatchSourcePayload {
    Local(LocalWatchSource),
    Ftp(FtpWatchSource),
}

// Stream captures bytes written after the watch starts: files present at the
// start baseline at their length, files appearing later from their first byte;
// a truncated file restarts from its first byte with a gap, and a replaced
// (rotated) file seals its artifact with a gap and starts a new one.
// Mirror keeps each output equal to its source, refetched when its size or
// modification time changes. A snapshot copies each file once.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileWatchPolicy {
    Unspecified,
    Stream,
    Mirror,
    SnapshotAtStart,
    SnapshotAtStop,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileWatchPhase {
    Unspecified,
    // Discovered; capture starts from the baseline.
    Baseline,
    Capturing,
    Synced,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileWatchGapReason {
    Unspecified,
    // The source shrank below the captured length; capture restarts at its first byte.
    Truncated,
    // The path names a different (rotated) file; the output is sealed and a new one starts.
    Replaced,
    // The source disappeared; its output is sealed.
    Vanished,
    // The server refused a ranged retrieval; the output now mirrors the whole file.
    ResumeUnsupported,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum FileWatchFramePayload {
    Progress(FileWatchProgress),
    Gap(FileWatchGap),
    Sealed(Box<FileWatchSealed>),
    Completed(FileWatchCompleted),
}

#[harmos::catalog]
pub enum Operations {
    FileWatch(FileWatch),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for FileWatch {
    const PATH: &'static str = "/dokime.v1.Files/FileWatch";
    const ACCESS: Access = Access::Control;
    // A local root reads the host's filesystem, which a remote listener never exposes.
    const LOCAL_ONLY: bool = true;
}

impl LineMatcher {
    // Publishes one row of `S` per matching line, each column read from the
    // named group of the same name.
    pub fn publishing<S: TelemetryStream>(pattern: impl Into<String>) -> Self
    where
        S::Row: harmos::Message + harmos::schema::Described,
    {
        Self {
            storage_schema: harmos::schema::graph::Schema::from_declaration(
                <S::Row as harmos::schema::Described>::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .expect("a matcher row declares a valid finite schema"),
            pattern: pattern.into(),
            stream: S::ID.to_owned(),
            fields: <S::Row as TelemetryRow>::columns()
                .into_iter()
                .filter_map(|column| {
                    let kind = match column.value_type {
                        ValueType::Text => LineFieldKind::Text,
                        ValueType::Integer => LineFieldKind::Integer,
                        ValueType::Number => LineFieldKind::Number,
                        ValueType::Boolean => LineFieldKind::Boolean,
                        _ => return None,
                    };
                    Some(LineField {
                        name: column.metadata.identifier.to_string(),
                        kind,
                    })
                })
                .collect(),
        }
    }
}

impl LineMarker {
    pub fn new(pattern: impl Into<String>, record_type: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            record_type: record_type.into(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
