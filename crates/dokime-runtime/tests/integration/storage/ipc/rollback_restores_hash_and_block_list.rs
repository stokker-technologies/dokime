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
fn rollback_restores_hash_and_block_list() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("samples.arrow");
    let batch = batch();
    let mut writer = Writer::create(&path, batch.schema()).unwrap();
    let checkpoint = writer.checkpoint();
    writer.append(&batch).unwrap();
    writer.rollback(&checkpoint).unwrap();
    writer.append(&batch).unwrap();
    let (length, hash) = writer.finish().unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(length, bytes.len() as u64);
    assert_eq!(hash, <[u8; 32]>::from(Sha256::digest(&bytes)));
    let reader = arrow_ipc::reader::FileReader::try_new(File::open(&path).unwrap(), None).unwrap();
    assert_eq!(reader.num_batches(), 1);
}
