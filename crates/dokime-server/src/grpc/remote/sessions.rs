/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Persistent Log Of Remote Sessions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use dokime::domain::model::remote::{RemoteSession, RemoteSessionOutcome, RemoteSessionPage};

/* === Constants === */

// The log keeps the most recent sessions only.
pub(crate) const CAPACITY: usize = 500;
// A peer quiet this long has gone; transport drops are not observable here.
pub(crate) const IDLE_MILLIS: u64 = 60_000;
const MAXIMUM_PAGE: usize = 100;
const MAXIMUM_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Remote sessions as the authenticator sees them, persisted beside the listener security.
#[derive(Default)]
pub(crate) struct SessionLog {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    // Oldest first.
    sessions: Vec<RemoteSession>,
    next: u64,
    path: Option<PathBuf>,
    dirty: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u32,
    next: u64,
    sessions: Vec<RemoteSession>,
}

/* === Enums === */

pub(crate) enum Outcome<'a> {
    Admitted,
    Rejected(&'a str),
}

/* === Traits === */

/* === Implementations === */

impl SessionLog {
    // Loads the persisted log and persists to it from now on. An unreadable log is kept on
    // disk untouched and this process records in memory only.
    pub(crate) fn attach(&self, path: PathBuf) {
        let mut state = self.lock();
        match read(&path) {
            Ok(Some(document)) => {
                state.next = document.next;
                // Sessions open in the previous process ended with it.
                state.sessions = document.sessions;
                for session in &mut state.sessions {
                    session.disconnected_at.get_or_insert(session.last_seen_at);
                }
                state.path = Some(path);
            }
            Ok(None) => state.path = Some(path),
            Err(error) => tracing::error!(%error, path = %path.display(), "remote session log is unreadable; recording in memory only"),
        }
    }

    pub(crate) fn record(&self, peer: SocketAddr, principal: &str, outcome: Outcome<'_>) {
        self.record_at(peer, principal, outcome, now());
    }

    pub(crate) fn list(&self, cursor: Option<&str>, limit: u32) -> Result<RemoteSessionPage, &'static str> {
        self.list_at(cursor, limit, now())
    }

    // Writes the log when it changed since the last write.
    pub(crate) fn flush(&self) {
        let (path, document) = {
            let mut state = self.lock();
            let Some(path) = state.path.clone().filter(|_| state.dirty) else { return };
            state.dirty = false;
            (path, Document { version: 1, next: state.next, sessions: state.sessions.clone() })
        };
        if let Err(error) = write(&path, &document) {
            tracing::warn!(%error, "remote session log could not be written");
            self.lock().dirty = true;
        }
    }

    fn record_at(&self, peer: SocketAddr, principal: &str, outcome: Outcome<'_>, now: u64) {
        let (kind, rejection) = match outcome {
            Outcome::Admitted => (RemoteSessionOutcome::Admitted, None),
            Outcome::Rejected(reason) => (RemoteSessionOutcome::Rejected, Some(reason.to_owned())),
        };
        // Refusals Group By Host, So Reconnecting From Fresh Ports Cannot Evict Admitted History.
        let peer = if rejection.is_some() { peer.ip().to_string() } else { peer.to_string() };
        let mut state = self.lock();
        state.close_idle(now);
        state.dirty = true;
        let open = state.sessions.iter_mut().rev().find(|session| {
            session.disconnected_at.is_none() && session.peer_address == peer && session.principal == principal && session.outcome == kind
        });
        if let Some(session) = open {
            session.last_seen_at = now;
            session.calls += 1;
            if rejection.is_some() {
                session.rejection = rejection;
            }
            return;
        }
        state.next += 1;
        let id = state.next.to_string();
        state.sessions.push(RemoteSession {
            id,
            peer_address: peer,
            principal: principal.to_owned(),
            connected_at: now,
            last_seen_at: now,
            disconnected_at: None,
            calls: 1,
            outcome: kind,
            rejection,
        });
        let excess = state.sessions.len().saturating_sub(CAPACITY);
        state.sessions.drain(..excess);
    }

    fn list_at(&self, cursor: Option<&str>, limit: u32, now: u64) -> Result<RemoteSessionPage, &'static str> {
        let before = match cursor {
            Some(cursor) => cursor.parse::<u64>().map_err(|_| "invalid remote session cursor")?,
            None => u64::MAX,
        };
        let limit = (limit as usize).clamp(1, MAXIMUM_PAGE);
        let mut state = self.lock();
        state.close_idle(now);
        let mut older = state.sessions.iter().rev().filter(|session| session.id.parse::<u64>().is_ok_and(|id| id < before));
        let sessions: Vec<RemoteSession> = older.by_ref().take(limit).cloned().collect();
        let next_cursor = older.next().and(sessions.last().map(|session| session.id.clone()));
        Ok(RemoteSessionPage { sessions, next_cursor })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl State {
    fn close_idle(&mut self, now: u64) {
        for session in &mut self.sessions {
            if session.disconnected_at.is_none() && now.saturating_sub(session.last_seen_at) >= IDLE_MILLIS {
                session.disconnected_at = Some(session.last_seen_at);
                self.dirty = true;
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_millis() as u64)
}

fn read(path: &Path) -> Result<Option<Document>, String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
        Ok(metadata) if !metadata.file_type().is_file() => return Err("not a regular file".into()),
        Ok(_) => {}
    }
    let bytes = harmos::filesystem::input::read(path, MAXIMUM_BYTES).map_err(|error| error.to_string())?;
    let document: Document = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if document.version != 1 {
        return Err(format!("unsupported version {}", document.version));
    }
    Ok(Some(document))
}

fn write(path: &Path, document: &Document) -> Result<(), String> {
    let bytes = serde_json::to_vec(document).map_err(|error| error.to_string())?;
    if let Some(parent) = path.parent() {
        super::store::private_directory(parent).map_err(|error| error.to_string())?;
    }
    harmos::filesystem::publication::write(path, &bytes).map_err(|error| error.to_string())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(port: u16) -> SocketAddr {
        SocketAddr::from(([192, 168, 1, 20], port))
    }

    #[test]
    fn calls_from_one_peer_share_a_session_until_idle() {
        let log = SessionLog::default();
        log.record_at(peer(5000), "access-key", Outcome::Admitted, 1_000);
        log.record_at(peer(5000), "access-key", Outcome::Admitted, 2_000);
        log.record_at(peer(5000), "anonymous", Outcome::Rejected("invalid credentials"), 2_500);
        let page = log.list_at(None, 20, 3_000).unwrap();
        assert_eq!(page.sessions.len(), 2);
        let rejected = &page.sessions[0];
        assert_eq!((rejected.outcome, rejected.rejection.as_deref()), (RemoteSessionOutcome::Rejected, Some("invalid credentials")));
        let admitted = &page.sessions[1];
        assert_eq!((admitted.calls, admitted.connected_at, admitted.last_seen_at), (2, 1_000, 2_000));
        assert_eq!(admitted.disconnected_at, None);
        log.record_at(peer(5001), "anonymous", Outcome::Rejected("invalid credentials"), 2_600);
        let grouped = log.list_at(None, 20, 3_000).unwrap();
        assert_eq!(grouped.sessions.len(), 2, "refusals from fresh ports share the host's session");
        assert_eq!((grouped.sessions[0].peer_address.as_str(), grouped.sessions[0].calls), ("192.168.1.20", 2));
        let later = log.list_at(None, 20, 2_000 + IDLE_MILLIS).unwrap();
        assert_eq!(later.sessions[1].disconnected_at, Some(2_000));
        log.record_at(peer(5000), "access-key", Outcome::Admitted, 3_000 + IDLE_MILLIS);
        assert_eq!(log.list_at(None, 20, 3_000 + IDLE_MILLIS).unwrap().sessions.len(), 3);
    }

    #[test]
    fn the_log_keeps_the_newest_sessions_and_pages_newest_first() {
        let log = SessionLog::default();
        for port in 0..(CAPACITY as u16 + 25) {
            log.record_at(peer(port), "access-key", Outcome::Admitted, 1_000);
        }
        let mut seen = Vec::new();
        let mut cursor = None;
        loop {
            let page = log.list_at(cursor.as_deref(), 20, 1_000).unwrap();
            assert!(page.sessions.len() <= 20);
            seen.extend(page.sessions.iter().map(|session| session.peer_address.clone()));
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        assert_eq!(seen.len(), CAPACITY);
        assert_eq!(seen.first(), Some(&peer(CAPACITY as u16 + 24).to_string()));
        assert_eq!(seen.last(), Some(&peer(25).to_string()));
        assert!(log.list_at(Some("not-a-cursor"), 20, 1_000).is_err());
    }

    #[test]
    fn the_log_survives_a_restart_with_open_sessions_closed() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("remote").join("sessions.json");
        let log = SessionLog::default();
        log.attach(path.clone());
        log.record_at(peer(7000), "anonymous", Outcome::Admitted, 5_000);
        log.flush();
        let restored = SessionLog::default();
        restored.attach(path.clone());
        let page = restored.list_at(None, 20, 6_000).unwrap();
        assert_eq!(page.sessions.len(), 1);
        assert_eq!(page.sessions[0].disconnected_at, Some(5_000));
        restored.record_at(peer(7001), "anonymous", Outcome::Admitted, 6_000);
        assert_eq!(restored.list_at(None, 20, 6_000).unwrap().sessions[0].id, "2");

        std::fs::write(&path, b"not json").unwrap();
        let unreadable = SessionLog::default();
        unreadable.attach(path.clone());
        unreadable.record_at(peer(7002), "anonymous", Outcome::Admitted, 7_000);
        unreadable.flush();
        assert_eq!(std::fs::read(&path).unwrap(), b"not json");
    }
}
