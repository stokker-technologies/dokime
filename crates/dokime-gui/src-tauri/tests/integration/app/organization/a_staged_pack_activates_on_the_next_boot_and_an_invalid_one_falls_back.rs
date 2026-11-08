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
fn a_staged_pack_activates_on_the_next_boot_and_an_invalid_one_falls_back() {
    let root = tempfile::tempdir().unwrap();
    let organization = root.path().join("organization");
    std::fs::write(root.path().join("pack.zip"), archive(&[(MANIFEST, MANIFEST_TEXT.as_bytes()), ("logos/mark.svg", LOGO)])).unwrap();
    let summary = stage(&organization, &root.path().join("pack.zip")).unwrap();
    assert_eq!(summary["label"], "Stokker Technologies");
    let booted = boot(&organization);
    assert_eq!(booted["pack"]["label"], "Stokker Technologies");
    assert!(!organization.join(PENDING).exists() && organization.join(ACTIVE).is_dir());
    std::fs::write(organization.join(ACTIVE).join(MANIFEST), "not toml [").unwrap();
    let fallback = boot(&organization);
    assert_eq!(fallback["pack"]["label"], "Dokime");
    assert!(fallback["error"].as_str().unwrap().contains("organization.toml"));
    assert!(organization.join(ACTIVE).is_dir(), "an invalid pack stays on disk for diagnosis");
}
