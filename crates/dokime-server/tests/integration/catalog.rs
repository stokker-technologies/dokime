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
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use super::*;
use dokime_domain::model;
use {
    dokime_runtime::DurationMillis, dokime_runtime::MeasurementUnit, dokime_runtime::UnitFamily, dokime_runtime::Value,
    dokime_runtime::ValueType, dokime_runtime::records::RecordContract, dokime_runtime::records::RecordField,
    dokime_runtime::records::RecordIconKey, dokime_runtime::records::RecordSeverity, dokime_runtime::records::RecordTone,
    dokime_runtime::telemetry::TelemetryColumn, dokime_runtime::telemetry::TelemetryContract, dokime_runtime::telemetry::TelemetryRole,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
struct CountRow {
    count: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn schemas_exact_pages() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let mut registration = support::fixture_registration(support::fixture_suite(&["passed"]));
    registration.record_contracts = vec![
        RecordContract::declared("fixture.reading", "Reading")
            .field(
                RecordField::new(support::metadata("counter", "Counter"), ValueType::Integer)
                    .with_default(Value::Integer(i64::MAX))
                    .with_choices([Value::Integer(i64::MIN), Value::Integer(i64::MAX)]),
            )
            .field(RecordField::new(support::metadata("comment", "Comment"), ValueType::Text).nullable().with_default(Value::Null))
            .field(RecordField::new(support::metadata("absent", "Absent"), ValueType::Text).optional())
            .severity(RecordSeverity::Warning)
            .displayed(RecordIconKey::Gauge, RecordTone::Warning)
            .strict(),
    ];
    let mut telemetry = TelemetryContract::declared("fixture.samples", "Samples")
        .typed::<CountRow>()
        .column(TelemetryColumn::declared("count", "Count", ValueType::Integer, TelemetryRole::Counter))
        .strict();
    telemetry.plot_window = Some(DurationMillis(u64::MAX));
    telemetry.columns[1].default = Some(Value::Integer(i64::MAX));
    telemetry.columns[1].unit =
        Some(MeasurementUnit { metadata: support::metadata("unit.tick", "Tick"), family: UnitFamily::Count, symbol: "tick".into() });
    telemetry.columns[1].display_unit = Some("ticks".into());
    registration.telemetry_contracts = vec![telemetry];
    handle.set_suites("provider:fixture", dokime_runtime::definition::SuiteSet::new([registration.clone()])).await.unwrap();
    let config = keyed();
    let server = Server::start(handle.clone(), config).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client.unary::<contracts::catalog::ListRecordSchemas>(contracts::catalog::ListRecordSchemas::default()).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    let first = client
        .unary::<contracts::catalog::ListRecordSchemas>(authenticated(contracts::catalog::ListRecordSchemas {
            limit: 1,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.schemas.len(), 1);
    let mut ids = first.schemas.iter().map(|s| s.record_type.clone()).collect::<Vec<_>>();
    let mut after = first.next_after_record_type.clone();
    while let Some(marker) = after {
        let page = client
            .unary::<contracts::catalog::ListRecordSchemas>(authenticated(contracts::catalog::ListRecordSchemas {
                limit: 1,
                after_record_type: marker,
                cursor: first.cursor.clone(),
            }))
            .await
            .unwrap()
            .into_inner();
        ids.extend(page.schemas.iter().map(|s| s.record_type.clone()));
        after = page.next_after_record_type;
    }
    let mut expected = handle.record_contracts().await.into_iter().map(|c| c.record_type.to_string()).collect::<Vec<_>>();
    expected.sort();
    assert_eq!(ids, expected);
    let record = client
        .unary::<contracts::catalog::GetRecordSchema>(authenticated(contracts::catalog::GetRecordSchema {
            record_type: "fixture.reading".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(record.default_severity, model::identity::wire::Severity::Warn);
    assert_eq!(record.validation_mode, Some(model::definition::wire::ValidationMode::Strict));
    assert!(!record.allow_extra_fields);
    let display = record.display.unwrap();
    assert_eq!(display.icon, model::definition::wire::RecordIcon::Gauge);
    assert_eq!(display.tone, model::definition::wire::RecordTone::Warning);
    assert_eq!(record.fields[0].default_value.as_ref().unwrap().to_core().unwrap(), Value::Integer(i64::MAX));
    assert_eq!(record.fields[0].choices[0].to_core().unwrap(), Value::Integer(i64::MIN));
    assert_eq!(record.fields[1].default_value.as_ref().unwrap().to_core().unwrap(), Value::Null);
    assert!(record.fields[1].nullable);
    assert!(record.fields[2].default_value.is_none());
    let streams = client
        .unary::<contracts::catalog::ListTelemetrySchemas>(authenticated(contracts::catalog::ListTelemetrySchemas::default()))
        .await
        .unwrap()
        .into_inner();
    assert!(streams.schemas.iter().any(|s| s.stream == "fixture.samples"));
    let telemetry = client
        .unary::<contracts::catalog::GetTelemetrySchema>(authenticated(contracts::catalog::GetTelemetrySchema {
            stream: "fixture.samples".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(telemetry.plot_window_ms, Some(u64::MAX));
    let unit = telemetry.columns[1].unit.as_ref().unwrap();
    assert_eq!(unit.family, model::definition::wire::UnitFamily::Count);
    assert_eq!(unit.symbol, "tick");
    assert_eq!(unit.metadata.as_ref().unwrap().identifier, "unit.tick");
    assert_eq!(telemetry.columns[1].display_unit.as_deref(), Some("ticks"));
    assert_eq!(telemetry.columns[1].default_value.as_ref().unwrap().to_core().unwrap(), Value::Integer(i64::MAX));
    assert_eq!(telemetry.columns[1].role, model::definition::wire::TelemetryRole::Counter);
    assert_eq!(telemetry.default_x.as_deref(), Some("timestamp"));
    assert_eq!(
        client
            .unary::<contracts::catalog::GetRecordSchema>(authenticated(contracts::catalog::GetRecordSchema {
                record_type: "missing".into()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        client
            .unary::<contracts::catalog::GetTelemetrySchema>(authenticated(contracts::catalog::GetTelemetrySchema {
                stream: "missing".into()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        client
            .unary::<contracts::catalog::ListTelemetrySchemas>(authenticated(contracts::catalog::ListTelemetrySchemas {
                limit: 101,
                ..Default::default()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        client
            .unary::<contracts::catalog::ListRecordSchemas>(authenticated(contracts::catalog::ListRecordSchemas {
                after_record_type: "missing".into(),
                cursor: first.cursor.clone(),
                ..Default::default()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    registration.record_contracts.clear();
    handle.set_suites("provider:fixture", dokime_runtime::definition::SuiteSet::new([registration])).await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::catalog::ListRecordSchemas>(authenticated(contracts::catalog::ListRecordSchemas {
                cursor: first.cursor,
                ..Default::default()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    assert_eq!(
        client
            .unary::<contracts::catalog::ListTelemetrySchemas>(authenticated(contracts::catalog::ListTelemetrySchemas {
                cursor: streams.cursor,
                ..Default::default()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn catalog_suite_extensions() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let mut suite = support::fixture_suite(&["passed"]);
    suite.test_case_templates[0].parameters[0].value_type = ValueType::Number;
    suite.test_case_templates[0].parameters[0].expression =
        dokime_runtime::definition::ParameterExpr::Literal { value: Value::Number(10.0) };
    suite.test_case_templates[0].parameters[0].min = Some(1.0);
    suite.test_case_templates[0].parameters[0].max = Some(100.0);
    suite.test_case_templates[0].parameters[0].extensions = BTreeMap::from([
        ("ui.description".to_owned(), Value::from("Choose an operator-facing value")),
        ("ui.limit".to_owned(), Value::Integer(i64::MAX)),
        ("ui.flags".to_owned(), Value::Object(BTreeMap::from([("visible".to_owned(), Value::from(true))]))),
    ]);
    let registration = support::fixture_registration(suite);
    handle.set_suites("provider:fixture", dokime_runtime::definition::SuiteSet::new([registration])).await.unwrap();
    let config = keyed();
    let server = Server::start(handle, config).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    let listed = client
        .unary::<contracts::catalog::ListSuites>(authenticated(contracts::catalog::ListSuites::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(listed.suites.len(), 1);

    let fetched = client
        .unary::<contracts::catalog::GetSuite>(authenticated(contracts::catalog::GetSuite { suite_id: "fixture.suite".to_owned() }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(fetched.templates[0].parameters[0].min, Some(1.0));
    assert_eq!(fetched.templates[0].parameters[0].max, Some(100.0));
    let extensions: BTreeMap<_, _> = fetched.templates[0].parameters[0]
        .extensions
        .iter()
        .map(|entry| (entry.key.as_str(), entry.value.as_ref().unwrap().to_core().unwrap()))
        .collect();
    assert_eq!(extensions["ui.description"], Value::Text("Choose an operator-facing value".to_owned()));
    assert_eq!(extensions["ui.limit"], Value::Integer(i64::MAX));
    match &extensions["ui.flags"] {
        Value::Object(value) => assert_eq!(value["visible"], Value::Bool(true)),
        other => panic!("expected object extension, got {other:?}"),
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
