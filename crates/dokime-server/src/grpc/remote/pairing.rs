/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Independent Pairing Verifiers And Bounded Client Lifecycles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
};

// Third Party Imports
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

// Local Imports
use super::{Controller, Error, store};
use dokime::domain::model::remote::{CreatedPairing, PairedClient, PairedClientPage, PairingFile, RemoteEndpoint};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Registry {
    entries: RwLock<Vec<Entry>>,
    revision: AtomicU64,
    saved: AtomicU64,
}

struct Entry {
    record: store::Client,
    revoked: CancellationToken,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for Registry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Registry").finish_non_exhaustive()
    }
}

impl Registry {
    pub(super) fn new(records: Vec<store::Client>) -> Self {
        let entries = records
            .into_iter()
            .map(|record| Entry {
                record,
                revoked: CancellationToken::new(),
            })
            .collect();
        Self {
            entries: RwLock::new(entries),
            revision: AtomicU64::new(0),
            saved: AtomicU64::new(0),
        }
    }

    pub(super) fn records(&self) -> Vec<store::Client> {
        self.entries.read().unwrap_or_else(std::sync::PoisonError::into_inner).iter().map(|entry| entry.record.clone()).collect()
    }

    pub(crate) fn authenticate(&self, digest: &[u8; 32]) -> Result<(String, CancellationToken), tonic::Status> {
        self.authenticate_at(digest, unix_millis())
    }

    fn authenticate_at(&self, digest: &[u8; 32], now: u64) -> Result<(String, CancellationToken), tonic::Status> {
        let mut entries = self.entries.write().map_err(|_| tonic::Status::unavailable("paired clients unavailable"))?;
        let mut matched = None;
        // Compare Every Verifier Without Early Exit.
        for (index, entry) in entries.iter().enumerate() {
            if bool::from(digest.ct_eq(&entry.record.verifier)) {
                matched = Some(index);
            }
        }
        let entry = matched.map(|index| &mut entries[index]).ok_or_else(|| tonic::Status::unauthenticated("invalid credentials"))?;
        if entry.record.metadata.last_seen_at.is_none_or(|previous| now.saturating_sub(previous) >= 5_000) {
            entry.record.metadata.last_seen_at = Some(now);
            self.revision.fetch_add(1, Ordering::Release);
        }
        Ok((entry.record.metadata.identifier.clone(), entry.revoked.clone()))
    }

    pub(super) fn list(&self, cursor: Option<&str>, limit: u32) -> Result<PairedClientPage, Error> {
        let limit = match limit {
            0 => 100,
            1..=500 => limit as usize,
            _ => return Err(Error::Configuration("client page limit exceeds 500")),
        };
        let after = cursor.map(decode_cursor).transpose()?;
        let mut clients: Vec<_> = self.records().into_iter().map(|record| record.metadata).collect();
        clients.sort_unstable_by(|left, right| right.created_at.cmp(&left.created_at).then(left.identifier.cmp(&right.identifier)));
        if let Some((created_at, identifier)) = after {
            clients.retain(|client| client.created_at < created_at || (client.created_at == created_at && client.identifier > identifier));
        }
        let more = clients.len() > limit;
        clients.truncate(limit);
        let next_cursor = if more { clients.last().map(encode_cursor) } else { None };
        Ok(PairedClientPage { clients, next_cursor })
    }
}

impl Controller {
    pub(super) async fn create_pairing(self: &Arc<Self>, label: String) -> Result<CreatedPairing, Error> {
        // Persist And Publish Even When The Native Caller Cancels.
        let controller = self.clone();
        tokio::spawn(async move {
            {
                controller.change(super::Change::Refresh).await?;
                controller.create(label).await
            }
        })
        .await?
    }

    async fn create(&self, label: String) -> Result<CreatedPairing, Error> {
        let mut state = self.state.lock().await;
        if state.closing {
            return Err(Error::Configuration("remote listener is closing"));
        }
        if state.error.is_some() {
            return Err(Error::Configuration("resolve the listener restart failure before creating a pairing"));
        }
        let bound = state.bound.as_ref().ok_or(Error::Configuration("remote listener must be active before creating a pairing"))?;
        let server = state.server.as_ref().ok_or(Error::Configuration("remote listener is not accepting calls"))?;
        let identity = self.runtime.watch_state().latest().state().identity.node.node_id.to_string();
        let mut pairing = PairingFile {
            version: 1,
            endpoints: endpoints(
                store::host_endpoints(server.address(), super::SERVER_REMOTE_NAMES as usize),
                server.address().port(),
                bound.tls.is_some(),
            ),
            backend_identifier: identity,
            label,
            ca_certificate_pem: if bound.tls.is_some() {
                Some(
                    state
                        .document
                        .identity
                        .as_ref()
                        .ok_or(Error::Configuration("active certificate is unavailable"))?
                        .ca_certificate
                        .clone(),
                )
            } else {
                None
            },
            access_key: None,
        };
        pairing.validate().map_err(Error::Listener)?;
        if bound.tls.is_some() {
            let identity = state.document.identity.as_ref().ok_or(Error::Configuration("active certificate is unavailable"))?;
            for endpoint in &pairing.endpoints {
                let authority = endpoint.split_once("://").expect("validated endpoint").1;
                let host = authority.rsplit_once(':').expect("validated port").0.trim_matches(['[', ']']);
                if !identity.names.iter().any(|name| name.eq_ignore_ascii_case(host)) {
                    return Err(Error::Configuration("remote endpoint is not covered by the active TLS certificate"));
                }
            }
        }
        if !matches!(bound.policy, crate::grpc::access::Policy::Clients(_)) {
            return Ok(CreatedPairing { client: None, pairing });
        }
        let mut candidate = state.document.clone();
        candidate.clients = self.clients.records();
        if candidate.clients.len() >= 512 {
            return Err(Error::Configuration("paired client limit reached; revoke an unused client"));
        }
        let random = Zeroizing::new(rand::random::<[u8; 32]>());
        let mut key = Zeroizing::new(String::with_capacity(64));
        use std::fmt::Write as _;
        for byte in random.iter() {
            write!(key, "{byte:02x}").expect("writing to a string succeeds");
        }
        let metadata = PairedClient {
            identifier: uuid::Uuid::now_v7().to_string(),
            label: pairing.label.clone(),
            created_at: unix_millis(),
            last_seen_at: None,
        };
        let record = store::Client {
            metadata: metadata.clone(),
            verifier: Sha256::digest(key.as_bytes()).into(),
        };
        candidate.clients.push(record.clone());
        self.write_security(&candidate).await?;
        self.clients.entries.write().unwrap_or_else(std::sync::PoisonError::into_inner).push(Entry {
            record,
            revoked: CancellationToken::new(),
        });
        state.document = candidate;
        pairing.access_key = Some(key.to_string());
        Ok(CreatedPairing { client: Some(metadata), pairing })
    }

    pub(super) async fn rename_client(self: &Arc<Self>, identifier: String, label: String) -> Result<(), Error> {
        let controller = self.clone();
        tokio::spawn(async move {
            let mut state = controller.state.lock().await;
            if state.closing {
                return Err(Error::Configuration("remote listener is closing"));
            }
            let mut candidate = state.document.clone();
            candidate.clients = controller.clients.records();
            let client = candidate
                .clients
                .iter_mut()
                .find(|client| client.metadata.identifier == identifier)
                .ok_or(Error::Configuration("paired client does not exist"))?;
            client.metadata.label = label.clone();
            store::validate_clients(&candidate.clients)?;
            controller.write_security(&candidate).await?;
            let mut entries = controller.clients.entries.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(entry) = entries.iter_mut().find(|entry| entry.record.metadata.identifier == identifier) {
                entry.record.metadata.label = label;
            }
            state.document = candidate;
            Ok(())
        })
        .await?
    }

    pub(super) async fn revoke_client(self: &Arc<Self>, identifier: String) -> Result<(), Error> {
        let controller = self.clone();
        tokio::spawn(async move {
            let mut state = controller.state.lock().await;
            if state.closing {
                return Err(Error::Configuration("remote listener is closing"));
            }
            let mut candidate = state.document.clone();
            candidate.clients = controller.clients.records();
            let previous = candidate.clients.len();
            candidate.clients.retain(|client| client.metadata.identifier != identifier);
            if candidate.clients.len() == previous {
                return Ok(());
            }
            controller.write_security(&candidate).await?;
            let mut entries = controller.clients.entries.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            entries.retain(|entry| {
                if entry.record.metadata.identifier != identifier {
                    return true;
                }
                entry.revoked.cancel();
                false
            });
            state.document = candidate;
            Ok(())
        })
        .await?
    }

    pub(super) async fn flush_clients(&self) -> Result<(), Error> {
        let mut state = self.state.lock().await;
        let revision = self.clients.revision.load(Ordering::Acquire);
        if revision == self.clients.saved.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut candidate = state.document.clone();
        candidate.clients = self.clients.records();
        self.write_security(&candidate).await?;
        state.document = candidate;
        self.clients.saved.store(revision, Ordering::Release);
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn endpoints(hosts: Vec<RemoteEndpoint>, port: u16, tls: bool) -> Vec<String> {
    hosts
        .into_iter()
        .map(|endpoint| {
            let name = endpoint.address;
            let host = if name.contains(':') { format!("[{name}]") } else { name };
            format!("{}://{host}:{port}", if tls { "https" } else { "http" })
        })
        .collect()
}

/* === Private Functions === */

fn unix_millis() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().min(u64::MAX as u128) as u64
}

fn encode_cursor(client: &PairedClient) -> String {
    let text = format!("{}:{}", client.created_at, client.identifier);
    text.bytes().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_cursor(cursor: &str) -> Result<(u64, String), Error> {
    let invalid = || Error::Configuration("invalid paired client cursor");
    if cursor.len() > 128 || !cursor.len().is_multiple_of(2) || !cursor.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid());
    }
    let bytes: Vec<u8> = (0..cursor.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&cursor[index..index + 2], 16).map_err(|_| invalid()))
        .collect::<Result<_, _>>()?;
    let text = String::from_utf8(bytes).map_err(|_| invalid())?;
    let (timestamp, identifier) = text.split_once(':').ok_or_else(invalid)?;
    uuid::Uuid::parse_str(identifier).map_err(|_| invalid())?;
    Ok((timestamp.parse().map_err(|_| invalid())?, identifier.to_owned()))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn record(identifier: &str, created_at: u64, key: &str) -> store::Client {
        store::Client {
            metadata: PairedClient {
                identifier: identifier.into(),
                label: "Lab".into(),
                created_at,
                last_seen_at: None,
            },
            verifier: Sha256::digest(key.as_bytes()).into(),
        }
    }

    #[test]
    fn independent_verifiers_and_revocation_tokens() {
        let first = record("00000000-0000-4000-8000-000000000001", 1, "first-key");
        let second = record("00000000-0000-4000-8000-000000000002", 2, "second-key");
        let registry = Registry::new(vec![first.clone(), second.clone()]);
        let (identifier, first_token) = registry.authenticate_at(&first.verifier, 1_000).unwrap();
        let (_, second_token) = registry.authenticate_at(&second.verifier, 1_000).unwrap();
        assert_eq!(identifier, first.metadata.identifier);
        let revision = registry.revision.load(Ordering::Acquire);
        registry.authenticate_at(&first.verifier, 1_000).unwrap();
        assert_eq!(registry.revision.load(Ordering::Acquire), revision);
        assert!(registry.records()[0].metadata.last_seen_at.is_some());
        registry.entries.write().unwrap().retain(|entry| {
            if entry.record.metadata.identifier != identifier {
                return true;
            }
            entry.revoked.cancel();
            false
        });
        assert!(first_token.is_cancelled());
        assert!(!second_token.is_cancelled());
        assert!(registry.authenticate_at(&first.verifier, 1_000).is_err());
        assert!(registry.authenticate_at(&second.verifier, 1_000).is_ok());
        let reopened = Registry::new(registry.records());
        assert!(reopened.authenticate_at(&first.verifier, 1_000).is_err());
        assert!(reopened.authenticate_at(&second.verifier, 1_000).is_ok());
    }

    #[test]
    fn client_pages_have_exclusive_stable_cursors() {
        let first = record("00000000-0000-4000-8000-000000000001", 2, "one");
        let second = record("00000000-0000-4000-8000-000000000002", 2, "two");
        let third = record("00000000-0000-4000-8000-000000000003", 1, "three");
        let registry = Registry::new(vec![third, second, first]);
        let page = registry.list(None, 1).unwrap();
        assert!(page.clients[0].identifier.ends_with('1'));
        let next = registry.list(page.next_cursor.as_deref(), 1).unwrap();
        assert!(next.clients[0].identifier.ends_with('2'));
        let last = registry.list(next.next_cursor.as_deref(), 1).unwrap();
        assert!(last.clients[0].identifier.ends_with('3'));
        assert!(last.next_cursor.is_none());
        assert_eq!(registry.list(None, 0).unwrap().clients.len(), 3);
        assert!(registry.list(None, 501).is_err());
        assert!(registry.list(Some("invalid"), 1).is_err());
    }
}
