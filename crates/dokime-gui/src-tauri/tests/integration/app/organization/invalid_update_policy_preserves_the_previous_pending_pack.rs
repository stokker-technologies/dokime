/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use super::*;

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
fn invalid_update_policy_preserves_the_previous_pending_pack() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join("organization");
    let archive_path = root.path().join("pack.zip");
    let valid = format!("{MANIFEST_TEXT}\n[updates]\nmode = 'disabled'\n");
    std::fs::write(&archive_path, archive(&[(MANIFEST, valid.as_bytes()), ("logos/mark.svg", LOGO)])).unwrap();
    stage(&store, &archive_path).unwrap();

    let invalid = valid.replace("mode = 'disabled'", "mode = 'manual'\nsource = 'missing'");
    std::fs::write(&archive_path, archive(&[(MANIFEST, invalid.as_bytes()), ("logos/mark.svg", LOGO)])).unwrap();
    assert!(stage(&store, &archive_path).unwrap_err().contains("does not exist"));
    let booted = boot(&store);
    let script = booted.clone();
    assert_eq!(script["pack"]["updates"]["mode"], "disabled");
    let updates: policy::UpdatePolicy = serde_json::from_value(booted["pack"]["updates"].clone()).unwrap();
    assert_eq!(updates.mode, policy::Mode::Disabled);
    assert_eq!(std::fs::read_to_string(store.join(ACTIVE).join(MANIFEST)).unwrap(), valid);
}
