/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File sources captured during an execution: every selected file becomes one
// execution artifact. `CopyFiles` copies once; `StartCapture` and `StopCapture`
// bound a capture that streams or mirrors, and line matchers turn streamed
// lines into rows and records.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::telemetry::TelemetryRow;
use crate::model::telemetry::TelemetryStream;
use crate::model::values::ValueType;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One ordered include or exclude rule over paths relative to the source root.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileRule {
    #[row(description = "A glob over forward-slash relative paths; `*` stays within one segment, `**` crosses segments.")]
    pub glob: String,

    pub action: FileRuleAction,
}

// A path on the backend host.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalLocation {
    #[row(description = "An absolute path on the backend host.")]
    pub path: String,
}

// A path on an FTP server, listed and retrieved by polling.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FtpLocation {
    #[row(description = "`ftps://[user@]host[:port]/path` (explicit TLS), or `ftp://` on a trusted bench; never a password.")]
    pub url: String,

    #[row(description = "The host secret store entry holding the password; absent logs in anonymously.")]
    pub secret_id: Option<String>,
}

// A directory and the files it selects: the last rule matching a path decides,
// and empty rules select every file at any depth. Depth follows the rules, so
// `*.log` stays at the top level and `**/*.log` descends.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileSelection {
    pub root: FileLocation,
    pub rules: Vec<FileRule>,
}

// One captured named group read as a column of the matcher's stream.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LineField {
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

// Captures bytes written after the baseline: files present at the start from
// their length, later files from byte zero. A truncated or replaced file seals
// its segment and continues in a new one. Each captured line is offered to
// every matcher and marker.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamCapture {
    pub matchers: Vec<LineMatcher>,
    pub markers: Vec<LineMarker>,
}

// Keeps each output equal to its source, checked once per interval.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MirrorCapture {
    #[row(description = "Seconds between change checks; absent takes the configured interval.")]
    pub interval_s: Option<u32>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureStarted {
    pub capture_id: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedArtifact {
    pub artifact: String,
    pub size_bytes: u64,
}

// Source bytes the output does not contain, or a change in what it mirrors.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureGap {
    pub path: String,

    #[row(description = "The output offset at which the gap sits.")]
    pub offset: u64,

    pub reason: CaptureGapReason,
}

// What a copy or capture sealed, and where its output skips.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedFiles {
    pub artifacts: Vec<CapturedArtifact>,
    pub gaps: Vec<CaptureGap>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileRuleAction {
    Include,
    Exclude,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineFieldKind {
    Text,
    Integer,
    Number,
    Boolean,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum FileLocation {
    Local(LocalLocation),
    Ftp(FtpLocation),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum FileSource {
    // One file, which must exist at the start.
    File(FileLocation),

    // A directory's selected files, including later arrivals.
    Tree(FileSelection),

    // The selection's newest file at the start, followed through rotation.
    Newest(FileSelection),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum CaptureMode {
    Stream(StreamCapture),
    Mirror(MirrorCapture),
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureGapReason {
    // The source shrank; its segment is sealed and capture restarts at its first byte.
    Truncated,

    // The path names a different (rotated) file; the output is sealed and a new one starts.
    Replaced,

    // The source disappeared; its output is sealed.
    Vanished,

    // The server refused a ranged retrieval; extraction fails, while artifact-only capture falls back to mirroring.
    ResumeUnsupported,
}

/* === Traits === */

/* === Implementations === */

impl FileRule {
    pub fn include(glob: impl Into<String>) -> Self {
        Self {
            glob: glob.into(),
            action: FileRuleAction::Include,
        }
    }

    pub fn exclude(glob: impl Into<String>) -> Self {
        Self {
            glob: glob.into(),
            action: FileRuleAction::Exclude,
        }
    }
}

impl FileLocation {
    pub fn ftp(url: impl Into<String>, secret_id: Option<String>) -> Self {
        Self::Ftp(FtpLocation { url: url.into(), secret_id })
    }
}

impl<P: AsRef<Path>> From<P> for FileLocation {
    fn from(path: P) -> Self {
        Self::Local(LocalLocation {
            path: path.as_ref().to_string_lossy().into_owned(),
        })
    }
}

impl FileSource {
    pub fn file(location: impl Into<FileLocation>) -> Self {
        Self::File(location.into())
    }

    pub fn tree(root: impl Into<FileLocation>, rules: impl IntoIterator<Item = FileRule>) -> Self {
        Self::Tree(FileSelection {
            root: root.into(),
            rules: rules.into_iter().collect(),
        })
    }

    pub fn newest(root: impl Into<FileLocation>, rules: impl IntoIterator<Item = FileRule>) -> Self {
        Self::Newest(FileSelection {
            root: root.into(),
            rules: rules.into_iter().collect(),
        })
    }
}

impl StreamCapture {
    pub fn plain() -> Self {
        Self {
            matchers: Vec::new(),
            markers: Vec::new(),
        }
    }

    #[must_use]
    pub fn matcher(mut self, matcher: LineMatcher) -> Self {
        self.matchers.push(matcher);
        self
    }

    #[must_use]
    pub fn marker(mut self, marker: LineMarker) -> Self {
        self.markers.push(marker);
        self
    }
}

impl MirrorCapture {
    pub fn configured() -> Self {
        Self { interval_s: None }
    }

    pub fn every(interval_s: u32) -> Self {
        Self { interval_s: Some(interval_s) }
    }
}

impl From<StreamCapture> for CaptureMode {
    fn from(capture: StreamCapture) -> Self {
        Self::Stream(capture)
    }
}

impl From<MirrorCapture> for CaptureMode {
    fn from(capture: MirrorCapture) -> Self {
        Self::Mirror(capture)
    }
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
