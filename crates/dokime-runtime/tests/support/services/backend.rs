/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::storage::{ScanPage, SchemaKey, StorageBackend, StorageCursor, StoredRecord, query::RecordSelection};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub async fn for_each_record(backend: &impl StorageBackend, at: &StorageCursor, schema: SchemaKey, mut read: impl FnMut(&StoredRecord)) {
    let selection = RecordSelection { schema, scope: None, item: None, source_start: None, source_end: None };

    let snapshot = backend.open_snapshot(at).await.unwrap();
    let mut after = 0;
    loop {
        let page = backend
            .read_records(
                &snapshot,
                &selection,
                ScanPage { after, rows: NonZeroUsize::new(16).unwrap(), bytes: NonZeroUsize::new(1024 * 1024).unwrap() },
            )
            .await
            .unwrap();
        let Some(last) = page.last() else {
            break;
        };
        after = last.position.sequence;
        for row in &page {
            read(row);
        }
    }
    backend.release_snapshot(&snapshot).await.unwrap();
}

/* === Private Functions === */

/* === Tests === */
