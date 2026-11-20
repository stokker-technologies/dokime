/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Local File Capture Engine

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// Third Party Imports
use prost::Message;
use regex::Regex;
use sha2::{Digest, Sha256};

// Local Imports
use crate::host::Context;
use crate::host::content::artifacts;
use crate::host::files::uploads;
use crate::{Error, Result};
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::files::{
    CaptureMode, CapturesStopResponse, UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsPublishRequest,
    capture_mode,
};
use dokime_protocol::model::{
    CaptureGap, CaptureGapReason, CapturedArtifact, FileLocation, FileRule, FileRuleAction, FileSource, file_location, file_source,
};
use harmos_protocol::model::FileMetadata;

/* === Constants === */

const MAX_RULES: usize = 64;
const MAX_GLOB_BYTES: usize = 1024;

/* === Macros === */

/* === Type Aliases === */

// Matched files by forward-slash path relative to the source root.
type Listing = BTreeMap<String, Entry>;

/* === Structs === */

/// One Admitted Capture: Its Caller, Source, Output and Baseline.
pub(crate) struct Capture {
    context: Context,
    identifier: u64,
    action: &'static str,
    source: Source,
    output: String,
    mode: Mode,
    baseline: Listing,
}

struct Source {
    root: PathBuf,
    rules: Vec<(Regex, bool)>,
    single: bool,
    newest: Option<String>,
    top_level: bool,
    scan_entries: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    size: u64,
    modified: Option<SystemTime>,
    identity: Option<(u64, u64)>,
}

/* === Enums === */

/// What a Capture Does to a Selected File's Bytes at Its Final Pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Each selected file once, as it is now.
    Copy,
    /// Bytes written after the baseline: existing files from their baseline length, later files whole.
    Stream,
    /// Each selected file whole, as it is when the capture stops.
    Mirror,
}

/* === Traits === */

/* === Implementations === */

impl Mode {
    pub(crate) fn of(action: &str, mode: Option<&CaptureMode>) -> Result<Self> {
        match mode.and_then(|mode| mode.kind.as_ref()) {
            Some(capture_mode::Kind::Stream(stream)) if stream.matchers.is_empty() && stream.markers.is_empty() => Ok(Self::Stream),
            Some(capture_mode::Kind::Stream(_)) => {
                Err(Error::Unimplemented(format!("{action}: Line Matchers and Markers Are Deferred; Capture the Stream Without Them")))
            }
            Some(capture_mode::Kind::Mirror(_)) => Ok(Self::Mirror),
            None => Err(Error::Invalid(format!("{action}: Select Stream or Mirror Capture"))),
        }
    }
}

impl Capture {
    /// Validates the request, admits its local root and takes the baseline.
    pub(crate) async fn open(
        context: &Context, action: &'static str, source: Option<&FileSource>, output: &str, mode: Mode,
    ) -> Result<Self> {
        let caller = context.caller.as_ref().ok_or_else(|| Error::Permission(format!("{action}: Call From a Plugin Execution")))?;
        caller.check_action_scope(&caller.execution, action)?;
        let source =
            source.and_then(|source| source.kind.as_ref()).ok_or_else(|| Error::Invalid(format!("{action}: Select a File Source")))?;
        let newest = matches!(source, file_source::Kind::Newest(_));
        let settings = context.captures.settings();
        let (location, rules, single) = match source {
            file_source::Kind::File(location) => {
                let path = local(action, location)?;
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| Error::Invalid(format!("{action}: Name a Local File")))?;
                let parent = path.parent().ok_or_else(|| Error::Invalid(format!("{action}: Name a Local File")))?;
                let rule = FileRule {
                    glob: literal(name),
                    action: FileRuleAction::Include as i32,
                };
                (parent.to_path_buf(), vec![rule], true)
            }
            file_source::Kind::Tree(selection) | file_source::Kind::Newest(selection) => {
                let root = selection.root.as_ref().ok_or_else(|| Error::Invalid(format!("{action}: Select the Source Root")))?;
                (local(action, root)?, selection.rules.clone(), false)
            }
        };
        validate_output(action, output, single)?;
        let rules = if rules.is_empty() {
            vec![FileRule {
                glob: "**".into(),
                action: FileRuleAction::Include as i32,
            }]
        } else {
            rules
        };
        let top_level = single
            || !rules
                .iter()
                .any(|rule| rule.action == FileRuleAction::Include as i32 && (rule.glob.contains('/') || rule.glob.contains("**")));
        let rules = compile(action, &rules)?;
        let root = admit_root(action, &location)?;
        let mut source = Source {
            root,
            rules,
            single,
            newest: None,
            top_level,
            scan_entries: settings.scan_entries,
        };
        let listing = source.scan(action).await?;
        if single && listing.is_empty() {
            return Err(Error::NotFound(format!("{action}: Name an Existing File")));
        }
        if newest {
            let newest = listing
                .iter()
                .max_by(|(left_path, left), (right_path, right)| left.modified.cmp(&right.modified).then_with(|| right_path.cmp(left_path)))
                .map(|(path, _)| path.clone())
                .ok_or_else(|| Error::NotFound(format!("{action}: Add a Matching File Before Capturing the Newest")))?;
            source.newest = Some(newest);
        }
        let baseline = if mode == Mode::Stream { source.select(listing) } else { Listing::new() };
        Ok(Self {
            context: context.clone(),
            identifier: context.captures.identifier(),
            action,
            source,
            output: output.to_owned(),
            mode,
            baseline,
        })
    }
    pub(crate) fn identifier(&self) -> u64 {
        self.identifier
    }
    pub(crate) fn execution(&self) -> &str {
        self.context.caller.as_ref().map_or("", |caller| caller.execution.as_str())
    }
    /// Takes the final pass and attaches every captured file to the execution.
    ///
    /// Gaps and an early end become execution evidence; what was captured before a failure stays attached.
    pub(crate) async fn seal(self) -> Result<CapturesStopResponse> {
        let mut artifacts = Vec::new();
        let mut gaps = Vec::new();
        let sealed = self.pass(&mut artifacts, &mut gaps).await;
        if let Err(error) = &sealed
            && let Some(caller) = &self.context.caller
            && let Err(cause) = caller.failure("capture", error)
        {
            tracing::warn!(%cause, "Record File Capture Failure");
        }
        sealed?;
        Ok(CapturesStopResponse { artifacts, gaps })
    }
    async fn pass(&self, artifacts: &mut Vec<CapturedArtifact>, gaps: &mut Vec<CaptureGap>) -> Result<()> {
        let settings = self.context.captures.settings();
        let listing = match self.source.scan(self.action).await {
            Ok(listing) => self.source.select(listing),
            Err(Error::NotFound(_)) if self.mode == Mode::Stream => Listing::new(),
            Err(error) => return Err(error),
        };
        // Files the Baseline Held That Are Gone Leave a Gap; They Contribute No Bytes.
        for (path, before) in &self.baseline {
            if !listing.contains_key(path) {
                self.gap(gaps, path, before.size, CaptureGapReason::Vanished)?;
            }
        }
        if listing.len() > settings.maximum_files {
            return Err(Error::Bound(format!("{}: Narrow the Source Rules or Raise the Host's Capture File Limit", self.action)));
        }
        let mut captured = 0_u64;
        for (path, entry) in &listing {
            let offset = match (self.mode, self.baseline.get(path)) {
                (Mode::Stream, Some(before)) if before.identity.is_some() && before.identity != entry.identity => {
                    self.gap(gaps, path, before.size, CaptureGapReason::Replaced)?;
                    0
                }
                (Mode::Stream, Some(before)) if entry.size < before.size => {
                    self.gap(gaps, path, before.size, CaptureGapReason::Truncated)?;
                    0
                }
                (Mode::Stream, Some(before)) if entry.size == before.size => continue,
                (Mode::Stream, Some(before)) => before.size,
                _ => 0,
            };
            let length = entry.size - offset;
            captured = captured.saturating_add(length);
            if captured > settings.maximum_bytes {
                return Err(Error::Bound(format!("{}: Narrow the Source Rules or Raise the Host's Capture Byte Limit", self.action)));
            }
            artifacts.push(self.attach(path, offset, length).await?);
        }
        Ok(())
    }
    fn gap(&self, gaps: &mut Vec<CaptureGap>, path: &str, offset: u64, reason: CaptureGapReason) -> Result<()> {
        let gap = CaptureGap {
            path: path.to_owned(),
            offset,
            reason: reason as i32,
        };
        if let Some(caller) = &self.context.caller {
            caller.evidence("capture:gap", gap.encode_to_vec(), None)?;
        }
        gaps.push(gap);
        Ok(())
    }
    // Uploads One Byte Range Through the Native File Path, Then Names It in the Execution.
    async fn attach(&self, path: &str, offset: u64, length: u64) -> Result<CapturedArtifact> {
        let context = &self.context;
        let name = if self.source.single {
            self.output.clone()
        } else if self.output.is_empty() {
            path.to_owned()
        } else {
            format!("{}/{path}", self.output)
        };
        let request = |step: &str| format!("capture/{}/{name}/{step}", self.identifier);
        let created = uploads::create::handle(
            context,
            UploadsCreateRequest {
                metadata: Some(FileMetadata {
                    id: String::new(),
                    scope: None,
                    media_type: media_type(&name).into(),
                    expected_length: Some(length),
                    expected_sha256: None,
                }),
                request_id: request("create"),
            },
        )
        .await?;
        let upload = created.upload.ok_or_else(|| Error::Precondition(format!("{}: Return the Native Upload", self.action)))?;
        let file = self.source.root.join(path);
        let transferred = async {
            let mut digest = Sha256::new();
            let mut written = 0_u64;
            while written < length {
                let count = (length - written).min(created.chunk_bytes.max(1));
                let data = read(self.action, &file, offset + written, count).await?;
                digest.update(&data);
                uploads::append::handle(
                    context,
                    UploadsAppendRequest {
                        upload: Some(upload.clone()),
                        offset: written,
                        data,
                        request_id: request(&format!("append/{written}")),
                    },
                )
                .await?;
                written += count;
            }
            let published = uploads::publish::handle(
                context,
                UploadsPublishRequest {
                    upload: Some(upload.clone()),
                    length,
                    sha256: digest.finalize().to_vec(),
                    request_id: request("publish"),
                },
            )
            .await?;
            published.upload.ok_or_else(|| Error::Precondition(format!("{}: Return the Published File", self.action)))
        }
        .await;
        let reference = match transferred {
            Ok(reference) => reference,
            Err(error) => {
                let removal = UploadsDeleteRequest {
                    upload: Some(upload),
                    request_id: request("delete"),
                };
                if let Err(cleanup) = uploads::delete::handle(context, removal).await {
                    tracing::warn!(%cleanup, "Remove Failed Capture Upload");
                }
                return Err(error);
            }
        };
        artifacts::create::handle(
            context,
            ArtifactsCreateRequest {
                file: Some(reference),
                scope: None,
                filename: name.clone(),
                content_type: media_type(&name).into(),
                request_id: request("artifact"),
                metadata: None,
            },
        )
        .await?;
        Ok(CapturedArtifact { artifact: name, size_bytes: length })
    }
}

impl Source {
    async fn scan(&self, action: &'static str) -> Result<Listing> {
        let root = self.root.clone();
        let rules = self.rules.clone();
        let depth = if self.top_level { 1 } else { usize::MAX };
        let limit = self.scan_entries;
        tokio::task::spawn_blocking(move || walk(action, &root, &rules, depth, limit))
            .await
            .map_err(|error| Error::Unavailable(format!("{action}: Retry the Directory Scan: {error}")))?
    }
    // Newest Follows Its Selected Path; Other Sources Keep Every Match.
    fn select(&self, mut listing: Listing) -> Listing {
        if let Some(newest) = &self.newest {
            listing.retain(|path, _| path == newest);
        }
        listing
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn local(action: &str, location: &FileLocation) -> Result<PathBuf> {
    match location.kind.as_ref() {
        Some(file_location::Kind::Local(path)) => {
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err(Error::Invalid(format!("{action}: Name an Absolute Local Path")));
            }
            Ok(path)
        }
        Some(file_location::Kind::Ftp(_)) => {
            Err(Error::Unimplemented(format!("{action}: FTP and FTPS Capture Is Deferred; Capture a Local Directory Instead")))
        }
        None => Err(Error::Invalid(format!("{action}: Select a Local Path"))),
    }
}

// The Source Root Is an Existing Directory, Resolved Through Its Links.
fn admit_root(action: &str, root: &Path) -> Result<PathBuf> {
    let resolved = std::fs::canonicalize(root).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => Error::NotFound(format!("{action}: Name an Existing Source Directory")),
        _ => Error::Unavailable(format!("{action}: Make the Source Directory Readable: {error}")),
    })?;
    if !resolved.is_dir() {
        return Err(Error::Invalid(format!("{action}: Name a Source Directory")));
    }
    Ok(resolved)
}

fn walk(action: &str, root: &Path, rules: &[(Regex, bool)], depth: usize, limit: usize) -> Result<Listing> {
    let mut listing = Listing::new();
    let mut pending = vec![(root.to_path_buf(), String::new(), 1_usize)];
    let mut visited = 0_usize;
    while let Some((directory, prefix, level)) = pending.pop() {
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && level > 1 => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::NotFound(format!("{action}: Name an Existing Source Directory")));
            }
            Err(error) => return Err(Error::Unavailable(format!("{action}: Make the Source Directory Readable: {error}"))),
        };
        for entry in entries {
            visited += 1;
            if visited > limit {
                return Err(Error::Bound(format!("{action}: Narrow the Source Directory to Fewer Entries")));
            }
            // A Concurrently Removed Entry Is Simply Absent from This Pass.
            let Ok(entry) = entry else { continue };
            let Ok(metadata) = std::fs::symlink_metadata(entry.path()) else { continue };
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else { continue };
            let relative = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
            if metadata.is_dir() {
                if level < depth {
                    pending.push((entry.path(), relative, level + 1));
                }
                continue;
            }
            if !metadata.is_file() || !matches(rules, &relative) {
                continue;
            }
            listing.insert(
                relative,
                Entry {
                    size: metadata.len(),
                    modified: metadata.modified().ok(),
                    identity: identity(&metadata),
                },
            );
        }
    }
    Ok(listing)
}

// The Last Matching Rule Wins.
fn matches(rules: &[(Regex, bool)], relative: &str) -> bool {
    rules.iter().rev().find(|(glob, _)| glob.is_match(relative)).is_some_and(|(_, include)| *include)
}

fn compile(action: &str, rules: &[FileRule]) -> Result<Vec<(Regex, bool)>> {
    if rules.len() > MAX_RULES || !rules.iter().any(|rule| rule.action == FileRuleAction::Include as i32) {
        return Err(Error::Invalid(format!("{action}: Supply 1 to {MAX_RULES} Rules With at Least One Include")));
    }
    let mut compiled = Vec::with_capacity(rules.len());
    for rule in rules {
        let include = match FileRuleAction::try_from(rule.action) {
            Ok(FileRuleAction::Include) => true,
            Ok(FileRuleAction::Exclude) => false,
            _ => return Err(Error::Invalid(format!("{action}: Choose Include or Exclude for Every Rule"))),
        };
        if rule.glob.is_empty() || rule.glob.len() > MAX_GLOB_BYTES || rule.glob.starts_with('/') || rule.glob.contains('\\') {
            return Err(Error::Invalid(format!("{action}: Write Rule Globs as Relative Forward-Slash Paths")));
        }
        let pattern = glob(&rule.glob).ok_or_else(|| Error::Invalid(format!("{action}: Close Every Character Class in Rule Globs")))?;
        let pattern = Regex::new(&pattern).map_err(|error| Error::Invalid(format!("{action}: Correct the Rule Glob: {error}")))?;
        compiled.push((pattern, include));
    }
    Ok(compiled)
}

// `*` and `?` stay within one segment, `**` spans segments and `[...]` selects one character.
fn glob(pattern: &str) -> Option<String> {
    let characters = pattern.chars().collect::<Vec<_>>();
    let mut expression = String::from("^");
    let mut index = 0;
    while index < characters.len() {
        match characters[index] {
            '*' if characters.get(index + 1) == Some(&'*') => {
                if characters.get(index + 2) == Some(&'/') {
                    expression.push_str("(?:.*/)?");
                    index += 3;
                } else {
                    expression.push_str(".*");
                    index += 2;
                }
                continue;
            }
            '*' => expression.push_str("[^/]*"),
            '?' => expression.push_str("[^/]"),
            '[' => {
                let end = characters[index + 1..].iter().position(|character| *character == ']')? + index + 1;
                let mut class = characters[index + 1..end].iter().collect::<String>();
                let negated = class.starts_with('!');
                if negated {
                    class.remove(0);
                }
                let class = class.replace('\\', "\\\\").replace('[', "\\[").replace('^', "\\^");
                expression.push_str(&format!("[{}{class}]", if negated { "^" } else { "" }));
                index = end;
            }
            character => expression.push_str(&regex::escape(&character.to_string())),
        }
        index += 1;
    }
    expression.push('$');
    Some(expression)
}

// A Glob Matching Exactly This Name: Every Metacharacter in Its Own Class.
fn literal(name: &str) -> String {
    name.chars()
        .map(|character| match character {
            '*' | '?' | '[' | ']' => format!("[{character}]"),
            other => other.to_string(),
        })
        .collect()
}

fn validate_output(action: &str, output: &str, single: bool) -> Result<()> {
    if output.is_empty() && !single {
        return Ok(());
    }
    let valid = !output.is_empty()
        && !output.contains(['\\', ':'])
        && output.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != "..");
    if !valid {
        return Err(Error::Invalid(format!("{action}: Name the Output as a Relative Forward-Slash Path Without Traversal")));
    }
    Ok(())
}

async fn read(action: &'static str, path: &Path, offset: u64, length: u64) -> Result<Vec<u8>> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let failure = |error: std::io::Error| Error::Unavailable(format!("{action}: Keep the Captured File Readable: {error}"));
        let mut file = std::fs::File::open(&path).map_err(failure)?;
        file.seek(SeekFrom::Start(offset)).map_err(failure)?;
        let mut data = Vec::with_capacity(length as usize);
        file.take(length).read_to_end(&mut data).map_err(failure)?;
        if (data.len() as u64) < length {
            return Err(Error::Conflict(format!("{action}: Keep the Captured File From Shrinking While It Is Read")));
        }
        Ok(data)
    })
    .await
    .map_err(|error| Error::Unavailable(format!("{action}: Retry the File Read: {error}")))?
}

fn media_type(name: &str) -> &'static str {
    match name.rsplit_once('.').map(|(_, extension)| extension.to_ascii_lowercase()).as_deref() {
        Some("log" | "txt") => "text/plain",
        Some("csv") => "text/csv",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
}

#[cfg(unix)]
fn identity(metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((metadata.dev(), metadata.ino()))
}

// Windows Exposes No Stable File Index Here; Creation Time Distinguishes a Replacement.
#[cfg(not(unix))]
fn identity(metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    let created = metadata.created().ok()?.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    Some((created.as_secs(), u64::from(created.subsec_nanos())))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{admit_root, glob, matches, validate_output};
    use crate::Error;
    use regex::Regex;

    fn rules(rules: &[(&str, bool)]) -> Vec<(Regex, bool)> {
        rules.iter().map(|(pattern, include)| (Regex::new(&glob(pattern).unwrap()).unwrap(), *include)).collect()
    }

    #[test]
    fn rules_select_depth_and_the_last_matching_rule_wins() {
        let top = rules(&[("*.log", true)]);
        assert!(matches(&top, "device.log"));
        assert!(!matches(&top, "nested/device.log"));
        let deep = rules(&[("**/*.log", true), ("archive/**", false)]);
        assert!(matches(&deep, "device.log"));
        assert!(matches(&deep, "nested/device.log"));
        assert!(!matches(&deep, "archive/device.log"));
        let classes = rules(&[("run[0-9].bin", true), ("run[!0-4].bin", false)]);
        assert!(matches(&classes, "run3.bin"));
        assert!(!matches(&classes, "run7.bin"));
        assert!(glob("run[0-9.bin").is_none());
    }

    #[test]
    fn sources_resolve_to_existing_directories() {
        let source = tempfile::tempdir().unwrap();
        std::fs::create_dir(source.path().join("device")).unwrap();
        std::fs::write(source.path().join("device.log"), b"").unwrap();
        let resolved = admit_root("Capture", &source.path().join("device/../device")).unwrap();
        assert_eq!(resolved, std::fs::canonicalize(source.path().join("device")).unwrap());
        assert!(matches!(admit_root("Capture", &source.path().join("device.log")), Err(Error::Invalid(_))));
        assert!(matches!(admit_root("Capture", &source.path().join("missing")), Err(Error::NotFound(_))));
    }

    #[test]
    fn outputs_stay_relative_and_within_the_execution() {
        assert!(validate_output("Capture", "", false).is_ok());
        assert!(validate_output("Capture", "device/logs", false).is_ok());
        assert!(validate_output("Capture", "", true).is_err());
        for invalid in ["/absolute", "a/../b", "a//b", "c:\\device", "./device"] {
            assert!(validate_output("Capture", invalid, false).is_err(), "{invalid}");
        }
    }
}
