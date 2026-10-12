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
use dokime_app::{
    core::{DurationMillis, MeasurementUnit, UnitFamily, Value, ValueType},
    records::{RecordContract, RecordField, RecordIconKey, RecordSeverity, RecordTone},
    telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole},
};
use dokime_protocol::v1::catalog_client::CatalogClient;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn schemas_exact_pages() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let mut registration = common::fixture_registration(common::fixture_suite(&["passed"]));
    registration.record_contracts = vec![
        RecordContract::declared("fixture.reading", "Reading")
            .field(
                RecordField::new(common::metadata("counter", "Counter"), ValueType::Integer)
                    .with_default(Value::Integer(i64::MAX))
                    .with_choices([Value::Integer(i64::MIN), Value::Integer(i64::MAX)]),
            )
            .field(
                RecordField::new(common::metadata("comment", "Comment"), ValueType::Text)
                    .nullable()
                    .with_default(Value::Null),
            )
            .field(
                RecordField::new(common::metadata("absent", "Absent"), ValueType::Text).optional(),
            )
            .severity(RecordSeverity::Warning)
            .displayed(RecordIconKey::Gauge, RecordTone::Warning)
            .strict(),
    ];
    let mut telemetry = TelemetryContract::declared("fixture.samples", "Samples")
        .column(TelemetryColumn::declared(
            "count",
            "Count",
            ValueType::Integer,
            TelemetryRole::Counter,
        ))
        .strict();
    telemetry.plot_window = Some(DurationMillis(u64::MAX));
    telemetry.columns[1].default = Some(Value::Integer(i64::MAX));
    telemetry.columns[1].unit = Some(MeasurementUnit {
        metadata: common::metadata("unit.tick", "Tick"),
        family: UnitFamily::Count,
        symbol: "tick".into(),
    });
    telemetry.columns[1].display_unit = Some("ticks".into());
    registration.telemetry_contracts = vec![telemetry];
    handle
        .set_suites(
            "provider:fixture",
            dokime_app::definition::SuiteSet::new([registration.clone()]),
        )
        .await
        .unwrap();
    let config = Config::default()
        .with_bearer_auth(vec![grant(Role::Read)])
        .unwrap();
    let server = Server::start(handle.clone(), config).await.unwrap();
    let mut client = CatalogClient::connect(server.local_endpoint())
        .await
        .unwrap();
    assert_eq!(
        client
            .list_record_schemas(v1::ListRecordSchemasRequest::default())
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let first = client
        .list_record_schemas(authenticated(v1::ListRecordSchemasRequest {
            limit: 1,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.schemas.len(), 1);
    let mut ids = first
        .schemas
        .iter()
        .map(|s| s.record_type.clone())
        .collect::<Vec<_>>();
    let mut after = first.next_after_record_type.clone();
    while let Some(marker) = after {
        let page = client
            .list_record_schemas(authenticated(v1::ListRecordSchemasRequest {
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
    let mut expected = handle
        .record_contracts()
        .await
        .into_iter()
        .map(|c| c.record_type.to_string())
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(ids, expected);
    let record = client
        .get_record_schema(authenticated(v1::GetRecordSchemaRequest {
            record_type: "fixture.reading".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(record.default_severity, v1::Severity::Warn as i32);
    assert_eq!(
        record.validation_mode,
        Some(v1::ValidationMode::Strict as i32)
    );
    assert!(!record.allow_extra_fields);
    let display = record.display.unwrap();
    assert_eq!(display.icon, v1::RecordIcon::Gauge as i32);
    assert_eq!(display.tone, v1::RecordTone::Warning as i32);
    assert_eq!(
        record.fields[0].default_value.as_ref().unwrap().kind,
        Some(v1::value::Kind::IntegerValue(i64::MAX))
    );
    assert_eq!(
        record.fields[0].choices[0].kind,
        Some(v1::value::Kind::IntegerValue(i64::MIN))
    );
    assert_eq!(
        record.fields[1].default_value.as_ref().unwrap().kind,
        Some(v1::value::Kind::NullValue(0))
    );
    assert!(record.fields[1].nullable);
    assert!(record.fields[2].default_value.is_none());
    let streams = client
        .list_telemetry_schemas(authenticated(v1::ListTelemetrySchemasRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert!(
        streams
            .schemas
            .iter()
            .any(|s| s.stream == "fixture.samples")
    );
    let telemetry = client
        .get_telemetry_schema(authenticated(v1::GetTelemetrySchemaRequest {
            stream: "fixture.samples".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(telemetry.plot_window_ms, Some(u64::MAX));
    let unit = telemetry.columns[1].unit.as_ref().unwrap();
    assert_eq!(unit.family, v1::UnitFamily::Count as i32);
    assert_eq!(unit.symbol, "tick");
    assert_eq!(unit.metadata.as_ref().unwrap().identifier, "unit.tick");
    assert_eq!(telemetry.columns[1].display_unit.as_deref(), Some("ticks"));
    assert_eq!(
        telemetry.columns[1].default_value.as_ref().unwrap().kind,
        Some(v1::value::Kind::IntegerValue(i64::MAX))
    );
    assert_eq!(telemetry.columns[1].role, v1::TelemetryRole::Counter as i32);
    assert_eq!(telemetry.default_x.as_deref(), Some("timestamp"));
    assert_eq!(
        client
            .get_record_schema(authenticated(v1::GetRecordSchemaRequest {
                record_type: "missing".into()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        client
            .get_telemetry_schema(authenticated(v1::GetTelemetrySchemaRequest {
                stream: "missing".into()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        client
            .list_telemetry_schemas(authenticated(v1::ListTelemetrySchemasRequest {
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
            .list_record_schemas(authenticated(v1::ListRecordSchemasRequest {
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
    handle
        .set_suites(
            "provider:fixture",
            dokime_app::definition::SuiteSet::new([registration]),
        )
        .await
        .unwrap();
    assert_eq!(
        client
            .list_record_schemas(authenticated(v1::ListRecordSchemasRequest {
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
            .list_telemetry_schemas(authenticated(v1::ListTelemetrySchemasRequest {
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

pub(crate) async fn catalog_suite_extensions() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let mut suite = common::fixture_suite(&["passed"]);
    suite.test_case_templates[0].parameters[0].value_type = ValueType::Number;
    suite.test_case_templates[0].parameters[0].expression =
        dokime_app::definition::ParameterExpr::Literal {
            value: Value::Number(10.0),
        };
    suite.test_case_templates[0].parameters[0].min = Some(1.0);
    suite.test_case_templates[0].parameters[0].max = Some(100.0);
    suite.test_case_templates[0].parameters[0].extensions = BTreeMap::from([
        (
            "ui.description".to_owned(),
            Value::from("Choose an operator-facing value"),
        ),
        ("ui.limit".to_owned(), Value::Integer(i64::MAX)),
        (
            "ui.flags".to_owned(),
            Value::Object(BTreeMap::from([("visible".to_owned(), Value::from(true))])),
        ),
    ]);
    let mut registration = common::fixture_registration(suite);
    registration.product = Some("fixture.product".to_owned());
    handle
        .set_suites(
            "provider:fixture",
            dokime_app::definition::SuiteSet::new([registration]),
        )
        .await
        .unwrap();
    let config = Config::default()
        .with_bearer_auth(vec![grant(Role::Read)])
        .unwrap();
    let server = Server::start(handle, config).await.unwrap();
    let mut client = CatalogClient::connect(server.local_endpoint())
        .await
        .unwrap();

    let listed = client
        .list_suites(authenticated(v1::ListSuitesRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(listed.suites.len(), 1);
    assert_eq!(listed.suites[0].product.as_deref(), Some("fixture.product"));

    let fetched = client
        .get_suite(authenticated(v1::GetSuiteRequest {
            suite_id: "fixture.suite".to_owned(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        fetched.identity.as_ref().unwrap().product.as_deref(),
        Some("fixture.product")
    );
    assert_eq!(fetched.templates[0].parameters[0].min, Some(1.0));
    assert_eq!(fetched.templates[0].parameters[0].max, Some(100.0));
    let extensions = &fetched.templates[0].parameters[0].extensions;
    assert_eq!(
        extensions["ui.description"].kind,
        Some(v1::value::Kind::TextValue(
            "Choose an operator-facing value".to_owned()
        ))
    );
    assert_eq!(
        extensions["ui.limit"].kind,
        Some(v1::value::Kind::IntegerValue(i64::MAX))
    );
    match extensions["ui.flags"].kind.as_ref() {
        Some(v1::value::Kind::ObjectValue(value)) => assert_eq!(
            value.fields["visible"].kind,
            Some(v1::value::Kind::BoolValue(true))
        ),
        other => panic!("expected object extension, got {other:?}"),
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
