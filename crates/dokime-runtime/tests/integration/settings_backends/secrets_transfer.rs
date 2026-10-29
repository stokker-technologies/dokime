/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Base64 Files Carry Secrets Between Independent Keyring Stores.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
// Third Party Imports
use base64::Engine as _;

// Local Imports
use dokime_domain::model::secrets::{
    catalog::Metadata,
    transfer::{Action, Choice},
};
use dokime_operations::secrets as wire;
use dokime_runtime::{Runtime, RuntimeConfig};

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

pub(crate) fn secret_metadata_rejects_old_name_field() {
    let value = serde_json::json!({ "name": "old.token", "label": "Old Token", "description": null });
    assert!(serde_json::from_value::<Metadata>(value).is_err());
    let value = serde_json::json!({ "identifier": "new.token", "label": "New Token", "description": null });
    let metadata: Metadata = serde_json::from_value(value).unwrap();
    assert_eq!(metadata.identifier, "new.token");
}

pub(crate) async fn saved_file_roundtrip() {
    let root = tempfile::tempdir().unwrap();
    let metadata = crate::service_support::metadata("secrets.source", "Source");
    let source = RuntimeConfig::new(root.path().join("source"), metadata).with_credential_store(keyring_core::mock::Store::new().unwrap());
    let mut source = Runtime::start(source).await.unwrap();
    let handle = source.handle();

    let metadata = Metadata {
        identifier: "roundtrip.token".into(),
        label: "Saved Secret".into(),
        description: None,
    };
    let request = wire::Put {
        metadata,
        value: Some("saved-value-λ".into()),
        generation: 0,
    };
    handle.operations().call::<wire::Put>(request).await.unwrap();
    let identifiers = vec!["roundtrip.token".into()];
    let archive = handle.operations().call::<wire::Export>(wire::Export { identifiers, generation: 1 }).await.unwrap();

    let file = root.path().join("secrets.b64");
    std::fs::write(&file, &archive.data).unwrap();
    source.shutdown().await.unwrap();
    drop(handle);

    let data = std::fs::read(file).unwrap();
    let decoded = base64::engine::general_purpose::STANDARD.decode(&data).unwrap();
    let document: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
    assert_eq!(
        document,
        serde_json::json!({
            "format_version": 1,
            "entries": [{"metadata": {"identifier": "roundtrip.token", "label": "Saved Secret", "description": null}, "value": "saved-value-λ"}],
        })
    );

    let metadata = crate::service_support::metadata("secrets.destination", "Destination");
    let destination =
        RuntimeConfig::new(root.path().join("destination"), metadata).with_credential_store(keyring_core::mock::Store::new().unwrap());
    let mut destination = Runtime::start(destination).await.unwrap();
    let handle = destination.handle();
    let preview = handle.operations().call::<wire::Preview>(wire::Preview { data: data.clone() }).await.unwrap();
    assert_eq!(preview.generation, 0);
    assert_eq!(preview.entries.len(), 1);
    assert!(!preview.entries[0].exists);

    let choices = vec![Choice {
        identifier: preview.entries[0].metadata.identifier.clone(),
        action: Action::Replace,
    }];
    handle
        .operations()
        .call::<wire::Import>(wire::Import {
            data,
            choices,
            generation: preview.generation,
        })
        .await
        .unwrap();
    let restored = handle.operations().call::<wire::Lookup>(wire::Lookup { identifier: "roundtrip.token".into() }).await.unwrap();
    assert_eq!(restored.value.as_deref(), Some("saved-value-λ"));
    destination.shutdown().await.unwrap();
}
