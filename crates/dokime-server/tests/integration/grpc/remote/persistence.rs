/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[allow(dead_code)]
#[path = "../../../../src/grpc/remote/store.rs"]
mod store;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use crate::grpc::error::Error;
use store::{Document, Identity, SERVER_REMOTE_STORE_BYTES, read, write};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn persistent_identity_and_verifier() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("remote/access.json");
    let mut document = read(&path, Document::initial().settings).unwrap();
    document.settings.certificate_names = vec!["localhost".into(), "127.0.0.1".into()];
    document.identity = Some(Identity::generate(document.settings.certificate_names.clone()).unwrap());
    document.verifier = Some([17; 32]);
    write(&path, &document).unwrap();
    let before = std::fs::read(&path).unwrap();
    let restored = read(&path, Document::initial().settings).unwrap();
    assert_eq!(restored.verifier, document.verifier);
    assert_eq!(restored.identity.as_ref().unwrap().certificate, document.identity.as_ref().unwrap().certificate);
    assert!(restored.identity.as_ref().unwrap().private_key == document.identity.as_ref().unwrap().private_key);
    let security = serde_json::from_slice::<serde_json::Value>(&before).unwrap();
    assert!(security.get("access_key").is_none());
    assert!(security.get("settings").is_none());
    write(&path, &restored).unwrap();
    assert!(std::fs::read(&path).unwrap() == before);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(std::fs::metadata(path.parent().unwrap()).unwrap().permissions().mode() & 0o777, 0o700);
    }
}

#[test]
fn invalid_explicit_security_is_not_reset() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("access.json");
    std::fs::write(&path, b"invalid private material").unwrap();
    let error = read(&path, Document::initial().settings).err().unwrap();
    assert!(!error.to_string().contains("private material"));
    assert_eq!(std::fs::read(&path).unwrap(), b"invalid private material");
    assert!(read(root.path(), Document::initial().settings).is_err());
    std::fs::write(&path, vec![b'a'; usize::try_from(SERVER_REMOTE_STORE_BYTES).unwrap() + 1]).unwrap();
    assert!(read(&path, Document::initial().settings).is_err());
}

#[test]
fn certificate_replacement_requires_new_trust() {
    let first = Identity::generate(vec!["localhost".into()]).unwrap();
    let second = Identity::generate(vec!["localhost".into(), "192.168.1.21".into()]).unwrap();
    assert_ne!(first.certificate, second.certificate);
    assert_ne!(first.private_key, second.private_key);
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("access.json");
    let mut document = Document::initial();
    document.identity = Some(second);
    write(&path, &document).unwrap();
    let restored = read(&path, document.settings.clone()).unwrap();
    assert_eq!(restored.identity.unwrap().certificate, document.identity.as_ref().unwrap().certificate);
}
