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
use harmos::storage::operations::{
    ObjectResult, ObjectSelection, QueryStorage, QueryStorageRequest, ReadObjects,
    ReadObjectsRequest,
};

// Local Imports
use crate::support::prelude::*;
use dokime::storage::objects::{Objects, suite::Suite};

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

pub(crate) async fn storage_operations_view() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = booted(root.path()).await;
    let maximum = NonZeroUsize::new(64 * 1024).unwrap();
    let suite = Suite::new("fixture.storage.operations", &empty_suite()).unwrap();
    let encoded = Objects::encode(&suite, maximum).unwrap();
    let receipt = handle.storage().write_object(&suite).await.unwrap();
    handle.storage().flush().await.unwrap();

    let info = handle
        .operations()
        .call_as::<QueryStorage>(QueryStorageRequest)
        .await
        .unwrap();
    assert_eq!(info.saved, handle.storage().cursor_saved());
    assert_eq!(info.accepted, handle.storage().cursor_accepted());
    assert!(info.saved.covers(&receipt.position).unwrap());

    let (output, mut input) = stream::<ObjectResult>(StreamSettings {
        items: NonZeroUsize::MIN,
        bytes: maximum,
    })
    .unwrap();
    let request = ReadObjectsRequest {
        selection: ObjectSelection {
            schema: Some(encoded.schema.clone()),
            key: Some(encoded.item.clone()),
            ..Default::default()
        },
        fields: None,
    };
    let query = handle
        .operations()
        .submit_endpoints::<ReadObjects, _>(request, output)
        .unwrap();
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        let row = input.recv().await.expect("saved suite is returned");
        let ObjectResult::Value(object) = &*row else {
            panic!("expected the native suite object");
        };
        assert_eq!(object.position, receipt.position);
        assert_eq!(object.version, receipt.version);
        assert_eq!(object.value, encoded);
        assert_eq!(
            Objects::decode::<Suite>(&object.value, maximum).unwrap(),
            suite
        );
        drop(row);
        assert!(input.recv().await.is_none());
        query.await.unwrap()
    })
    .await
    .expect("bounded query completes");
    assert!(result.covers(&receipt.position).unwrap());

    drop(handle);
    runtime.shutdown().await.unwrap();
}
