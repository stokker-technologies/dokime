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
fn uncataloged_first_flush_can_resume_from_its_verified_header() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("samples.arrow");
    let batch = batch();
    let mut writer = Writer::create(&path, batch.schema()).unwrap();
    let header = writer.checkpoint();
    writer.append(&batch).unwrap();
    drop(writer);
    let mut writer = Writer::recover_empty(&path, batch.schema()).unwrap();
    assert_eq!(writer.checkpoint().end(), header.end());
    writer.append(&batch).unwrap();
    writer.finish().unwrap();
    drop(writer);
    let reader = arrow_ipc::reader::FileReader::try_new(File::open(&path).unwrap(), None).unwrap();
    assert_eq!(reader.num_batches(), 1);
    assert_eq!(reader.collect::<std::result::Result<Vec<_>, _>>().unwrap(), [batch]);
}
