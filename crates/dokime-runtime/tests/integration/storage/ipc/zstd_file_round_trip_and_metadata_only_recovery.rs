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
fn zstd_file_round_trip_and_metadata_only_recovery() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("samples.arrow");
    let batch = batch();
    let mut writer = Writer::create(&path, batch.schema()).unwrap();
    let first = writer.append(&batch).unwrap();
    let second = writer.append(&batch).unwrap();
    let checkpoint = writer.checkpoint();
    assert_eq!(read_batch(&path, batch.schema(), &second).unwrap(), batch);
    // Simulate A Crash During The Next Write, Before A Footer Exists.
    drop(writer);
    let mut file = OpenOptions::new().append(true).open(&path).unwrap();
    file.write_all(b"uncommitted partial batch").unwrap();
    drop(file);
    let before = std::fs::read(&path).unwrap()[..checkpoint.end() as usize].to_vec();
    let (length, digest) = recover(&path, batch.schema(), &[first, second], &checkpoint).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes.len() as u64, length);
    assert_eq!(Sha256::digest(&bytes).as_slice(), digest);
    assert_eq!(&bytes[..before.len()], before);
    let reader = arrow_ipc::reader::FileReader::try_new(File::open(&path).unwrap(), None).unwrap();
    let batches = reader.collect::<std::result::Result<Vec<_>, _>>().unwrap();
    assert_eq!(batches, vec![batch.clone(), batch]);
}
