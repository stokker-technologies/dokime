/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Profile Conformance And Atomic Edit Scenarios

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, fs::OpenOptions, sync::Mutex};

// Third Party Imports
use serde::Deserialize;
use zeroize::Zeroizing;

// Local Imports
use dokime_client::{
    profiles::{self, Error, Secret, Store},
    secrets::{Error as SecretError, SecretStore},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
struct Secrets {
    values: Mutex<BTreeMap<String, String>>,
    unavailable: bool,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    document: String,
    valid: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl SecretStore for Secrets {
    fn read(&self, endpoint: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
        if self.unavailable {
            return Err(SecretError::Unavailable);
        }
        Ok(self.values.lock().unwrap().get(endpoint).cloned().map(Zeroizing::new))
    }
    fn write(&self, endpoint: &str, token: &str) -> Result<(), SecretError> {
        if self.unavailable {
            return Err(SecretError::Unavailable);
        }
        self.values.lock().unwrap().insert(endpoint.to_owned(), token.to_owned());
        Ok(())
    }
    fn remove(&self, endpoint: &str) -> Result<(), SecretError> {
        if self.unavailable {
            return Err(SecretError::Unavailable);
        }
        self.values.lock().unwrap().remove(endpoint);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
#[test]
fn shared_language_conformance() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("../fixtures/profiles.json")).unwrap();
    for case in cases {
        assert_eq!(profiles::decode(case.document.as_bytes()).is_ok(), case.valid, "{}", case.name);
    }
}

#[test]
fn selection_snapshot_bound() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("../fixtures/profiles.json")).unwrap();
    let mut document = profiles::decode(cases[0].document.as_bytes()).unwrap();
    document.profiles[0].secret = Secret::Keyring;
    let selected = profiles::select(&document, Some("local"), None, Some("unknown")).unwrap();
    assert_eq!(selected.expected_backend_id.as_deref(), Some("machine-1"));
    let direct = profiles::select(&document, None, Some("http://127.0.0.1:9000"), Some("unknown")).unwrap();
    assert_eq!(direct.secret, Secret::None);
    assert!(direct.expected_backend_id.is_none());
    assert!(profiles::select(&document, Some("local"), Some("http://127.0.0.1"), None).is_err());
    assert!(profiles::select(&document, None, None, Some("unknown")).is_err());
    document.default = None;
    document.profiles.clear();
    assert_eq!(selected.target.endpoint(), "http://127.0.0.1:8765/");
    assert!(profiles::select(&document, None, None, None).is_err());
}

#[test]
fn edits_refuse_contention() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("backends.json");
    let store = Store::new(&path);
    assert!(store.load().unwrap().profiles.is_empty());
    assert!(!path.exists());
    store.update(|_| Ok(())).unwrap();
    let original = std::fs::read(&path).unwrap();
    let lock = OpenOptions::new().read(true).write(true).open(directory.path().join("backends.json.lock")).unwrap();
    lock.lock().unwrap();
    assert!(matches!(store.update(|_| Ok(())), Err(Error::Busy)));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    drop(lock);
    assert!(
        store
            .update(|document| {
                document.version = 2;
                Ok(())
            })
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    store
        .update(|document| {
            document.default = None;
            Ok(())
        })
        .unwrap();
    assert_eq!(store.load().unwrap().version, 1);
}

#[test]
fn secrets_keep_profile() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("../fixtures/profiles.json")).unwrap();
    let original = profiles::decode(cases[0].document.as_bytes()).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("backends.json"));
    store
        .update(|document| {
            *document = original;
            Ok(())
        })
        .unwrap();
    let selection = profiles::select(&store.load().unwrap(), None, None, None).unwrap();
    let secrets = Secrets::default();
    // A Default Edit Does Not Redirect An In-Flight Login.
    store
        .update(|document| {
            document.default = None;
            Ok(())
        })
        .unwrap();
    store.remember(&selection, "private-token", &secrets).unwrap();
    assert_eq!(profiles::get(&store.load().unwrap(), "local").unwrap().secret, Secret::Keyring);
    assert_eq!(secrets.read(selection.target.endpoint()).unwrap().unwrap().as_str(), "private-token");
    let bytes = std::fs::read(store.path()).unwrap();
    assert!(!String::from_utf8(bytes).unwrap().contains("private-token"));
    secrets.remove(selection.target.endpoint()).unwrap();
    assert!(secrets.read(selection.target.endpoint()).unwrap().is_none());
    // A Target Edit Refuses To Save For The New Association.
    store
        .update(|document| {
            document.profiles[0].endpoint = "http://127.0.0.1:9000".into();
            Ok(())
        })
        .unwrap();
    let before = std::fs::read(store.path()).unwrap();
    assert!(store.remember(&selection, "other-token", &secrets).is_err());
    assert!(secrets.values.lock().unwrap().is_empty());
    assert_eq!(std::fs::read(store.path()).unwrap(), before);
    let current = profiles::select(&store.load().unwrap(), Some("local"), None, None).unwrap();
    let unavailable = Secrets { unavailable: true, ..Secrets::default() };
    assert!(matches!(store.remember(&current, "private-token", &unavailable), Err(Error::Secrets(SecretError::Unavailable))));
    assert_eq!(std::fs::read(store.path()).unwrap(), before);
}
