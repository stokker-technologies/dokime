/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[path = "../../../src/commands/id_file.rs"]
mod implementation;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use implementation::Args;

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
fn truncates_before_mutation_and_writes_one_identity_per_line() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("ids.txt");
    std::fs::write(&path, b"stale identity\n").unwrap();
    let args = Args::destination(path.clone());

    let file = args.open().unwrap().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"", "an interrupted command must not leave stale identities");
    file.write(["first", "second"]).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "first\nsecond\n");

    assert!(Args::default().open().unwrap().is_none());
    let missing = Args::destination(directory.path().join("missing").join("ids.txt"));
    assert!(matches!(missing.open(), Err(crate::Error::Input(_))));
}
