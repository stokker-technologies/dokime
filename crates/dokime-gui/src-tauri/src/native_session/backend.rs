/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{path::PathBuf, time::Duration};

use super::{BackendDiagnostics, DesktopSessionError as Error};
use dokime_server::host;

const DESKTOP_READINESS_TIMEOUT_MS: u64 = 60_000;
const DESKTOP_STARTUP_DIAGNOSTIC_BYTES: u64 = 65_536;

const REDACTED: &str = "<redacted>";
const BEARER: &str = "bearer";
const AUTHORIZATION: &str = "authorization";
const SECRET_KEYS: [&str; 4] = [AUTHORIZATION, "token", "secret", "password"];

#[derive(Clone, Debug)]
pub struct Backend {
    arguments: host::Args,
    log: Option<PathBuf>,
}

impl Backend {
    pub fn new(arguments: host::Args) -> Self {
        Self { arguments, log: None }
    }

    #[must_use]
    pub fn with_log(mut self, path: Option<PathBuf>) -> Self {
        self.log = path;
        self
    }
}

pub async fn start(backend: &Backend, on_phase: impl FnMut(&str) + Send) -> Result<(host::Session, String), Error> {
    let launch = async {
        let config = backend.arguments.configuration()?;
        let network = backend.arguments.network.config()?;
        host::Session::start(&backend.arguments, config, network, on_phase).await
    };
    let started = tokio::time::timeout(Duration::from_millis(DESKTOP_READINESS_TIMEOUT_MS), launch).await;
    let result = match started {
        Ok(result) => result.map_err(|error| {
            let message = error.to_string();
            redact(&message[..message.floor_char_boundary(DESKTOP_STARTUP_DIAGNOSTIC_BYTES as usize)])
        }),
        Err(_) => Err("The local server did not report readiness in time.".to_owned()),
    };
    match result {
        Ok(session) => {
            let endpoint = session.endpoint();
            Ok((session, endpoint))
        }
        Err(message) => {
            tracing::error!(%message, "embedded server startup failed");
            Err(Error::Startup {
                message: message.clone(),
                diagnostics: Some(Box::new(BackendDiagnostics {
                    cause: message,
                    log: backend.log.as_ref().map(|path| path.display().to_string()),
                })),
            })
        }
    }
}

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
            // An authorization value carries a scheme and a secret.
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
        // secret) extend the mask already written.
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

#[cfg(test)]
#[test]
fn diagnostics_redact_secrets() {
    let message = redact("Authorization: Bearer fixture-access-key\npassword='fixture-password' token=fixture-token");
    for secret in ["fixture-access-key", "fixture-password", "fixture-token"] {
        assert!(!message.contains(secret));
    }
    assert!(message.contains(REDACTED));
}
