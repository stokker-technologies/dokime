/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File watches: a local or FTP source polled during an execution, each matched
// file spooled privately and sealed as one execution artifact through the
// artifact upload path when the watch stops or the file vanishes. The
// `WatchFile` operation runs the same watch over one file for a step, with its
// line matchers.

/* === Modules === */
mod ftp;
mod local;
mod operation;
mod source;
mod watch;

/* === Re-Exports === */
pub use dokime_domain::model::files::wire::FileWatchGapReason as GapReason;
pub use dokime_domain::model::files::wire::FileWatchPhase as Phase;
pub use dokime_domain::model::files::wire::FileWatchPolicy as Policy;
pub use operation::{Resources as StepBuiltins, register};
pub use watch::{FileWatch, WatchEvent};

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::files::{FileMatcher, LineMatchers};
use crate::{RuntimeHandle, runtime::Error, runtime::Result};
use dokime_domain::model::artifacts::name;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::files::wire::FileRule;
use dokime_domain::model::files::wire::FileRuleAction;
use dokime_domain::model::files::wire::FileWatchSource;
use dokime_domain::model::files::wire::FileWatchSourcePayload;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::operations::files::FileWatch;
use dokime_domain::operations::files::WatchFile;
use source::Source;

/* === Constants === */
// Error code of a request the watch refuses before it starts.
pub const INVALID: &str = "file_watch_request";

// Error code of a watch that reached its file or byte cap.
pub const CAPPED: &str = "file_watch_cap";

// Error code of a source the watch cannot reach or read.
pub const UNREACHABLE: &str = "file_watch_source";

// Error code of a plain FTP source on a bench the profile does not trust.
pub const UNTRUSTED: &str = "file_watch_untrusted";

// Error code of a source scheme this build does not support yet.
pub const UNSUPPORTED: &str = "file_watch_unsupported";

pub const DEFAULT_POLL: Duration = Duration::from_millis(1000);
pub const MINIMUM_POLL: Duration = Duration::from_millis(50);
pub const MAXIMUM_POLL: Duration = Duration::from_millis(60_000);

// The longest bound one watch may ask for, matching the follow stream policy.
pub const MAXIMUM_SECONDS: u32 = 86_400;

pub const DEFAULT_FILES: usize = 64;

// An execution holds at most 256 outputs; one watch may claim all of them.
pub const MAXIMUM_FILES: usize = 256;

pub const DEFAULT_BYTES: u64 = 256 * 1024 * 1024;
pub const MAXIMUM_BYTES: u64 = 4 * 1024 * 1024 * 1024;

// Bounds one discovery pass, so a vast or looping tree is refused, not walked.
const SCAN_ENTRIES: usize = 100_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// A validated watch request, before its source is opened.
struct Settings {
    scope: RecordScope,
    output: String,
    artifact: Option<String>,
    policy: Policy,
    matcher: FileMatcher,
    lines: LineMatchers,
    poll_interval: Duration,
    deadline: Option<Duration>,
    maximum_files: usize,
    maximum_bytes: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    // Opens a watch over the request's source, scoped to its explicit execution
    // or to the calling step's execution; the two must agree when both exist.
    pub async fn open_file_watch(&self, request: &FileWatch, context_execution: Option<&str>) -> Result<FileWatch> {
        let settings = self.watch_settings(request, context_execution)?;
        let source = self.watch_source(request.source.as_ref()).await?;
        FileWatch::open(self.clone(), settings, source).await
    }

    // Opens a watch over the one file the request's source names: its parent
    // is the root and its name the only rule.
    pub async fn open_watch_file(&self, request: &WatchFile, context_execution: Option<&str>) -> Result<FileWatch> {
        let (source, file) = single_file(request.source.as_ref())?;
        if let Some(artifact) = &request.artifact
            && !name::is_relative_path(artifact)
        {
            return Err(invalid("the artifact must be a relative path with forward slashes"));
        }
        let mut settings = self.watch_settings(
            &FileWatch {
                execution_id: request.execution_id.clone(),
                source: None,
                rules: vec![FileRule { glob: literal(&file), action: FileRuleAction::Include }],
                output: request.output.clone(),
                policy: request.policy,
                poll_interval_ms: request.poll_interval_ms,
                maximum_seconds: request.maximum_seconds,
                maximum_files: Some(1),
                maximum_bytes: request.maximum_bytes,
            },
            context_execution,
        )?;
        for matcher in &request.matchers {
            let stream = dokime_domain::model::identity::Identifier::new(&matcher.stream).map_err(|error| invalid(error.to_string()))?;
            let state = self.state_read();
            let contract =
                state.telemetry_definitions.get(&stream).ok_or_else(|| invalid(format!("matcher stream {stream} is not registered")))?;
            if contract.storage_schema.as_ref() != Some(&matcher.storage_schema) {
                return Err(invalid(format!("matcher stream {stream} has a different declared row schema")));
            }
        }
        settings.artifact = request.artifact.clone();
        settings.lines = LineMatchers::compile(&request.matchers, &request.markers).map_err(|source| invalid(source.to_string()))?;
        if !settings.lines.is_empty() && settings.policy != Policy::Stream {
            return Err(invalid("line matchers and markers need the Stream policy"));
        }
        // The named file sits in its parent: no deeper entry can match it.
        let source = self.watch_source(Some(&source)).await?.single_file();
        FileWatch::open(self.clone(), settings, source).await
    }

    async fn watch_source(&self, source: Option<&FileWatchSource>) -> Result<Source> {
        Ok(match source.and_then(|source| source.payload.as_ref()) {
            Some(FileWatchSourcePayload::Local(local)) => Source::local(&local.root).await?,
            Some(FileWatchSourcePayload::Ftp(ftp)) => {
                let password = match &ftp.credential_id {
                    Some(id) => Some(self.watch_credential(id).await?),
                    None => None,
                };
                Source::ftp(&ftp.url, password, self.config().trusted_bench).await?
            }
            None => return Err(invalid("a file watch needs a local or FTP source")),
        })
    }

    fn watch_settings(&self, request: &FileWatch, context_execution: Option<&str>) -> Result<Settings> {
        // Resolve The Execution And Its Cycle
        let execution = match (request.execution_id.as_deref(), context_execution) {
            (Some(explicit), Some(context)) if explicit != context => {
                return Err(invalid("the explicit execution differs from the calling step's execution"));
            }
            (Some(execution), _) | (None, Some(execution)) => execution,
            (None, None) => return Err(invalid("a file watch requires an execution")),
        };
        let execution = TestExecutionId::from_uuid(execution).map_err(|_| invalid("the execution id is invalid"))?;
        let scope = {
            let state = self.state_read();
            let view = state
                .executions
                .get(&execution)
                .ok_or_else(|| Error::not_found("execution_not_found", "the watched execution was not found"))?;
            if view.finished_at.is_some() {
                return Err(Error::conflict("execution_sealed", "the watched execution no longer accepts artifacts"));
            }
            RecordScope::default()
                .with_cycle(view.test_cycle_id.clone())
                .with_test_case(view.test_case_id.clone())
                .with_test_execution(execution)
        };

        // Validate The Output, Rules, Policy And Caps
        if !request.output.is_empty() && !name::is_relative_path(&request.output) {
            return Err(invalid("the output folder must be a relative path with forward slashes"));
        }
        let matcher = FileMatcher::compile(&request.rules).map_err(|source| invalid(source.to_string()))?;
        if request.policy == Policy::Unspecified {
            return Err(invalid("a file watch needs a policy"));
        }
        let poll_interval = request.poll_interval_ms.map_or(DEFAULT_POLL, |milliseconds| Duration::from_millis(milliseconds.into()));
        if !(MINIMUM_POLL..=MAXIMUM_POLL).contains(&poll_interval) {
            return Err(invalid("the poll interval must be 50–60000 milliseconds"));
        }
        let deadline = match request.maximum_seconds {
            Some(seconds) if seconds == 0 || seconds > MAXIMUM_SECONDS => {
                return Err(invalid(format!("the watch bound must be 1–{MAXIMUM_SECONDS} seconds")));
            }
            seconds => seconds.map(|seconds| Duration::from_secs(seconds.into())),
        };
        let maximum_files = request.maximum_files.map_or(DEFAULT_FILES, |files| files as usize);
        if maximum_files == 0 || maximum_files > MAXIMUM_FILES {
            return Err(invalid(format!("a watch may track 1–{MAXIMUM_FILES} files")));
        }
        let maximum_bytes = request.maximum_bytes.unwrap_or(DEFAULT_BYTES);
        if maximum_bytes == 0 || maximum_bytes > MAXIMUM_BYTES {
            return Err(invalid("a watch may capture 1 byte to 4 GiB"));
        }
        Ok(Settings {
            scope,
            output: request.output.clone(),
            artifact: None,
            policy: request.policy,
            matcher,
            lines: LineMatchers::default(),
            poll_interval,
            deadline,
            maximum_files,
            maximum_bytes,
        })
    }

    // Reads the source password from the host credential store; it never
    // travels in the request.
    async fn watch_credential(&self, credential_id: &str) -> Result<String> {
        let credentials = self.credential_store();
        let id = credential_id.to_owned();
        let read = tokio::task::spawn_blocking(move || credentials.get_secret(&id));
        match tokio::time::timeout(Duration::from_secs(5), read).await {
            Ok(Ok(Ok(Some(secret)))) => Ok(secret.expose().to_owned()),
            Ok(Ok(Ok(None))) => {
                Err(Error::not_found("credential_not_found", format!("credential {credential_id:?} is not stored on this host")))
            }
            Ok(Ok(Err(_))) | Ok(Err(_)) | Err(_) => {
                Err(Error::unavailable(UNREACHABLE, format!("credential {credential_id:?} could not be read from credential storage")))
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(message: impl Into<String>) -> Error {
    Error::validation(INVALID, message, Vec::new())
}

// Splits a source naming one file into the source over its parent and the
// file's name.
fn single_file(source: Option<&FileWatchSource>) -> Result<(FileWatchSource, String)> {
    let split = |path: &str| {
        let trimmed = path.trim_end_matches('/');
        trimmed
            .rsplit_once('/')
            .filter(|(_, file)| !file.is_empty())
            .map(|(parent, file)| (if parent.is_empty() { "/" } else { parent }.to_owned(), file.to_owned()))
    };
    match source.and_then(|source| source.payload.as_ref()) {
        Some(FileWatchSourcePayload::Local(local)) => {
            let path = std::path::Path::new(&local.root);
            let file = path.file_name().and_then(|file| file.to_str()).ok_or_else(|| invalid("the local source must name a file"))?;
            let parent = path.parent().and_then(|parent| parent.to_str()).ok_or_else(|| invalid("the local source must name a file"))?;
            Ok((
                FileWatchSource {
                    payload: Some(FileWatchSourcePayload::Local(dokime_domain::model::files::wire::LocalWatchSource {
                        root: parent.to_owned(),
                    })),
                },
                file.to_owned(),
            ))
        }
        Some(FileWatchSourcePayload::Ftp(ftp)) => {
            let (parent, file) = split(&ftp.url).ok_or_else(|| invalid("the FTP source must name a file"))?;
            Ok((
                FileWatchSource {
                    payload: Some(FileWatchSourcePayload::Ftp(dokime_domain::model::files::wire::FtpWatchSource {
                        url: parent,
                        credential_id: ftp.credential_id.clone(),
                    })),
                },
                file,
            ))
        }
        None => Err(invalid("a file watch needs a local or FTP source")),
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
