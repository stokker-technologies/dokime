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
fn finalize_appends_only_metadata_and_is_idempotent() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("samples.arrow");
    let batch = batch();
    let mut writer = Writer::create(&path, batch.schema()).unwrap();
    writer.append(&batch).unwrap();
    let before = std::fs::read(&path).unwrap();
    let result = writer.finish().unwrap();
    assert_eq!(writer.finish().unwrap(), result);
    let after = std::fs::read(&path).unwrap();
    assert_eq!(&after[..before.len()], before);
    assert_eq!(Sha256::digest(&after).as_slice(), result.1);
    assert!(writer.append(&batch).is_err());
}
