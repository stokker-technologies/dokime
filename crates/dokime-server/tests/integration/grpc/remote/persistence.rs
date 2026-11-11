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
#[path = "../../../../src/grpc/remote/sessions.rs"]
mod sessions;
#[allow(dead_code)]
#[path = "../../../../src/grpc/remote/store.rs"]
mod store;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use crate::grpc::error::Error;
use sessions::{Outcome, SessionLog};
use store::{Client, Document, Identity, SERVER_REMOTE_STORE_BYTES, read, write};

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
    document.clients.push(Client {
        metadata: dokime::domain::model::remote::PairedClient {
            identifier: uuid::Uuid::now_v7().to_string(),
            label: "Lab".into(),
            created_at: 1,
            last_seen_at: None,
        },
        verifier: [17; 32],
    });
    write(&path, &document).unwrap();
    let before = std::fs::read(&path).unwrap();
    let restored = read(&path, Document::initial().settings).unwrap();
    assert_eq!(restored.clients[0].verifier, document.clients[0].verifier);
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
fn leaf_reissue_preserves_persistent_ca() {
    let mut identity = Identity::generate(vec!["localhost".into()]).unwrap();
    let ca = identity.ca_certificate.clone();
    let ca_key = identity.ca_private_key.clone();
    let leaf = identity.certificate.clone();
    identity.reissue(vec!["localhost".into(), "192.168.1.21".into()]).unwrap();
    assert_eq!(identity.ca_certificate, ca);
    assert!(identity.ca_private_key == ca_key);
    assert_ne!(identity.certificate, leaf);
    let key = rcgen::KeyPair::from_pem(&ca_key).unwrap();
    assert!(rcgen::Issuer::from_ca_cert_pem(&ca, key).is_ok());
    let root = tempfile::tempdir().unwrap();
    let mut document = Document::initial();
    document.identity = Some(identity);
    let path = root.path().join("access.json");
    write(&path, &document).unwrap();
    let mut restored = read(&path, document.settings).unwrap().identity.unwrap();
    restored.reissue(vec!["localhost".into(), "192.168.1.22".into()]).unwrap();
    assert_eq!(restored.ca_certificate, ca);
}

#[test]
fn the_log_survives_a_restart_with_open_sessions_closed() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("remote").join("sessions.json");
    let log = SessionLog::new();
    log.attach(path.clone());
    log.record_at("192.168.1.20:7000".parse().unwrap(), "anonymous", Outcome::Admitted, 5_000);
    log.flush();
    let restored = SessionLog::new();
    restored.attach(path.clone());
    let page = restored.list_at(None, 20, 6_000).unwrap();
    assert_eq!(page.sessions.len(), 1);
    assert_eq!(page.sessions[0].disconnected_at, Some(5_000));
    restored.record_at("192.168.1.20:7001".parse().unwrap(), "anonymous", Outcome::Admitted, 6_000);
    assert_eq!(restored.list_at(None, 20, 6_000).unwrap().sessions[0].id, "2");

    std::fs::write(&path, b"not json").unwrap();
    let unreadable = SessionLog::new();
    unreadable.attach(path.clone());
    unreadable.record_at("192.168.1.20:7002".parse().unwrap(), "anonymous", Outcome::Admitted, 7_000);
    unreadable.flush();
    assert_eq!(std::fs::read(&path).unwrap(), b"not json");
}
