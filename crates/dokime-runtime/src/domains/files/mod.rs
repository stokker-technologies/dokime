/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File captures: a local or FTP source polled during an execution, each selected
// file spooled privately and sealed as one execution artifact through the
// artifact upload path when the capture stops or the file vanishes. `CopyFiles`,
// `StartCapture` and `StopCapture` run these captures for a step.

/* === Modules === */

mod capture;
#[cfg(feature = "ftp")]
mod ftp;
pub(crate) mod local;
mod operation;
pub(crate) mod source;

/* === Re-Exports === */

pub use capture::{CaptureEvent, FileCapture, Mode, Phase};
pub use dokime_domain::model::files::wire::CaptureGapReason as GapReason;
pub use operation::{Resources as StepBuiltins, prepare};

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::files::{FileMatcher, LineMatchers};
use crate::{RuntimeHandle, runtime::Error, runtime::Result};
use dokime_domain::model::artifacts::name;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::files::wire::CaptureMode;
use dokime_domain::model::files::wire::FileLocation;
use dokime_domain::model::files::wire::FileRule;
use dokime_domain::model::files::wire::FileRuleAction;
use dokime_domain::model::files::wire::FileSource;
use dokime_domain::model::files::wire::LineMarker;
use dokime_domain::model::files::wire::LineMatcher;
use dokime_domain::model::identity::TestExecutionId;
use dokime_operations::files::CopyFiles;
use dokime_operations::files::StartCapture;
use source::Source;

/* === Constants === */

// Error code of a request the capture refuses before it starts.
pub const INVALID: &str = "file_capture_request";

// Error code of a capture that reached its file or byte limit.
pub const CAPPED: &str = "file_capture_cap";

// Error code of a source the capture cannot reach or read.
pub const UNREACHABLE: &str = "file_capture_source";

// Error code of a source that does not exist at the start.
pub const MISSING: &str = "file_capture_missing";

// Error code of a plain FTP source on a bench the profile does not trust.
pub const UNTRUSTED: &str = "file_capture_untrusted";

// Error code of a source this build does not support.
pub const UNSUPPORTED: &str = "file_capture_unsupported";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A validated request, before its source is opened.
struct Settings {
    scope: RecordScope,
    output: String,
    artifact: Option<String>,
    mode: Mode,
    newest: bool,
    single: bool,
    matcher: FileMatcher,
    lines: LineMatchers,
    poll_interval: Duration,
    maximum_files: usize,
    maximum_bytes: u64,
    maximum_failed_passes: u64,
    line_bytes: std::num::NonZeroUsize,
}

// A source reduced to one root and its selection, shared by the host and the
// harness so both discover the same files.
pub(crate) struct Selected {
    pub(crate) location: FileLocation,
    pub(crate) rules: Vec<FileRule>,
    pub(crate) single: bool,
    pub(crate) newest: bool,
    pub(crate) top_level: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    // Opens a capture that streams or mirrors for the calling step's execution.
    pub async fn open_file_capture(&self, request: &StartCapture, execution: &str) -> Result<FileCapture> {
        let (mode, poll_interval, matchers, markers): (Mode, Duration, &[LineMatcher], &[LineMarker]) = match &request.mode {
            CaptureMode::Stream(stream) => {
                (Mode::Stream, self.config().storage.filesystem.poll_interval, &stream.matchers, &stream.markers)
            }
            CaptureMode::Mirror(mirror) => (Mode::Mirror, mirror_interval(mirror.interval_s, &self.config().file_capture)?, &[], &[]),
        };
        for matcher in matchers {
            let stream = dokime_domain::model::identity::Identifier::new(&matcher.stream).map_err(|error| invalid(error.to_string()))?;
            let state = self.state_read();
            let contract =
                state.telemetry_definitions.get(&stream).ok_or_else(|| invalid(format!("matcher stream {stream} is not registered")))?;
            if contract.storage_schema.as_ref() != Some(&matcher.storage_schema) {
                return Err(invalid(format!("matcher stream {stream} has a different declared row schema")));
            }
        }
        let limits = self.config().file_capture;
        let lines = LineMatchers::compile(
            matchers,
            markers,
            limits.maximum_line_rules,
            limits.maximum_pattern_bytes,
            limits.maximum_regex_bytes,
            self.schema_settings(),
        )
        .map_err(|source| invalid(source.to_string()))?;
        self.open_capture(&request.source, &request.output, mode, poll_interval, lines, execution).await
    }

    // Opens a capture that copies its source once, on its first pass.
    pub async fn open_file_copy(&self, request: &CopyFiles, execution: &str) -> Result<FileCapture> {
        let poll_interval = self.config().storage.filesystem.poll_interval;
        self.open_capture(&request.source, &request.output, Mode::Copy, poll_interval, LineMatchers::default(), execution).await
    }

    async fn open_capture(
        &self, source: &FileSource, output: &str, mode: Mode, poll_interval: Duration, lines: LineMatchers, execution: &str,
    ) -> Result<FileCapture> {
        let selected = selected(source)?;
        validate_output(output, selected.single)?;
        let settings = self.capture_settings(&selected, output, mode, poll_interval, lines, execution)?;
        let source = self.capture_source(&selected.location).await?;
        let source = if selected.top_level { source.single_file() } else { source };
        FileCapture::open(self.clone(), settings, source).await
    }

    async fn capture_source(&self, location: &FileLocation) -> Result<Source> {
        match location {
            FileLocation::Local(local) => Source::local(&local.path).await,
            #[cfg(feature = "ftp")]
            FileLocation::Ftp(ftp) => {
                let password = match &ftp.secret_id {
                    Some(id) => Some(self.capture_secret(id).await?),
                    None => None,
                };
                Source::ftp(&ftp.url, password, self.config().trusted_bench).await
            }
            #[cfg(not(feature = "ftp"))]
            FileLocation::Ftp(_) => Err(Error::conflict(UNSUPPORTED, "this build has no FTP support")),
        }
    }

    fn capture_settings(
        &self, selected: &Selected, output: &str, mode: Mode, poll_interval: Duration, lines: LineMatchers, execution: &str,
    ) -> Result<Settings> {
        // Resolve the Execution and Its Cycle
        let execution = TestExecutionId::from_uuid(execution).map_err(|_| invalid("the execution id is invalid"))?;
        let scope = {
            let state = self.state_read();
            let view = state
                .executions
                .get(&execution)
                .ok_or_else(|| Error::not_found("execution_not_found", "the captured execution was not found"))?;
            if view.finished_at.is_some() {
                return Err(Error::conflict("execution_sealed", "the captured execution no longer accepts artifacts"));
            }
            RecordScope::default()
                .with_cycle(view.test_cycle_id.clone())
                .with_test_case(view.test_case_id.clone())
                .with_test_execution(execution)
        };
        let matcher = FileMatcher::compile(&selected.rules).map_err(|source| invalid(source.to_string()))?;
        let limits = self.config().file_capture;
        Ok(Settings {
            scope,
            output: if selected.single { String::new() } else { output.to_owned() },
            artifact: selected.single.then(|| output.to_owned()),
            mode,
            newest: selected.newest,
            single: selected.single,
            matcher,
            lines,
            poll_interval,
            maximum_files: if selected.single { 1 } else { limits.maximum_files },
            maximum_bytes: limits.maximum_bytes,
            maximum_failed_passes: limits.maximum_failed_passes,
            line_bytes: limits.maximum_line_bytes,
        })
    }

    // Reads the source password from the host secret store; it never
    // travels in the request.
    #[cfg(feature = "ftp")]
    async fn capture_secret(&self, secret_id: &str) -> Result<String> {
        let secrets = self.secret_store();
        let id = secret_id.to_owned();
        let read = secrets.get_secret(&id);
        match tokio::time::timeout(self.config().file_capture.secret_timeout, read).await {
            Ok(Ok(Some(secret))) => Ok(secret.expose().to_owned()),
            Ok(Ok(None)) => Err(Error::not_found("secret_not_found", format!("secret {secret_id:?} is not stored on this host"))),
            Ok(Err(_)) | Err(_) => {
                Err(Error::unavailable(UNREACHABLE, format!("secret {secret_id:?} could not be read from secret storage")))
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid(message: impl Into<String>) -> Error {
    Error::validation(INVALID, message, Vec::new())
}

// One request normalization for native and harness paths.
pub(crate) fn selected(source: &FileSource) -> Result<Selected> {
    match source {
        FileSource::File(location) => {
            let (location, file) = split_file(location)?;
            Ok(Selected {
                location,
                rules: vec![FileRule::include(literal(&file))],
                single: true,
                newest: false,
                top_level: true,
            })
        }
        FileSource::Tree(selection) => Ok(directory(&selection.root, &selection.rules, false)),
        FileSource::Newest(selection) => Ok(directory(&selection.root, &selection.rules, true)),
    }
}

// Depth follows the rules: discovery descends only when an include rule can
// match below the root.
fn directory(root: &FileLocation, rules: &[FileRule], newest: bool) -> Selected {
    let rules = if rules.is_empty() { vec![FileRule::include("**")] } else { rules.to_vec() };
    let descends = rules.iter().any(|rule| rule.action == FileRuleAction::Include && (rule.glob.contains('/') || rule.glob.contains("**")));
    Selected {
        location: root.clone(),
        rules,
        single: false,
        newest,
        top_level: !descends,
    }
}

pub(crate) fn validate_output(output: &str, single: bool) -> Result<()> {
    if single {
        return name::validate(output).map_err(|source| invalid(format!("the artifact name {output:?}: {source}")));
    }
    if !output.is_empty() && !name::is_relative_path(output) {
        return Err(invalid("the output prefix must be a relative path with forward slashes"));
    }
    Ok(())
}

// One Mirror cadence for native and harness paths.
pub(crate) fn mirror_interval(interval_s: Option<u32>, limits: &crate::config::FileCaptureSettings) -> Result<Duration> {
    let Some(interval_s) = interval_s else {
        return Ok(limits.mirror_interval);
    };
    let interval = Duration::from_secs(interval_s.into());
    if !(limits.minimum_mirror_interval..=limits.maximum_mirror_interval).contains(&interval) {
        return Err(invalid(format!("the mirror interval of {interval_s} s exceeds the configured bounds")));
    }
    Ok(interval)
}

// The record detail naming one gap reason.
pub(crate) fn gap_reason(reason: GapReason) -> &'static str {
    match reason {
        GapReason::Truncated => "truncated",
        GapReason::Replaced => "replaced",
        GapReason::Vanished => "vanished",
        GapReason::ResumeUnsupported => "resume_unsupported",
    }
}

// Splits a location naming one file into the location of its parent and the
// file's name.
fn split_file(location: &FileLocation) -> Result<(FileLocation, String)> {
    match location {
        FileLocation::Local(local) => {
            let path = std::path::Path::new(&local.path);
            let file = path.file_name().and_then(|file| file.to_str()).ok_or_else(|| invalid("the local source must name a file"))?;
            let parent = path.parent().ok_or_else(|| invalid("the local source must name a file"))?;
            Ok((FileLocation::from(parent), file.to_owned()))
        }
        FileLocation::Ftp(ftp) => {
            let trimmed = ftp.url.trim_end_matches('/');
            let (parent, file) =
                trimmed.rsplit_once('/').filter(|(_, file)| !file.is_empty()).ok_or_else(|| invalid("the FTP source must name a file"))?;
            let parent = if parent.is_empty() { "/" } else { parent };
            Ok((FileLocation::ftp(parent, ftp.secret_id.clone()), file.to_owned()))
        }
    }
}

// A glob matching exactly this name: every metacharacter in its own class.
fn literal(name: &str) -> String {
    name.chars()
        .map(|character| match character {
            '*' | '?' | '[' | ']' | '{' | '}' => format!("[{character}]"),
            other => other.to_string(),
        })
        .collect()
}

fn capped(message: impl Into<String>) -> Error {
    Error::overload(CAPPED, message)
}

fn unreachable_source(message: impl std::fmt::Display) -> Error {
    Error::unavailable(UNREACHABLE, message.to_string())
}

/* === Tests === */
