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
fn failed_promotion_restores_the_active_pack() {
    let root = tempfile::tempdir().unwrap();
    let active = root.path().join(ACTIVE);
    std::fs::create_dir(&active).unwrap();
    std::fs::write(active.join(MANIFEST), MANIFEST_TEXT).unwrap();

    let pending = root.path().join(PENDING);
    assert!(replace_pack(&pending, &active).is_err(), "a missing pending pack cannot replace the active pack");

    assert_eq!(std::fs::read_to_string(active.join(MANIFEST)).unwrap(), MANIFEST_TEXT);
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1, "rollback removes only its empty backup directory");
}
