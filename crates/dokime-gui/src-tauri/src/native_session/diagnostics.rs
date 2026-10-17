/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Owned backend stderr capture.
 *
 * A desktop launch has no console, so an inherited stderr vanishes and a
 * failed start leaves nothing to review. The pump reads stderr line by line,
 * redacts credentials, streams every line into one per-launch log file and
 * keeps a bounded tail for the startup error. Stdout stays the readiness
 * channel; nothing here reads it.
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
// Third Party Imports
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::ChildStderr,
    task::JoinHandle,
};
// Local Imports

/* === Constants === */
const LINE_LIMIT: u64 = 16 * 1024;
const TAIL_BYTES: usize = 64 * 1024;
const TAIL_LINES: usize = 200;

const KEEP_LOGS: usize = 20;
const LOG_PREFIX: &str = "backend-stderr.";
const LOG_SUFFIX: &str = ".log";

// A grandchild that inherited stderr can hold the pipe open after the backend
// itself exited; the tail gathered by then is still the useful part.
const SETTLE_TIMEOUT: Duration = Duration::from_secs(2);

const REDACTED: &str = "<redacted>";
const BEARER: &str = "bearer";
const AUTHORIZATION: &str = "authorization";
const SECRET_KEYS: [&str; 4] = [AUTHORIZATION, "token", "secret", "password"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Capture {
    tail: Arc<Mutex<Tail>>,
    log: Option<PathBuf>,
    pump: JoinHandle<()>,
}

#[derive(Default)]
struct Tail {
    lines: VecDeque<String>,
    bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Capture {
    pub(super) async fn start(stderr: ChildStderr, logs: Option<&Path>, pid: Option<u32>) -> Self {
        // Open The Per-Launch Log
        let opened = match logs {
            Some(directory) => match open_log(directory, pid).await {
                Ok(opened) => Some(opened),
                Err(error) => {
                    // The tail still reaches the startup error without a file.
                    tracing::warn!(
                        directory = %directory.display(),
                        %error,
                        "dokime backend stderr log could not be opened"
                    );
                    None
                }
            },
            None => None,
        };
        let (file, log) = opened.unzip();

        // Start Draining Before Readiness
        // A backend that fills an undrained stderr pipe blocks before it can
        // print its ready line.
        let tail = Arc::new(Mutex::new(Tail::default()));
        let pump = tokio::spawn(pump(BufReader::new(stderr), file, Arc::clone(&tail)));
        Self { tail, log, pump }
    }

    pub(super) async fn settle(mut self) -> (String, Option<PathBuf>) {
        if !self.pump.is_finished() {
            let _ = tokio::time::timeout(SETTLE_TIMEOUT, &mut self.pump).await;
        }
        let tail = self.tail.lock().expect("backend diagnostics poisoned").text();
        (tail, self.log)
    }
}

impl Tail {
    fn push(&mut self, line: String) {
        self.bytes += line.len() + 1;
        self.lines.push_back(line);
        while self.lines.len() > TAIL_LINES || (self.bytes > TAIL_BYTES && self.lines.len() > 1) {
            if let Some(evicted) = self.lines.pop_front() {
                self.bytes -= evicted.len() + 1;
            }
        }
    }

    fn text(&self) -> String {
        self.lines.iter().map(String::as_str).collect::<Vec<_>>().join("\n")
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn pump(mut stderr: BufReader<ChildStderr>, mut file: Option<File>, tail: Arc<Mutex<Tail>>) {
    let mut bytes = Vec::new();
    loop {
        // Read One Bounded Line
        bytes.clear();
        match (&mut stderr).take(LINE_LIMIT).read_until(b'\n', &mut bytes).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        // Redact Before Anything Is Kept
        let line = redact(String::from_utf8_lossy(&bytes).trim_end_matches(['\r', '\n']));

        // Keep The Line
        if let Some(writer) = file.as_mut() {
            let written = async {
                writer.write_all(line.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await
            }
            .await;
            if let Err(error) = written {
                tracing::warn!(%error, "dokime backend stderr log stopped accepting writes");
                file = None;
            }
        }
        tail.lock().expect("backend diagnostics poisoned").push(line);
    }
}

async fn open_log(directory: &Path, pid: Option<u32>) -> std::io::Result<(File, PathBuf)> {
    tokio::fs::create_dir_all(directory).await?;
    if let Err(error) = prune(directory).await {
        tracing::warn!(
            directory = %directory.display(),
            %error,
            "older dokime backend stderr logs could not be pruned"
        );
    }
    // A fixed-width UTC stamp sorts launches chronologically by name; the
    // process id separates launches within the same second.
    let now = time::OffsetDateTime::now_utc();
    let path = directory.join(format!(
        "{LOG_PREFIX}{:04}{:02}{:02}T{:02}{:02}{:02}Z.{}{LOG_SUFFIX}",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        pid.unwrap_or_default(),
    ));
    let file = File::create(&path).await?;
    Ok((file, path))
}

async fn prune(directory: &Path) -> std::io::Result<()> {
    let mut entries = tokio::fs::read_dir(directory).await?;
    let mut logs = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_name().to_str().is_some_and(|name| name.starts_with(LOG_PREFIX) && name.ends_with(LOG_SUFFIX)) {
            logs.push(entry.path());
        }
    }
    logs.sort();
    // Leave room for the log this launch is about to create.
    let excess = logs.len().saturating_sub(KEEP_LOGS - 1);
    for stale in &logs[..excess] {
        tokio::fs::remove_file(stale).await?;
    }
    Ok(())
}

/// Masks bearer credentials and the values of authorization, token, secret
/// and password keys, whether written as `key: value`, `key=value` or JSON.
fn redact(line: &str) -> String {
    // Locate The Secrets
    // ASCII lowercasing keeps byte offsets, and every boundary below sits next
    // to an ASCII byte, so the offsets stay valid on the original line.
    let lower = line.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut masks = Vec::new();
    for (start, _) in lower.match_indices(BEARER) {
        let after = start + BEARER.len();
        let value = skip(bytes, after, |byte| matches!(byte, b' ' | b'\t'));
        if value > after {
            masks.push((value, value_end(bytes, value)));
        }
    }
    for key in SECRET_KEYS {
        for (start, _) in lower.match_indices(key) {
            let separator = skip(bytes, start + key.len(), quote_or_blank);
            if !matches!(bytes.get(separator), Some(b':' | b'=')) {
                continue;
            }
            let value = skip(bytes, separator + 1, quote_or_blank);
            // An authorization value carries a scheme and a credential.
            let end = if key == AUTHORIZATION { quoted_end(bytes, value) } else { value_end(bytes, value) };
            masks.push((value, end));
        }
    }

    // Rebuild The Line
    masks.retain(|(start, end)| start < end);
    masks.sort_unstable();
    let mut redacted = String::with_capacity(line.len());
    let mut cursor = 0;
    for (start, end) in masks {
        // Overlapping secrets (an authorization header holding a bearer
        // credential) extend the mask already written.
        if start < cursor {
            cursor = cursor.max(end);
            continue;
        }
        redacted.push_str(&line[cursor..start]);
        redacted.push_str(REDACTED);
        cursor = end;
    }
    redacted.push_str(&line[cursor..]);
    redacted
}

fn skip(bytes: &[u8], from: usize, mut skipped: impl FnMut(u8) -> bool) -> usize {
    bytes[from..].iter().position(|byte| !skipped(*byte)).map_or(bytes.len(), |offset| from + offset)
}

fn value_end(bytes: &[u8], from: usize) -> usize {
    skip(bytes, from, |byte| !byte.is_ascii_whitespace() && !matches!(byte, b'"' | b'\'' | b',' | b';' | b'&' | b'}' | b']'))
}

fn quoted_end(bytes: &[u8], from: usize) -> usize {
    skip(bytes, from, |byte| !matches!(byte, b'"' | b'\''))
}

fn quote_or_blank(byte: u8) -> bool {
    matches!(byte, b'"' | b'\'' | b' ' | b'\t')
}

/* === Tests === */
