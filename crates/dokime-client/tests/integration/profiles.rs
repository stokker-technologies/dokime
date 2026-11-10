/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Profile Selection, Pairing and Atomic Edit Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client::{
    pairing,
    profiles::{self, Store},
};

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
fn pairing_folder_lists_files_and_keeps_credentials_native() {
    // Validate Shared Pairing Inputs At The Real File Boundary.
    let root = tempfile::tempdir().unwrap();
    let input = root.path().join("input.pair");
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!("../fixtures/pairing.json")).unwrap();
    for case in cases {
        std::fs::write(&input, case["document"].as_str().unwrap()).unwrap();
        assert_eq!(pairing::read(&input).is_ok(), case["valid"].as_bool().unwrap(), "{}", case["name"]);
    }
    std::fs::write(&input, vec![b' '; 1_048_577]).unwrap();
    assert!(pairing::read(&input).is_err());
    std::fs::write(&input, [0xff]).unwrap();
    assert!(pairing::read(&input).is_err());

    // Import A Pairing Into An Empty Folder.
    let store = Store::new(root.path().join("remotes"));
    assert!(store.load().unwrap().profiles.is_empty());
    let key = "a".repeat(64);
    let file = pairing::PairingFile {
        version: 1,
        endpoints: vec!["http://127.0.0.1:8765".into(), "http://127.0.0.1:8766".into()],
        backend_identifier: "backend".into(),
        label: "Lab".into(),
        certificate_pem: None,
        access_key: Some(key.clone()),
    };
    let mut invalid = file.clone();
    invalid.endpoints = vec!["http://0.0.0.0:8765".into()];
    assert!(store.import_pairing(&invalid, "lab").is_err());
    assert!(!store.path().exists());
    let imported = store.import_pairing(&file, "lab").unwrap();
    assert!(store.path().join("lab.pair").is_file());
    assert!(store.import_pairing(&file, "lab").is_err());
    assert!(store.import_pairing(&file, "../escape").is_err());

    // Listing Exposes Metadata; Selection Reads The Original Secret And Identity.
    let document = store.load().unwrap();
    assert_eq!(document.profiles, vec![imported]);
    assert!(!serde_json::to_string(&document).unwrap().contains(&key));
    let selection = profiles::selection(profiles::get(&document, "lab").unwrap()).unwrap();
    assert_eq!(selection.access_key.as_deref().map(String::as_str), Some(key.as_str()));
    assert_eq!(selection.expected_backend_id.as_deref(), Some("backend"));
    assert_eq!(pairing::read(&store.path().join("lab.pair")).unwrap(), file);
    assert!(pairing::read(&store.path().join("lab.json")).is_err());

    // Removing A Backend Removes Its Only Connection Input.
    store.remove("lab").unwrap();
    assert!(store.load().unwrap().profiles.is_empty());
    assert!(profiles::selection(&document.profiles[0]).is_err());

    // Refuse An Unreadable Pairing Without Resetting Its Contents.
    let path = store.path().join("broken.pair");
    std::fs::write(&path, b"invalid pairing").unwrap();
    assert!(store.load().is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"invalid pairing");
}
