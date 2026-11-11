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
fn partial_pack_inherits_assets() {
    assert!(validate_archive(&archive(&[("logos/mark.svg", LOGO)])).unwrap_err().contains("organization.toml"));
    let root = tempfile::tempdir().unwrap();
    let pack = archive(&[(MANIFEST, b"[app]\nlabel = \"Stokker Technologies\"\n")]);
    std::fs::write(root.path().join("pack.zip"), pack).unwrap();
    assert_eq!(stage(&root.path().join("organization"), &root.path().join("pack.zip")).unwrap()["label"], "Stokker Technologies");
    let escaping =
        archive(&[(MANIFEST, b"[app]\nlabel = \"Stokker Technologies\"\n[app.logos.transparent]\non_light = \"../../secret.svg\"\n")]);
    std::fs::write(root.path().join("escape.zip"), escaping).unwrap();
    assert!(stage(&root.path().join("organization"), &root.path().join("escape.zip")).unwrap_err().contains("unsafe"));
}
