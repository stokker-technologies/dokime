/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Authenticated Transfer Preserves Complete Catalog Generations Under Failure.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc,
    atomic::{AtomicIsize, Ordering},
};

// Third Party Imports
use zeroize::Zeroizing;

// Local Imports
use super::{Backend, Change, SecretStore, SecretStoreError, SecretValue, TestStore, transfer};
use crate::config::{RuntimeConfig, Settings};
use dokime_domain::model::secrets::{
    catalog::Metadata,
    transfer::{Action, Choice, MAX_CIPHERTEXT_BYTES},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct FaultStore {
    inner: TestStore,
    remaining: AtomicIsize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Backend for FaultStore {
    fn set(&self, slot: &str, value: &str) -> Result<(), SecretStoreError> {
        if self.remaining.load(Ordering::SeqCst) == 0 {
            return Err(SecretStoreError::Unavailable);
        }
        self.remaining.fetch_sub(1, Ordering::SeqCst);
        self.inner.set(slot, value)
    }
    fn get(&self, slot: &str) -> Result<Option<SecretValue>, SecretStoreError> {
        self.inner.get(slot)
    }
    fn remove(&self, slot: &str) -> Result<(), SecretStoreError> {
        self.inner.remove(slot)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn fixture() -> (tempfile::TempDir, SecretStore, Arc<FaultStore>) {
    let root = tempfile::tempdir().unwrap();
    let settings = Settings::load(root.path().join("application")).unwrap();
    let metadata = crate::Metadata::new(crate::Identifier::new("secret.fixture").unwrap(), "Secret Host", None);
    let config = RuntimeConfig::new(root.path(), metadata);
    let backend = Arc::new(FaultStore { inner: TestStore::default(), remaining: AtomicIsize::new(-1) });
    let mut store = SecretStore::from_config(&config, settings).unwrap();
    store.backend = backend.clone();
    (root, store, backend)
}

fn change(name: &str, value: &str) -> Change {
    Change { metadata: Metadata { name: name.into(), label: name.into(), description: None }, value: Some(SecretValue::new(value.into())) }
}

fn password() -> Zeroizing<String> {
    Zeroizing::new("correct horse battery staple".into())
}

fn choices() -> Vec<Choice> {
    vec![Choice { name: "alpha".into(), action: Action::Replace }, Choice { name: "beta".into(), action: Action::Skip }]
}

/* === Tests === */

#[test]
fn authenticated_secret_transfer_is_bounded_and_atomic() {
    let (_source_root, source, _) = fixture();
    source.apply(vec![change("alpha", "exported-alpha-value"), change("beta", "exported-beta-value")], &[], 0).unwrap();
    let names = vec!["alpha".into(), "beta".into()];
    let archive = source.export(&names, password(), 1).unwrap();
    assert!(!archive.ciphertext.windows(20).any(|bytes| bytes == b"exported-alpha-value"));

    let (root, destination, backend) = fixture();
    destination.apply(vec![change("alpha", "old-alpha"), change("beta", "retained-beta")], &[], 0).unwrap();
    let preview = destination.preview(&archive.ciphertext, password()).unwrap();
    assert_eq!(preview.generation, 1);
    assert_eq!(preview.host_id, destination.host);
    assert_ne!(preview.host_id, source.host);
    assert!(preview.entries.iter().all(|entry| entry.exists));
    assert!(!serde_json::to_string(&preview).unwrap().contains("exported-alpha-value"));
    destination.import(&archive.ciphertext, password(), &choices(), preview.generation).unwrap();
    assert_eq!(destination.get_secret("alpha").unwrap().unwrap().expose(), "exported-alpha-value");
    assert_eq!(destination.get_secret("beta").unwrap().unwrap().expose(), "retained-beta");

    let committed = destination.catalog();
    let slots = backend.inner.values.lock().unwrap().len();
    let wrong = Zeroizing::new("wrong password".into());
    assert!(destination.import(&archive.ciphertext, wrong, &choices(), 2).is_err());
    let mut corrupted = archive.ciphertext.clone();
    *corrupted.last_mut().unwrap() ^= 1;
    let mut appended = archive.ciphertext.clone();
    appended.push(0);
    let truncated = &archive.ciphertext[..archive.ciphertext.len() - 1];
    for bytes in [&corrupted[..], &appended[..], truncated] {
        assert!(destination.import(bytes, password(), &choices(), 2).is_err());
        assert_eq!(destination.catalog(), committed);
        assert_eq!(backend.inner.values.lock().unwrap().len(), slots);
    }

    let invalid =
        br#"{"format_version":1,"entries":[{"metadata":{"name":"../outside","label":"Outside","description":null},"value":"hidden"}]}"#;
    let encrypted = transfer::encrypt(invalid, &password()).unwrap();
    assert!(destination.import(&encrypted, password(), &[Choice { name: "../outside".into(), action: Action::Replace }], 2).is_err());
    assert!(destination.preview(&vec![0; MAX_CIPHERTEXT_BYTES + 1], password()).is_err());
    assert!(destination.import(&archive.ciphertext, password(), &choices(), 1).is_err());
    assert_eq!(destination.catalog(), committed);
    assert_eq!(backend.inner.values.lock().unwrap().len(), slots);

    backend.remaining.store(1, Ordering::SeqCst);
    let replace = vec![Choice { name: "alpha".into(), action: Action::Replace }, Choice { name: "beta".into(), action: Action::Replace }];
    assert!(destination.import(&archive.ciphertext, password(), &replace, 2).is_err());
    assert_eq!(destination.catalog(), committed);
    assert_eq!(backend.inner.values.lock().unwrap().len(), slots);
    assert_eq!(destination.get_secret("beta").unwrap().unwrap().expose(), "retained-beta");

    backend.remaining.store(-1, Ordering::SeqCst);
    let settings_path = root.path().join("application/settings.json");
    let saved = std::fs::read(&settings_path).unwrap();
    assert!(!String::from_utf8_lossy(&saved).contains("exported-alpha-value"));
    std::fs::write(&settings_path, b"invalid settings").unwrap();
    assert!(destination.apply(vec![change("alpha", "unconfirmed-value")], &[], 2).is_err());
    std::fs::write(&settings_path, &saved).unwrap();
    assert_eq!(destination.catalog(), committed);
    assert_eq!(destination.get_secret("alpha").unwrap().unwrap().expose(), "exported-alpha-value");
    assert_eq!(backend.inner.values.lock().unwrap().len(), slots + 1);

    let guard = destination.transfer.lock().unwrap();
    assert!(destination.export(&names, password(), 2).is_err());
    drop(guard);
    let request = dokime_domain::operations::secrets::Export { names, password: password().to_string(), generation: 2 };
    assert!(!format!("{request:?}").contains(password().as_str()));
    assert_ne!(super::SHARED_SERVICE, super::LEGACY_SERVICE);
}
