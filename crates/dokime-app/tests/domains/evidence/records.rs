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
use crate::support::prelude::*;
use dokime::Message;
use dokime::records::{RecordPublication, RecordPublishRequest, RecordSeverity};

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

pub(crate) async fn native_publications_order() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = Runtime::start(config(root.path()).with_plugins([
        fixture_bundle(root.path(), "fixture-sidecar-protocol"),
        revised_bundle(
            root.path(),
            "dokime-route-plugin",
            "route-publications-peer",
            "records",
        ),
    ]))
    .await
    .unwrap();
    let runtime = owner.handle();
    let mut records = runtime.subscribe_records();
    let mut request = RecordPublishRequest::new(
        Identifier::new("fixture.sidecar.observed").unwrap(),
        RecordSeverity::Info,
    );
    request
        .details
        .insert("value".into(), dokime::core::Value::Number(7.0));
    request.scope = Some(dokime::core::RecordScope::default().with_cycle(TestCycleId::new()));
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
    let arrival = dokime::core::Timestamp::now_utc();
    // The host files what the plugin relays under the caller's own identity,
    // and refuses to the plugin, synchronously, what it cannot file: a forged
    // source, an undeclared record type and a payload that is no request.
    for (index, publication) in publications.into_iter().enumerate() {
        let filed = runtime
            .operations()
            .call_encoded(
                "fixture-sidecar.publish-record",
                publication.encode_to_vec(),
            )
            .await;
        assert_eq!(
            filed.is_ok(),
            matches!(index, 0 | 4),
            "publication {index}: {filed:?}"
        );
    }
    let mut admitted = Vec::new();
    tokio::time::timeout(Duration::from_secs(5), async {
        while admitted.len() < 2 {
            let entry = records.next().await.unwrap().unwrap();
            assert_eq!(entry.record_type.as_str(), "fixture.sidecar.observed");
            admitted.push(entry);
        }
    })
    .await
    .unwrap();
    assert_eq!(admitted[0].timestamp.as_str(), timestamp);
    assert!(admitted[1].timestamp >= arrival);
    assert!(admitted[1].sequence > admitted[0].sequence);
    for entry in &admitted {
        assert_eq!(entry.source, "fixture-sidecar");
        assert_eq!(entry.scope, None);
        assert_eq!(entry.details["value"], dokime::core::Value::Number(7.0));
    }
    let mut ordered = Vec::new();
    for (route, kind) in [
        ("fixture-sidecar.publish-record", "fixture.sidecar.observed"),
        ("route-fixture.publish_record", "fixture.route.observed"),
        ("fixture-sidecar.publish-record", "fixture.sidecar.observed"),
    ] {
        request.record_type = Identifier::new(kind).unwrap();
        let publication = RecordPublication {
            request_json: serde_json::to_string(&request).unwrap(),
            produced_at: timestamp.into(),
        };
        runtime
            .operations()
            .call_encoded(route, publication.encode_to_vec())
            .await
            .unwrap();
    }
    tokio::time::timeout(Duration::from_secs(5), async {
        for _ in 0..3 {
            ordered.push(records.next().await.unwrap().unwrap());
        }
    })
    .await
    .unwrap();
    assert_eq!(
        ordered
            .iter()
            .map(|row| row.source.as_str())
            .collect::<Vec<_>>(),
        ["fixture-sidecar", "route-fixture", "fixture-sidecar"]
    );
    assert!(ordered[0].sequence < ordered[1].sequence && ordered[1].sequence < ordered[2].sequence);
    drop(records);
    drop(runtime);
    owner.shutdown().await.unwrap();
}
