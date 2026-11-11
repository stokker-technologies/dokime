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
fn invalid_pending_pack_preserves_the_last_valid_active_pack() {
    let root = tempfile::tempdir().unwrap();
    let organization = root.path().join("organization");
    let archive_path = root.path().join("pack.zip");
    std::fs::write(&archive_path, archive(&[(MANIFEST, MANIFEST_TEXT.as_bytes()), ("logos/mark.svg", LOGO)])).unwrap();
    stage(&organization, &archive_path).unwrap();
    assert_eq!(boot(&organization)["pack"]["label"], "Stokker Technologies");
    stage(&organization, &archive_path).unwrap();
    std::fs::write(organization.join(PENDING).join(MANIFEST), "invalid [").unwrap();

    let retained = boot(&organization);

    assert_eq!(retained["pack"]["label"], "Stokker Technologies");
    assert!(retained["error"].as_str().unwrap().contains("pending organization pack"));
    assert_eq!(std::fs::read_to_string(organization.join(ACTIVE).join(MANIFEST)).unwrap(), MANIFEST_TEXT);
    assert_eq!(std::fs::read_to_string(organization.join(PENDING).join(MANIFEST)).unwrap(), "invalid [");
}
