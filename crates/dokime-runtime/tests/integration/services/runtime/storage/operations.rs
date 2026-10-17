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
use harmos::operations::{StreamSettings, stream};
use harmos::storage::ObjectCatalog;
use harmos::storage::operations::{Query, QueryStorage, QueryStorageRequest};
use harmos::storage::query::{ObjectSelection, ObjectVersions, Query as QueryRequest, QueryOutput, RowSlice, Source};

// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::storage::objects::{Objects, suite::Suite};

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

#[tokio::test]
async fn storage_operations_view() {
    let root = tempdir().unwrap();
    let config = config(root.path());
    // An in-process storage client claims the runtime's read budget, as report
    // queries do: one nested object schema expands into far more Arrow fields
    // than any single message-sized budget holds.
    let budget = config.storage.budget.read_memory_bytes.min(NonZeroUsize::new(u32::MAX as usize).unwrap());
    let (mut runtime, handle) = started(config).await;
    let maximum = NonZeroUsize::new(64 * 1024).unwrap();
    let suite = Suite::new("fixture.storage.operations", &empty_suite()).unwrap();
    let encoded = Objects::encode(&suite, maximum).unwrap();
    let receipt = handle.storage().write_object(&suite).await.unwrap();
    handle.storage().flush().await.unwrap();

    let info = handle.operations().call_as::<QueryStorage>(QueryStorageRequest).await.unwrap();
    assert_eq!(info.saved, handle.storage().cursor_saved());
    assert_eq!(info.accepted, handle.storage().cursor_accepted());
    assert!(info.saved.covers(&receipt.position).unwrap());

    let (output, mut input) = stream::<QueryOutput>(StreamSettings { items: NonZeroUsize::MIN, bytes: budget }).unwrap();
    let request = QueryRequest {
        at: info.saved.clone(),
        source: Source::Objects(ObjectSelection {
            schema: Some(encoded.schema.clone()),
            key: Some(encoded.item.clone()),
            versions: ObjectVersions::Latest,
        }),
        transforms: Vec::new(),
        slice: RowSlice::default(),
    };
    let query = handle.operations().submit_endpoints::<Query, _>(request, output).unwrap();
    let result = async {
        let Some(schema) = input.recv().await else {
            panic!("query ended before its schema: {:?}", query.await);
        };
        assert!(matches!(&*schema, QueryOutput::Schema(_)));
        drop(schema);
        let row = input.recv().await.expect("saved suite is returned");
        let QueryOutput::Batch(batch) = &*row else {
            panic!("expected an Arrow suite batch");
        };
        assert_eq!(batch.as_record_batch().num_rows(), 1);
        assert!(batch.as_record_batch().column_by_name("payload").is_some());
        drop(row);
        assert!(input.recv().await.is_none());
        query.await.unwrap()
    }
    .await;
    assert!(result.at.covers(&receipt.position).unwrap());
    assert_eq!(result.rows, 1);

    drop(handle);
    runtime.shutdown().await.unwrap();
}
