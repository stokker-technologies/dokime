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
use std::collections::{BTreeMap, BTreeSet};
// Third Party Imports
use futures_util::FutureExt;

// Local Imports
use super::*;
use dokime::definition::{ParameterDefinition, ParameterExpr, SuiteSet};
use dokime::domain::model;
use dokime::runtime::command::CommandContext;
use dokime::telemetry::{TelemetryBatch, TelemetryRow, TelemetrySample};
use dokime::{Identifier, ParameterId, RecordScope, Value, ValueType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn parameter(key: &str, group: &str, value: f64) -> ParameterDefinition {
    ParameterDefinition {
        expansion: Default::default(),
        min: None,
        max: None,
        parameter_id: ParameterId::from_seed(format!("traceability.fixture.{key}")),
        metadata: support::metadata(key, key),
        group_id: Some(Identifier::new(group).unwrap()),
        value_type: ValueType::Number,
        unit: None,
        required: false,
        expression: ParameterExpr::Literal { value: Value::Number(value) },
        runtime_writable: false,
        selection: None,
        list: None,
        extensions: BTreeMap::new(),
    }
}

fn host_batch(scope: Option<RecordScope>, host_name: &str) -> TelemetryBatch {
    let stream = Identifier::new("host.system").unwrap();
    let mut sample =
        TelemetrySample::new(dokime::model::identity::new_id(), stream.clone(), 0, dokime::model::time::now_utc(), "fixture.host");
    sample.scope = scope;

    let row = plugin_host::System {
        host_name: Some(host_name.into()),
        cpu_load_percent: Some(12.5),
        ..Default::default()
    };
    let payload = dokime::harmos::Message::encode(&row);
    sample.values = row.into_row();
    let timestamp = sample.timestamp.clone();
    sample.values.insert("timestamp".into(), Value::from(timestamp));
    let sample = sample.with_storage_payload(payload);
    TelemetryBatch::new(dokime::model::identity::new_id(), stream, vec![sample])
}

pub(crate) async fn get_traceability_cursor() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    let mut suite = support::fixture_suite(&["passed"]);
    suite.test_case_templates[0].parameters.extend([
        parameter("environment.latitude", "traceability.environment", 47.3769),
        parameter("operator.email", "traceability.operator", 1.0),
    ]);
    let mut registration = support::fixture_registration(suite.clone());
    registration.telemetry_contracts = plugin_host::host_telemetry_contracts();
    handle.set_suites("fixture", SuiteSet::new([registration])).await.unwrap();
    let cycle_id = support::cycle_with_cases(
        &handle,
        support::metadata("traceability.fixture", "Traceability"),
        support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await;

    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    let first = client
        .unary::<contracts::observation::GetTraceability>(authenticated(contracts::observation::GetTraceability {
            limit: 1,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.snapshots.len(), 1);
    assert!(!first.complete);
    assert!(!first.next_after_traceability_id.is_empty());
    let first_snapshot = first.snapshots[0].clone();
    assert!(!first_snapshot.entries.is_empty());
    let first_source = first_snapshot.entries[0].source.as_deref().unwrap();
    assert!(!first_source.is_empty());
    // What this replaces looked for the parameter name "environment.latitude"
    // among traceability entry identifiers, where a parameter name cannot
    // appear — inside an `if let`, so it never ran once. Assert instead what
    // every entry must carry, which can fail.
    assert!(first_snapshot.entries.iter().all(|entry| {
        entry.metadata.as_ref().is_some_and(|metadata| !metadata.identifier.is_empty() && !metadata.label.is_empty())
            && entry.value.is_some()
            && entry.source.as_deref().is_some_and(|source| !source.is_empty())
    }));

    let first_host_receipt = handle.publish_telemetry_batch(host_batch(None, "first-host")).await.unwrap();
    assert_eq!(first_host_receipt.accepted, 1);
    handle.storage().flush().await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::observation::GetTraceability>(authenticated(contracts::observation::GetTraceability {
                limit: 1,
                after_traceability_id: first.next_after_traceability_id.clone(),
                cursor: first.cursor.clone(),
                host_traceability_fingerprint: first.host_traceability_fingerprint.clone(),
                ..Default::default()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    let first_after_host = client
        .unary::<contracts::observation::GetTraceability>(authenticated(contracts::observation::GetTraceability {
            limit: 1,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    // Unrelated Telemetry Advances Storage Without Invalidating The Host Page.
    let before = handle.storage().cursor_saved();
    let stream = Identifier::new("host.network").unwrap();
    let row = plugin_host::Network {
        sample_index: Some(1),
        ..Default::default()
    };
    let payload = dokime::harmos::Message::encode(&row);
    let mut sample =
        TelemetrySample::new(dokime::model::identity::new_id(), stream.clone(), 0, dokime::model::time::now_utc(), "fixture.host");
    sample.values = row.into_row();
    sample.values.insert("timestamp".into(), Value::from(sample.timestamp.clone()));
    let sample = sample.with_storage_payload(payload);
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime::model::identity::new_id(), stream, vec![sample])).await.unwrap();
    let after = handle.storage().flush().await.unwrap();
    assert!(after.sequence > before.sequence);

    let mut pages = vec![first_after_host];
    for _iteration in 0..16 {
        if pages.last().is_some_and(|page| page.complete) {
            break;
        }
        let page = pages.last().unwrap();
        let next = client
            .unary::<contracts::observation::GetTraceability>(authenticated(contracts::observation::GetTraceability {
                limit: 1,
                after_traceability_id: page.next_after_traceability_id.clone(),
                cursor: page.cursor.clone(),
                host_traceability_fingerprint: page.host_traceability_fingerprint.clone(),
                ..Default::default()
            }))
            .await
            .unwrap()
            .into_inner();
        pages.push(next);
    }
    assert!(pages.last().is_some_and(|page| page.complete));

    let mut ids = BTreeSet::new();
    let sources = pages
        .iter()
        .flat_map(|page| page.snapshots.iter())
        .inspect(|snapshot| {
            assert!(ids.insert(snapshot.traceability_id.clone()));
        })
        .flat_map(|snapshot| snapshot.entries.iter())
        .filter_map(|entry| entry.source.clone())
        .collect::<BTreeSet<_>>();
    assert!(!sources.contains("traceability.environment"));
    assert!(sources.contains("traceability.test_execution"));
    assert!(sources.contains("traceability.operator"));

    let second_cycle_id = support::cycle_with_cases(
        &handle,
        support::metadata("traceability.second", "Second Traceability"),
        support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await;
    let second_host_receipt = handle.publish_telemetry_batch(host_batch(None, "second-host")).await.unwrap();
    assert_eq!(second_host_receipt.accepted, 1);
    handle.storage().flush().await.unwrap();

    let scoped = client
        .unary::<contracts::observation::GetTraceability>(authenticated(contracts::observation::GetTraceability {
            scope: Some(model::identity::wire::EvidenceScope {
                cycle_id: Some(cycle_id.to_string()),
                ..Default::default()
            }),
            limit: 100,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(scoped.snapshots.len() >= 2);
    assert_eq!(scoped.scope.as_ref().and_then(|scope| scope.cycle_id.as_deref()), Some(cycle_id.as_str()),);
    assert!(
        scoped
            .snapshots
            .iter()
            .any(|snapshot| { snapshot.scope.as_ref().and_then(|scope| scope.cycle_id.as_deref()) == Some(cycle_id.as_str()) }),
        "{scoped:?}"
    );
    assert!(
        scoped
            .snapshots
            .iter()
            .flat_map(|snapshot| &snapshot.entries)
            .all(|entry| { entry.metadata.as_ref().is_none_or(|metadata| metadata.identifier != "host.hostname") })
    );
    let current = client
        .unary::<contracts::observation::GetTraceability>(authenticated(contracts::observation::GetTraceability {
            limit: 100,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(current.snapshots.iter().any(|snapshot| {
        snapshot.entries.iter().any(|entry| {
            entry.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "host.hostname")
                && entry
                    .value
                    .as_ref()
                    .map(|value| value.to_core().unwrap())
                    .is_some_and(|kind| matches!(kind, dokime::Value::Text(value) if value == "second-host"))
        })
    }));

    let mut invalid_cursor = pages[0].cursor.clone().unwrap();
    invalid_cursor.epoch = "different-epoch".into();
    assert_eq!(
        client
            .unary::<contracts::observation::GetTraceability>(authenticated(contracts::observation::GetTraceability {
                cursor: Some(invalid_cursor),
                after_traceability_id: pages[0].next_after_traceability_id.clone(),
                host_traceability_fingerprint: pages[0].host_traceability_fingerprint.clone(),
                ..Default::default()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );

    handle.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    handle.seal_cycle(second_cycle_id, CommandContext::default()).await.unwrap();
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn traceability_bounds_clone() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path(), &dokime_server::grpc::Service::prepare(Config::local()).unwrap()).await;
    let handle = owner.handle();
    let suite = support::fixture_suite(&["passed"]);
    let mut registration = support::fixture_registration(suite);
    registration.telemetry_contracts = plugin_host::host_telemetry_contracts();
    handle.set_suites("fixture", SuiteSet::new([registration])).await.unwrap();

    let receipt = handle.publish_telemetry_batch(host_batch(None, "budget-host")).await.unwrap();
    assert_eq!(receipt.accepted, 1);
    handle.storage().flush().await.unwrap();

    let snapshot = handle.watch_state().latest();
    let error = handle.traceability_snapshots_page_at(&snapshot, None, "", 100, 0, 512 * 1024).boxed_local().await.unwrap_err();
    assert!(error.to_string().contains("bounded page budget"));

    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Tests === */
