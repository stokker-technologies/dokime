/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;
use std::io;

#[cfg(feature = "native")]
use std::io::{Read, Seek, SeekFrom};

#[cfg(feature = "native")]
use std::path::Path;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

#[cfg(feature = "native")]
use std::time::SystemTime;

use super::stream::{ArtifactSink, validated};

#[cfg(feature = "native")]
use super::stream::{inferred_mime, transfer_identity};

#[cfg(feature = "native")]
use crate::transport::{ArtifactChunk, MAX_ARTIFACT_BYTES, MAX_ARTIFACT_CHUNK_BYTES};

#[cfg(feature = "native")]
const GROWTH_FLOOR: u64 = 64 * 1024;

const ARTIFACTS: &str = "artifacts";

// The registration name of the whole artifacts folder.
const ROOT: &str = "";

pub(super) struct Directories {
    area: PathBuf,

    #[cfg(feature = "native")]
    token: Arc<str>,

    #[cfg(feature = "native")]
    sink: Arc<dyn ArtifactSink>,

    #[cfg(feature = "native")]
    ordinals: Arc<AtomicU64>,

    registered: BTreeMap<String, PathBuf>,

    #[cfg(feature = "native")]
    watched: BTreeMap<String, Watched>,
}

#[cfg(feature = "native")]
struct Watched {
    transfer_id: String,
    mime: String,
    sequence: u64,
    sent: u64,
    modified: Option<SystemTime>,
    restart: bool,
    refused: bool,
}

impl Directories {
    pub(super) fn new(token: Arc<str>, sink: Arc<dyn ArtifactSink>, ordinals: Arc<AtomicU64>) -> Self {
        #[cfg(not(feature = "native"))]
        let _ = (sink, ordinals);

        Self {
            area: working_area(&token),
            #[cfg(feature = "native")]
            token,
            #[cfg(feature = "native")]
            sink,
            #[cfg(feature = "native")]
            ordinals,
            registered: BTreeMap::new(),
            #[cfg(feature = "native")]
            watched: BTreeMap::new(),
        }
    }

    pub(super) fn directory(&mut self, name: &str) -> io::Result<PathBuf> {
        let name = validated(name)?;
        if let Some(path) = self.registered.get(&name) {
            return Ok(path.clone());
        }
        let path = self.area.join(ARTIFACTS).join(&name);
        std::fs::create_dir_all(&path)?;
        self.registered.insert(name, path.clone());

        Ok(path)
    }

    // The whole artifacts folder, registered under no prefix: a file anywhere
    // in it is filed under its path relative to it. Every named directory
    // lies inside it, so once it is registered only it is walked.
    pub(super) fn root(&mut self) -> io::Result<PathBuf> {
        if let Some(path) = self.registered.get(ROOT) {
            return Ok(path.clone());
        }
        let path = self.area.join(ARTIFACTS);
        std::fs::create_dir_all(&path)?;
        self.registered.insert(ROOT.to_owned(), path.clone());

        Ok(path)
    }

    #[cfg(feature = "native")]
    pub(super) fn scan(&mut self) -> Vec<String> {
        self.walk(false)
    }

    #[cfg(feature = "native")]
    pub(super) fn sweep(&mut self) -> Vec<String> {
        if self.registered.is_empty() {
            return Vec::new();
        }
        let mut notes = self.walk(true);
        if let Err(error) = std::fs::remove_dir_all(&self.area) {
            notes.push(format!("the execution's artifact directory {} could not be removed: {error}", self.area.display()));
        }
        self.registered.clear();

        notes
    }

    #[cfg(feature = "native")]
    fn walk(&mut self, sweeping: bool) -> Vec<String> {
        let mut notes = Vec::new();
        let registered = std::mem::take(&mut self.registered);
        let walked = match registered.get_key_value(ROOT) {
            Some(root) => vec![root],
            None => registered.iter().collect(),
        };
        for (name, root) in walked {
            for (artifact, path) in files(name, root, &mut notes) {
                self.file(artifact, &path, sweeping, &mut notes);
            }
        }
        self.registered = registered;

        notes
    }

    #[cfg(feature = "native")]
    fn file(&mut self, name: String, path: &Path, sweeping: bool, notes: &mut Vec<String>) {
        if self.watched.get(&name).is_some_and(|watched| watched.refused) {
            return;
        }
        // A file that vanished between the walk and the stat is skipped rather
        // than reported: a tool writing through a temporary name does that on
        // purpose, and what it leaves behind is what the sweep files.
        let Ok(metadata) = std::fs::metadata(path) else {
            return;
        };
        if !metadata.is_file() {
            return;
        }
        if !self.watched.contains_key(&name) {
            let mime = match inferred_mime(&name) {
                Ok(mime) => mime,
                Err(error) => {
                    notes.push(format!("{error}; it stays in the directory"));
                    self.refuse(name);
                    return;
                }
            };
            let transfer_id = transfer_identity(&self.token, &self.ordinals);
            self.watched.insert(name.clone(), Watched::new(transfer_id, mime));
        }
        let watched = self.watched.get_mut(&name).expect("the file was just inserted");
        let length = metadata.len();
        if length > MAX_ARTIFACT_BYTES {
            notes.push(format!(
                "artifact `{name}` is {length} bytes, past the {MAX_ARTIFACT_BYTES} byte limit one artifact may carry; it is not filed as evidence"
            ));
            if watched.sequence > 0 {
                // Pieces already handed over cannot be recalled, so the host is
                // told the content will never be completed rather than left
                // holding a prefix it would store as the whole file.
                let chunk = watched.abandon(&name);
                self.sink.push(chunk);
            }
            self.refuse(name);
            return;
        }

        let modified = metadata.modified().ok();
        let restart = watched.restart || length < watched.sent || (length == watched.sent && modified != watched.modified);
        let from = if restart { 0 } else { watched.sent };
        let pending = length.saturating_sub(from);
        if !sweeping && pending < GROWTH_FLOOR {
            // Nothing is sent, so nothing is recorded as sent either — except
            // that the content is known to have changed, which the next send
            // has to know even if it happens three ticks from now.
            watched.restart = restart;
            return;
        }

        match read(path, from, pending, sweeping) {
            Ok(pieces) => {
                let mut replace = restart;
                let count = pieces.len();
                for (index, bytes) in pieces.into_iter().enumerate() {
                    let last = sweeping && index + 1 == count;
                    let chunk = watched.piece(&name, bytes, last, replace);
                    replace = false;
                    self.sink.push(chunk);
                }
                watched.sent = length;
                watched.modified = modified;
                watched.restart = false;
            }
            // A read that failed leaves the file exactly as it was before: the
            // bytes are still on disk, and the next tick — or the sweep — sends
            // them. Only a file that never becomes readable goes unreported,
            // and the sweep's own failure is what reports that.
            Err(error) if sweeping => notes.push(format!("artifact `{name}` could not be read from the execution's directory: {error}")),
            Err(_) => {}
        }
    }

    #[cfg(feature = "native")]
    fn refuse(&mut self, name: String) {
        self.watched.entry(name).or_insert_with(|| Watched::new(String::new(), String::new())).refused = true;
    }
}

#[cfg(feature = "native")]
impl Watched {
    fn new(transfer_id: String, mime: String) -> Self {
        Self {
            transfer_id,
            mime,
            sequence: 0,
            sent: 0,
            modified: None,
            restart: false,
            refused: false,
        }
    }

    fn piece(&mut self, name: &str, bytes: Vec<u8>, last: bool, replace: bool) -> ArtifactChunk {
        let sequence = self.sequence;
        self.sequence += 1;

        ArtifactChunk {
            transfer_id: self.transfer_id.clone(),
            name: name.to_owned(),
            mime: self.mime.clone(),
            sequence,
            bytes,
            last,
            aborted: false,
            replace: replace && sequence > 0,
        }
    }

    fn abandon(&mut self, name: &str) -> ArtifactChunk {
        ArtifactChunk {
            aborted: true,
            ..self.piece(name, Vec::new(), true, false)
        }
    }
}

fn working_area(token: &str) -> PathBuf {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    let readable = token.chars().filter(|character| character.is_ascii_alphanumeric() || *character == '-').take(32).collect::<String>();

    std::env::temp_dir().join(format!("dokime-{readable}-{:016x}", hasher.finish()))
}

#[cfg(feature = "native")]
fn files(name: &str, root: &Path, notes: &mut Vec<String>) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let mut pending = vec![(name.to_owned(), root.to_owned())];
    while let Some((prefix, directory)) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        let mut children = Vec::new();
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let Some(segment) = entry.file_name().to_str().map(str::to_owned) else {
                notes.push(format!("the file {} is not named in UTF-8 and cannot be filed as an artifact", entry.path().display()));
                continue;
            };
            let artifact = if prefix.is_empty() { segment } else { format!("{prefix}/{segment}") };
            if kind.is_dir() {
                children.push((artifact, entry.path()));
            } else if kind.is_file() {
                found.push((artifact, entry.path()));
            }
        }
        children.sort_by(|left, right| right.0.cmp(&left.0));
        pending.extend(children);
    }
    found.sort_by(|left, right| left.0.cmp(&right.0));

    found
}

#[cfg(feature = "native")]
fn read(path: &Path, from: u64, pending: u64, sweeping: bool) -> io::Result<Vec<Vec<u8>>> {
    let mut file = std::fs::File::open(path)?;
    file.seek(SeekFrom::Start(from))?;
    let mut pieces = Vec::new();
    let mut remaining = pending;
    while remaining > 0 {
        let wanted = remaining.min(MAX_ARTIFACT_CHUNK_BYTES as u64) as usize;
        let mut bytes = vec![0; wanted];
        file.read_exact(&mut bytes)?;
        remaining -= wanted as u64;
        pieces.push(bytes);
    }
    // Every transfer ends with exactly one terminal piece, including the one
    // for a file that is empty or that had already handed over everything it
    // has: an artifact the run produced and an artifact it forgot to produce
    // are different facts, and only that piece tells them apart.
    if sweeping && pieces.is_empty() {
        pieces.push(Vec::new());
    }

    Ok(pieces)
}
