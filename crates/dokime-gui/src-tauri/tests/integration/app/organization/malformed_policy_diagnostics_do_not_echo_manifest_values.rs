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
fn malformed_policy_diagnostics_do_not_echo_manifest_values() {
    let root = tempfile::tempdir().unwrap();
    let manifest = format!("{MANIFEST_TEXT}\n[updates]\nmode = 'private-value-42'\n");
    std::fs::write(root.path().join(MANIFEST), manifest).unwrap();

    let error = resolve_directory(Some(root.path())).unwrap_err();

    assert!(error.contains("organization update policy is invalid"));
    assert!(!error.contains("private-value-42"));
}
