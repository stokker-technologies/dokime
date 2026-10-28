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
use crate::service_support::prelude::*;
use async_trait::async_trait;
use dokime_domain::model;
use dokime_runtime::execution::Runnable;
use dokime_runtime::runtime::EmbeddedProvider;
use harmos::sidecar::StreamDeclaration;
use std::sync::Arc;
use {dokime_runtime::InstalledPlugin, dokime_runtime::ValueType};
use {
    dokime_runtime::records::RecordContract, dokime_runtime::records::RecordField, dokime_runtime::records::RecordPublication,
    dokime_runtime::records::RecordSeverity, model::records::publication::RecordPublishRequest,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Publisher;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[async_trait]
impl EmbeddedProvider for Publisher {
    fn catalog(&self) -> InstalledPlugin {
        InstalledPlugin {
            source: "fixture_records".into(),
            version: "0.1.0".into(),
            enabled: true,
            record_contracts: vec![RecordContract {
                record_type: identifier("fixture.sidecar.observed"),
                schema_version: 1,
                metadata: metadata("fixture.sidecar.observed", "Observed"),
                display: Default::default(),
                template_id: None,
                owner: None,
                default_severity: RecordSeverity::Info,
                fields: vec![RecordField::new(metadata("value", "Value"), ValueType::Number)],
                validation_mode: None,
                allow_extra_fields: false,
            }],
            ..Default::default()
        }
    }
    fn streams(&self) -> Vec<StreamDeclaration> {
        Vec::new()
    }
    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn Runnable>>> {
        Ok(Vec::new())
    }
    async fn stop(&self) -> dokime_runtime::runtime::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn native_publications_order() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = Runtime::start(config(root.path()).register(Arc::new(Publisher))).await.unwrap();
    let runtime = owner.handle();
    let caller = owner.fixture_operation_caller("fixture_records").unwrap();
    let mut records = runtime.subscribe_records();
    let mut request = RecordPublishRequest::new(Identifier::new("fixture.sidecar.observed").unwrap(), RecordSeverity::Info);
    request.details.insert("value".into(), dokime_runtime::Value::Number(7.0));
    request.scope = Some(dokime_runtime::RecordScope::default().with_cycle(dokime_runtime::model::identity::new_id::<TestCycleId>()));
    let timestamp = "2026-09-17T10:00:00Z";
    let first = RecordPublication {
        request_json: serde_json::to_string(&request).unwrap(),
        produced_at: timestamp.into(),
    };
    let mut forged = serde_json::to_value(&request).unwrap();
    forged["source"] = "foreign.owner".into();
    let mut undeclared = request.clone();
    undeclared.record_type = Identifier::new("undeclared.record").unwrap();
    let publications = [
        first.clone(),
        RecordPublication {
            request_json: serde_json::to_string(&forged).unwrap(),
            produced_at: timestamp.into(),
        },
        RecordPublication {
            request_json: serde_json::to_string(&undeclared).unwrap(),
            produced_at: timestamp.into(),
        },
        RecordPublication {
            request_json: "garbled".into(),
            produced_at: timestamp.into(),
        },
        RecordPublication {
            request_json: first.request_json,
            produced_at: String::new(),
        },
    ];
    let arrival = dokime_runtime::model::time::now_utc();
    // The host files what the plugin relays under the caller's own identity,
    // and refuses to the plugin, synchronously, what it cannot file: a forged
    // source, an undeclared record type and a payload that is no request.
    for (index, publication) in publications.into_iter().enumerate() {
        let filed = caller.call::<RecordPublication>(publication).await;
        assert_eq!(filed.is_ok(), matches!(index, 0 | 4), "publication {index}: {filed:?}");
    }
    let mut admitted = Vec::new();
    while admitted.len() < 2 {
        let entry = records.next().await.unwrap().unwrap();
        assert_eq!(entry.record_type.as_str(), "fixture.sidecar.observed");
        admitted.push(entry);
    }
    assert_eq!(admitted[0].timestamp.as_str(), timestamp);
    assert!(admitted[1].timestamp >= arrival);
    assert!(admitted[1].sequence > admitted[0].sequence);
    for entry in &admitted {
        assert_eq!(entry.source, "fixture_records");
        assert_eq!(entry.scope, None);
        assert_eq!(entry.details["value"], dokime_runtime::Value::Number(7.0));
    }
    drop(records);
    drop(runtime);
    owner.shutdown().await.unwrap();
}
