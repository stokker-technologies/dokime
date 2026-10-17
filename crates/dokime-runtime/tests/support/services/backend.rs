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
pub async fn records(backend: &impl StorageBackend, at: &StorageCursor, schema: SchemaKey) -> Vec<StoredRecord> {
    let selection = RecordSelection { schema, scope: None, item: None, source_start: None, source_end: None };
    let snapshot = backend.open_snapshot(at).await.unwrap();
    let mut records = Vec::new();
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
        records.extend(page);
    }
    backend.release_snapshot(&snapshot).await.unwrap();
    records
}

/* === Private Functions === */

/* === Tests === */
