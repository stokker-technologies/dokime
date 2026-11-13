/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Product Credential Persistence for Library Access

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Mutex;

// Third Party Imports
use harmos_runtime::grpc::access::{CredentialStore, Credentials, Pairing, Pending};
use prost::Message;
use zeroize::Zeroizing;

// Local Imports

/* === Constants === */

const SERVICE: &str = "dokime.access";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Store {
    storage: dokime_storage::Store,
    host_id: String,
    commit: Mutex<()>,
}

#[derive(Clone, PartialEq, Message)]
struct Snapshot {
    #[prost(uint64, tag = "1")]
    revision: u64,

    #[prost(uint64, tag = "2")]
    listener_revision: u64,

    #[prost(message, optional, tag = "3")]
    listener: Option<harmos_protocol::v1::access::ListenerValue>,

    #[prost(string, tag = "4")]
    certificate_pem: String,

    #[prost(string, tag = "5")]
    key_slot: String,

    #[prost(message, repeated, tag = "6")]
    pairings: Vec<SavedPairing>,
}

#[derive(Clone, PartialEq, Message)]
struct SavedPairing {
    #[prost(string, tag = "1")]
    id: String,

    #[prost(string, tag = "2")]
    label: String,

    #[prost(bytes = "vec", tag = "3")]
    credential_sha256: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub(super) fn new(storage: dokime_storage::Store, host_id: String) -> Self {
        Self { storage, host_id, commit: Mutex::new(()) }
    }
    fn key(&self, slot: &str) -> Result<keyring::Entry, keyring::Error> {
        keyring::Entry::new(SERVICE, &format!("{}:{slot}", self.host_id))
    }
}

impl CredentialStore for Store {
    fn load(&self) -> Pending<'_, Option<Credentials>> {
        Box::pin(async move {
            let _guard = self.commit.lock().map_err(|_| std::io::Error::other("Load Access Credentials: Store Lock Failed"))?;
            let Some(value) = self.storage.administration::<Snapshot>("access")? else { return Ok(None) };
            let private_key = Zeroizing::new(self.key(&value.key_slot)?.get_password()?);
            Ok(Some(Credentials {
                revision: value.revision,
                listener_revision: value.listener_revision,
                listener: value.listener.ok_or_else(|| std::io::Error::other("Load Access Credentials: Missing Listener"))?,
                certificate_pem: value.certificate_pem,
                private_key_pem: private_key.to_string(),
                pairings: value
                    .pairings
                    .into_iter()
                    .map(|p| Pairing {
                        id: p.id,
                        label: p.label,
                        credential_sha256: p.credential_sha256,
                    })
                    .collect(),
            }))
        })
    }
    fn commit(&self, expected_revision: u64, value: Credentials) -> Pending<'_, ()> {
        Box::pin(async move {
            let _guard = self.commit.lock().map_err(|_| std::io::Error::other("Commit Access Credentials: Store Lock Failed"))?;
            let previous = self.storage.administration::<Snapshot>("access")?;
            if previous.as_ref().map_or(0, |value| value.revision) != expected_revision {
                return Err(std::io::Error::other("Commit Access Credentials: Get Again and Retry the Revision").into());
            }
            let key_slot = uuid::Uuid::now_v7().to_string();
            let key = self.key(&key_slot)?;
            let private_key = Zeroizing::new(value.private_key_pem);
            key.set_password(&private_key)?;
            let snapshot = Snapshot {
                revision: value.revision,
                listener_revision: value.listener_revision,
                listener: Some(value.listener),
                certificate_pem: value.certificate_pem,
                key_slot,
                pairings: value
                    .pairings
                    .into_iter()
                    .map(|p| SavedPairing {
                        id: p.id,
                        label: p.label,
                        credential_sha256: p.credential_sha256,
                    })
                    .collect(),
            };
            match self.storage.compare_administration("access", previous.as_ref(), &snapshot) {
                Ok(true) => {}
                outcome => {
                    let _ = key.delete_credential();
                    return match outcome {
                        Err(error) => Err(error.into()),
                        _ => Err(std::io::Error::other("Commit Access Credentials: Get Again and Retry the Revision").into()),
                    };
                }
            }
            if let Some(previous) = previous {
                // The Saved Pointer Owns Publication; an Unreferenced Old Key Cannot Restore Access.
                if let Err(error) = self.key(&previous.key_slot).and_then(|key| key.delete_credential()) {
                    tracing::warn!(%error, "Remove Retired Access Key: Credential Store Cleanup Failed");
                }
            }
            Ok(())
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
