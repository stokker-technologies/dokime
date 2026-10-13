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
use std::backtrace::Backtrace;
use std::fmt;
use std::panic::Location;
// Third Party Imports
use serde::{Deserialize, Serialize};
use tracing_error::SpanTrace;
// Local Imports

/* === Constants === */
const HOME: &str = "~";

const USER: &str = "<user>";

const CAPTURE: &str = module_path!();

const UNKNOWN: &str = "<unknown>";

const UNRESOLVED_HINT: &str = "no frame resolved to a symbol: the plugin was \
built without debug info, or on Windows its .pdb was not found beside the \
executable";

/* === Macros === */
/* === Statics === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    #[serde(default)]
    pub chain: Vec<String>,
    pub location: String,
    #[serde(default)]
    pub span_trace: Vec<String>,
    #[serde(default)]
    pub backtrace: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Redaction {
    root: Option<String>,
    home: Option<String>,
    user: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for Diagnostic {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.message),
                std::mem::size_of_val(&self.message),
            ),
            (
                harmos::MemorySize::memory_size(&self.chain),
                std::mem::size_of_val(&self.chain),
            ),
            (
                harmos::MemorySize::memory_size(&self.location),
                std::mem::size_of_val(&self.location),
            ),
            (
                harmos::MemorySize::memory_size(&self.span_trace),
                std::mem::size_of_val(&self.span_trace),
            ),
            (
                harmos::MemorySize::memory_size(&self.backtrace),
                std::mem::size_of_val(&self.backtrace),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl Diagnostic {
    #[track_caller]
    pub fn capture<E>(error: &E) -> Self
    where
        E: std::error::Error + ?Sized,
    {
        Self::at(Location::caller(), error)
    }

    pub fn at<E>(location: &Location<'_>, error: &E) -> Self
    where
        E: std::error::Error + ?Sized,
    {
        let mut chain = Vec::new();
        let mut source = error.source();
        while let Some(cause) = source {
            chain.push(cause.to_string());
            source = cause.source();
        }

        Self {
            message: error.to_string(),
            chain,
            location: located(location),
            span_trace: spans(),
            backtrace: backtrace(),
        }
    }

    #[track_caller]
    pub fn of(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            chain: Vec::new(),
            location: located(Location::caller()),
            span_trace: spans(),
            backtrace: backtrace(),
        }
    }

    #[must_use]
    pub fn context(mut self, message: impl Into<String>) -> Self {
        self.chain
            .insert(0, std::mem::replace(&mut self.message, message.into()));
        self
    }

    #[must_use]
    pub fn summary(&self) -> String {
        let mut summary = self.message.clone();
        for cause in &self.chain {
            summary.push_str(": ");
            summary.push_str(cause);
        }

        summary
    }

    #[must_use]
    pub fn redacted(&self) -> Self {
        self.redact(&Redaction::environment())
    }

    fn redact(&self, redaction: &Redaction) -> Self {
        Self {
            message: redaction.apply(&self.message),
            chain: self
                .chain
                .iter()
                .map(|cause| redaction.apply(cause))
                .collect(),
            location: redaction.apply(&self.location),
            span_trace: self
                .span_trace
                .iter()
                .map(|span| redaction.apply(span))
                .collect(),
            backtrace: redaction.apply(&self.backtrace),
        }
    }
}

impl Redaction {
    fn environment() -> Self {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()
            .filter(|home| !home.is_empty());
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .ok()
            .filter(|user| !user.is_empty())
            .or_else(|| {
                home.as_deref()
                    .and_then(|home| home.rsplit(['/', '\\']).next())
                    .map(str::to_owned)
            });

        Self {
            root: std::env::current_dir()
                .ok()
                .map(|root| root.display().to_string()),
            home,
            user,
        }
    }

    fn apply(&self, text: &str) -> String {
        let mut redacted = text.to_owned();
        if let Some(root) = self.root.as_deref().filter(|root| !root.is_empty()) {
            redacted = redacted.replace(&format!("{root}/"), "");
            redacted = redacted.replace(&format!("{root}\\"), "");
        }
        if let Some(home) = self.home.as_deref().filter(|home| !home.is_empty()) {
            redacted = redacted.replace(home, HOME);
        }
        if let Some(user) = self.user.as_deref().filter(|user| !user.is_empty()) {
            redacted = redacted.replace(user, USER);
        }

        redacted
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "error: {}", self.message)?;
        for cause in &self.chain {
            writeln!(formatter, "  caused by: {cause}")?;
        }
        if !self.location.is_empty() {
            writeln!(formatter, "  at {}", self.location)?;
        }
        for span in &self.span_trace {
            writeln!(formatter, "  in {span}")?;
        }
        if !self.backtrace.trim().is_empty() {
            writeln!(formatter, "  backtrace:")?;
            for line in self.backtrace.lines() {
                writeln!(formatter, "  {line}")?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for Diagnostic {}

/* === Public Functions === */
#[must_use]
pub fn redacted(text: &str) -> String {
    Redaction::environment().apply(text)
}

#[track_caller]
pub fn captured<T, E>(answered: Result<T, E>) -> Result<T, Diagnostic>
where
    E: std::error::Error,
{
    match answered {
        Ok(value) => Ok(value),
        Err(error) => Err(Diagnostic::at(Location::caller(), &error)),
    }
}

/* === Private Functions === */
fn located(location: &Location<'_>) -> String {
    format!("{}:{}", location.file(), location.line())
}

fn backtrace() -> String {
    let captured = Backtrace::force_capture().to_string();
    let frames = frames(&captured)
        .skip_while(|frame| frame.contains(CAPTURE) || frame.contains("dokime::diagnostics"))
        .collect::<Vec<_>>();
    if frames.is_empty() {
        return captured;
    }

    let mut rendered = collapsed(&frames);
    if wholly_unresolved(&frames) {
        rendered.push(UNRESOLVED_HINT.to_owned());
    }
    rendered.join("\n")
}

fn collapsed(frames: &[&str]) -> Vec<String> {
    let mut rendered = Vec::new();
    let mut index = 0;
    let mut position = 0;
    while position < frames.len() {
        let run = frames[position..]
            .iter()
            .take_while(|frame| !has_symbol(frame))
            .count();
        if run >= 2 {
            rendered.push(format!("… {run} frames without symbols"));
            position += run;
            continue;
        }
        rendered.push(renumbered(index, frames[position]));
        index += 1;
        position += 1;
    }
    rendered
}

fn wholly_unresolved(frames: &[&str]) -> bool {
    frames.iter().any(|frame| !has_symbol(frame)) && frames.iter().all(|frame| !rust_path(frame))
}

fn has_symbol(frame: &str) -> bool {
    symbol(frame) != UNKNOWN
}

fn rust_path(frame: &str) -> bool {
    symbol(frame).contains("::")
}

fn symbol(frame: &str) -> &str {
    let header = frame.split('\n').next().unwrap_or(frame);
    match header.find(": ") {
        Some(colon) => header[colon + 2..].trim(),
        None => header.trim(),
    }
}

fn frames(backtrace: &str) -> impl Iterator<Item = &str> {
    let mut starts = backtrace
        .char_indices()
        .filter(|(at, _)| *at == 0 || backtrace.as_bytes()[at - 1] == b'\n')
        .filter(|(at, _)| numbered(&backtrace[*at..]))
        .map(|(at, _)| at)
        .collect::<Vec<_>>();
    starts.push(backtrace.len());

    (0..starts.len().saturating_sub(1))
        .map(move |frame| backtrace[starts[frame]..starts[frame + 1]].trim_end_matches('\n'))
}

fn numbered(line: &str) -> bool {
    let digits = line.trim_start_matches(' ');
    let rest = digits.trim_start_matches(|character: char| character.is_ascii_digit());

    digits.len() != rest.len() && rest.starts_with(": ")
}

fn renumbered(index: usize, frame: &str) -> String {
    let leading = frame.len() - frame.trim_start_matches(' ').len();
    let Some(colon) = frame.find(": ") else {
        return frame.to_owned();
    };

    format!("{:leading$}{index}{}", "", &frame[colon..])
}

fn plain(styled: &str) -> String {
    let mut plain = String::with_capacity(styled.len());
    let mut characters = styled.chars();
    while let Some(character) = characters.next() {
        if character != '\u{1b}' {
            plain.push(character);
            continue;
        }
        // CSI: everything up to and including the first final byte.
        for escaped in characters.by_ref() {
            if escaped.is_ascii_alphabetic() {
                break;
            }
        }
    }

    plain
}

fn spans() -> Vec<String> {
    let mut spans = Vec::new();
    SpanTrace::capture().with_spans(|metadata, fields| {
        let fields = plain(fields);
        spans.push(if fields.is_empty() {
            metadata.name().to_owned()
        } else {
            format!("{}  {fields}", metadata.name())
        });

        true
    });

    spans
}

/* === Tests === */
