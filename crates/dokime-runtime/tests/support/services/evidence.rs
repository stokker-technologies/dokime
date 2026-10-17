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
use super::identifier;
use dokime_runtime::{Identifier, RecordScope, Timestamp, Value};
use dokime_runtime::definition::TestCase;
use {
    dokime_domain::model::execution::result::ExecutionStatus,
    dokime_domain::model::execution::result::TestExecution,
};
use dokime_runtime::records::RecordSeverity;
use dokime_runtime::telemetry::{TelemetryBatch, TelemetrySample};
use dokime_runtime::transport::{
    ProviderCriterionOutcome, ProviderCycleOutcome, ProviderDependencyProbe,
    ProviderExecutionFailure, ProviderExecutionOutcome, ProviderPhaseOutcome,
    ProviderRecordOutcome, ProviderStatusOutcome, ProviderTelemetryOutcome,
};
use dokime_runtime::RuntimeHandle;
use dokime_runtime::domains::evidence::RecordPublishRequest;
use dokime_runtime::domains::execution::provider::ExecutionFixtureContext;
use dokime_domain::model::execution::result::CycleView;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct ProviderIdentity<'a> {
    pub module: &'a str,
    pub registrations: u64,
    pub suite: &'a str,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn cycle_outcome(
    cycle: &CycleView,
    provider: ProviderIdentity<'_>,
    outcome: impl Fn(&TestCase, &TestExecution) -> ProviderExecutionOutcome,
) -> ProviderCycleOutcome {
    let executions = runnable_executions(cycle)
        .map(|execution| {
            let case = cycle
                .cases()
                .find(|case| case.test_case_id == execution.test_case_id)
                .expect("execution case exists");
            outcome(case, execution)
        })
        .collect::<Vec<_>>();
    let failures = executions
        .iter()
        .flat_map(|execution| execution.failures.clone())
        .collect::<Vec<_>>();

    ProviderCycleOutcome {
        status: if failures.is_empty() {
            "passed"
        } else {
            "failed"
        }
        .to_owned(),
        dependency_imports: vec![ProviderDependencyProbe {
            module: provider.module.to_owned(),
            status: "imported".to_owned(),
            version: None,
            error: None,
        }],
        provider: ProviderStatusOutcome {
            status: "active".to_owned(),
            domain_registration_count: provider.registrations,
            test_suite_count: provider.registrations,
            test_suite_identifiers: vec![provider.suite.to_owned()],
            error: None,
        },
        executions,
        failures,
    }
}

pub fn python_outcome(
    cycle: &CycleView,
    default_status: &str,
    case_parameter_status: bool,
) -> ProviderCycleOutcome {
    cycle_outcome(
        cycle,
        ProviderIdentity {
            module: "example_module",
            registrations: 4,
            suite: "plt400.ract",
        },
        |case, execution| {
            let status = if case_parameter_status {
                case.parameters
                    .get("fixture.result")
                    .and_then(|value| match value {
                        Value::Text(value) => Some(value.as_str()),
                        _ => None,
                    })
                    .unwrap_or(default_status)
            } else {
                default_status
            };
            let records = if status == "passed" {
                vec![ProviderRecordOutcome {
                    record_type: "tracking.fixture_warning".to_owned(),
                    timestamp: Timestamp::new("2026-01-01T00:00:01Z"),
                    severity: RecordSeverity::Warning,
                    message: Some("Fixture warning event for notification inspection.".to_owned()),
                    error: None,
                    details: BTreeMap::from([(
                        "warning_code".to_owned(),
                        Value::from("fixture_warning"),
                    )]),
                }]
            } else {
                vec![ProviderRecordOutcome {
                    record_type: "test.step.failed".to_owned(),
                    timestamp: Timestamp::new("2026-01-01T00:00:02Z"),
                    severity: RecordSeverity::Error,
                    message: Some("Collect Tracking failed: fixture failure".to_owned()),
                    error: None,
                    details: BTreeMap::from([
                        ("identifier".to_owned(), Value::from("collect_tracking")),
                        ("label".to_owned(), Value::from("Collect Tracking")),
                        ("stacktrace".to_owned(), Value::from("fixture stacktrace")),
                    ]),
                }]
            };

            ProviderExecutionOutcome {
                case_id: case.test_case_id.as_str().to_owned(),
                test_execution_id: execution.test_execution_id.as_str().to_owned(),
                template_identifier: "plt400.ract.tracking".to_owned(),
                label: "Tracking".to_owned(),
                module: "fixture_suite.plt400.ract.tracking".to_owned(),
                status: status.to_owned(),
                phases: vec![
                    ProviderPhaseOutcome {
                        identifier: "setup".to_owned(),
                        label: "Setup".to_owned(),
                        status: if status == "passed" {
                            "passed"
                        } else {
                            "skipped"
                        }
                        .to_owned(),
                    },
                    ProviderPhaseOutcome {
                        identifier: "collect_tracking".to_owned(),
                        label: "Collect Tracking".to_owned(),
                        status: status.to_owned(),
                    },
                ],
                records,
                telemetry: vec![ProviderTelemetryOutcome {
                    stream: "tracking".to_owned(),
                    label: "Tracking".to_owned(),
                    samples: vec![
                        crate::service_support::contracts::provider_tracking_row(1.0, status == "passed"),
                        crate::service_support::contracts::provider_tracking_row(2.0, status == "passed"),
                    ],
                }],
                criteria: vec![ProviderCriterionOutcome {
                    identifier: "simulation.execution".to_owned(),
                    label: "Simulation Execution".to_owned(),
                    status: status.to_owned(),
                    expected: None,
                    required: Some(true),
                    message: (status != "passed").then(|| "fixture failure".to_owned()),
                }],
                failures: if status == "passed" {
                    Vec::new()
                } else {
                    vec![ProviderExecutionFailure {
                        stage: "template".to_owned(),
                        template_identifier: Some("plt400.ract.tracking".to_owned()),
                        module: None,
                        message: "fixture failure".to_owned(),
                        detail: None,
                    }]
                },
            }
        },
    )
}

pub fn custom_outcome(cycle: &CycleView) -> ProviderCycleOutcome {
    cycle_outcome(
        cycle,
        ProviderIdentity {
            module: "testdb",
            registrations: 1,
            suite: "custom.suite",
        },
        |case, execution| ProviderExecutionOutcome {
            case_id: case.test_case_id.as_str().to_owned(),
            test_execution_id: execution.test_execution_id.as_str().to_owned(),
            template_identifier: "custom.template".to_owned(),
            label: "Custom Template".to_owned(),
            module: "testdb".to_owned(),
            status: "passed".to_owned(),
            phases: vec![ProviderPhaseOutcome {
                identifier: "custom.phase".to_owned(),
                label: "Custom Phase".to_owned(),
                status: "passed".to_owned(),
            }],
            records: vec![
                tracking_resource_opened_record(),
                relay_resource_opened_record(),
            ],
            telemetry: Vec::new(),
            criteria: vec![ProviderCriterionOutcome {
                identifier: "custom.criterion".to_owned(),
                label: "Custom Criterion".to_owned(),
                status: "passed".to_owned(),
                expected: Some(Value::from("passed")),
                required: Some(true),
                message: None,
            }],
            failures: Vec::new(),
        },
    )
}

pub fn runnable_executions(
    cycle: &dokime_domain::model::execution::result::CycleView,
) -> impl Iterator<Item = &TestExecution> {
    cycle.test_executions.iter().filter(|execution| {
        matches!(
            execution.status,
            ExecutionStatus::Pending | ExecutionStatus::Running
        )
    })
}

pub async fn activate_fixture_execution(
    context: &ExecutionFixtureContext,
    execution: &TestExecution,
) {
    context
        .update_execution_phase(
            execution.scope.clone(),
            Identifier::new("fixture.phase").unwrap(),
            "Fixture Phase".to_owned(),
            "running".to_owned(),
        )
        .await
        .unwrap();
}

pub async fn publish_tracking_sample(
    runtime: &RuntimeHandle,
    cycle: &dokime_domain::model::execution::result::CycleView,
) {
    let execution = cycle
        .test_executions
        .iter()
        .find(|execution| execution.status == ExecutionStatus::Running)
        .expect("running execution exists");
    let case = cycle
        .cases()
        .find(|case| case.test_case_id == execution.test_case_id)
        .expect("execution case exists");
    let mut sample = TelemetrySample::new(dokime_runtime::model::identity::new_id(), Identifier::new("tracking").unwrap(),
        0,
        dokime_runtime::model::time::now_utc(),
        "python.fixture",
    );
    sample.scope = Some(
        dokime_runtime::RecordScope::default()
            .with_cycle(cycle.test_cycle_id.clone())
            .with_test_case(case.test_case_id.clone())
            .with_test_execution(execution.test_execution_id.clone())
            .with_template(case.test_case_template_id.clone()),
    );
    sample
        .values
        .insert("elapsed_s".to_owned(), Value::Number(1.0));
    sample
        .values
        .insert("target_locked".to_owned(), Value::Bool(true));

    runtime
        .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), Identifier::new("tracking").unwrap(),
            vec![crate::service_support::contracts::typed_minimal_tracking_sample(
                sample,
            )],
        ))
        .await
        .unwrap();
}

pub fn provider_execution_scope(cycle: &dokime_domain::model::execution::result::CycleView) -> RecordScope {
    let execution = initialized_execution(cycle);
    let case = cycle
        .cases()
        .find(|case| case.test_case_id == execution.test_case_id)
        .expect("execution case exists");
    RecordScope::default()
        .with_cycle(cycle.test_cycle_id.clone())
        .with_test_case(execution.test_case_id.clone())
        .with_test_execution(execution.test_execution_id.clone())
        .with_template(case.test_case_template_id.clone())
}

pub fn initialized_execution(
    cycle: &dokime_domain::model::execution::result::CycleView,
) -> &dokime_domain::model::execution::result::TestExecution {
    cycle
        .test_executions
        .iter()
        .find(|execution| {
            matches!(
                execution.status,
                ExecutionStatus::Pending | ExecutionStatus::Running
            )
        })
        .expect("fake provider outcome requires an initialized execution")
}

pub fn custom_record() -> RecordPublishRequest {
    let mut request =
        RecordPublishRequest::new(identifier("custom.measurement"), RecordSeverity::Info);
    request.details = BTreeMap::from([("value".to_owned(), Value::Number(1.0))]);
    request
}

pub fn tracking_derived_record(record_type: &str, scope: &RecordScope) -> RecordPublishRequest {
    let mut request = RecordPublishRequest::new(identifier(record_type), RecordSeverity::Info);
    request.scope = Some(scope.clone());
    request.message = Some(format!("{record_type} derived from tracking telemetry."));
    request.details = match record_type {
        "target.lost" => tracking_target_details("lost"),
        "target.acquired" => tracking_target_details("locked"),
        "tool.connection.lost" => tracking_connection_details("disconnected"),
        "tool.connection.acquired" => tracking_connection_details("connected"),
        _ => BTreeMap::new(),
    };
    request
}

pub fn tracking_target_details(target_lock_state: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("resource_id".to_owned(), Value::from("tracking_resource")),
        ("resource_kind".to_owned(), Value::from("tool")),
        (
            "target_lock_state".to_owned(),
            Value::from(target_lock_state),
        ),
        ("status_source".to_owned(), Value::from("tracking")),
    ])
}

pub fn tracking_connection_details(connection_state: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("resource_id".to_owned(), Value::from("tracking_resource")),
        ("resource_kind".to_owned(), Value::from("tool")),
        ("connection_state".to_owned(), Value::from(connection_state)),
        ("status_source".to_owned(), Value::from("tracking")),
    ])
}

pub fn tracking_batch() -> TelemetryBatch {
    let mut sample = dokime_runtime::telemetry::TelemetrySample::new(dokime_runtime::model::identity::new_id(), identifier("tracking"),
        0,
        dokime_runtime::model::time::now_utc(),
        "testdb",
    );
    sample
        .values
        .insert("distance_m".to_owned(), Value::Number(1.0));
    TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier("tracking"),
        vec![crate::service_support::contracts::typed_fixture_sample(sample)],
    )
}

pub fn host_system_batch() -> TelemetryBatch {
    let mut sample = dokime_runtime::telemetry::TelemetrySample::new(dokime_runtime::model::identity::new_id(), identifier("host.system"),
        0,
        dokime_runtime::model::time::now_utc(),
        "fixture.host",
    );
    sample
        .values
        .insert("cpu_load_percent".to_owned(), Value::Number(12.5));
    sample
        .values
        .insert("memory_pressure_percent".to_owned(), Value::Number(42.0));
    sample
        .values
        .insert("battery_percent".to_owned(), Value::Number(99.0));

    TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier("host.system"),
        vec![crate::service_support::contracts::typed_fixture_sample(sample)],
    )
}

pub fn tracking_transition_batch() -> TelemetryBatch {
    let mut samples = [
        tracking_status_sample(0, 0.0, 1.0, true, true),
        tracking_status_sample(1, 3.0, 2.0, false, true),
        tracking_status_sample(2, 8.0, 3.0, true, true),
        tracking_status_sample(3, 10.0, 4.0, false, false),
        tracking_status_sample(4, 17.0, 6.0, true, true),
    ];

    insert_sample_value(&mut samples[1], "speed_m_per_s", 1.0 / 3.0);
    insert_sample_value(&mut samples[1], "velocity_x_m_per_s", 1.0 / 3.0);
    insert_sample_value(&mut samples[1], "velocity_y_m_per_s", 0.0);
    insert_sample_value(&mut samples[2], "target_reacquire_time_s", 5.0);
    insert_sample_value(&mut samples[4], "connection_acquired_time_s", 7.0);
    insert_sample_value(&mut samples[4], "connection_loss_distance_m", 4.0);
    insert_sample_value(&mut samples[4], "connection_acquired_distance_m", 6.0);

    TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier("tracking"),
        samples
            .into_iter()
            .map(crate::service_support::contracts::typed_fixture_sample)
            .collect(),
    )
}

pub fn insert_sample_value(
    sample: &mut dokime_runtime::telemetry::TelemetrySample,
    key: impl Into<String>,
    value: f64,
) {
    sample.values.insert(key.into(), Value::Number(value));
    sample.storage_payload =
        crate::service_support::contracts::typed_fixture_sample(sample.clone()).storage_payload;
}

pub fn tracking_status_sample(
    index: usize,
    elapsed_s: f64,
    distance_m: f64,
    target_locked: bool,
    tool_connected: bool,
) -> dokime_runtime::telemetry::TelemetrySample {
    let mut sample = dokime_runtime::telemetry::TelemetrySample::new(dokime_runtime::model::identity::new_id(), identifier("tracking"),
        0,
        Timestamp::new(format!("2026-05-28T10:00:0{index}Z")),
        "testdb",
    );
    sample
        .values
        .insert("elapsed_s".to_owned(), Value::Number(elapsed_s));
    sample
        .values
        .insert("distance_m".to_owned(), Value::Number(distance_m));
    sample
        .values
        .insert("position_x_m".to_owned(), Value::Number(distance_m));
    sample
        .values
        .insert("position_y_m".to_owned(), Value::Number(0.0));
    sample
        .values
        .insert("target_locked".to_owned(), Value::from(target_locked));
    sample
        .values
        .insert("tool_connected".to_owned(), Value::from(tool_connected));
    crate::service_support::contracts::typed_fixture_sample(sample)
}

pub fn relay_resource_opened_record() -> ProviderRecordOutcome {
    ProviderRecordOutcome {
        record_type: "resource.opened".to_owned(),
        timestamp: Timestamp::new("2026-01-01T00:00:01Z"),
        severity: RecordSeverity::Info,
        message: Some("Relay resource opened.".to_owned()),
        error: None,
        details: BTreeMap::from([
            ("resource_name".to_owned(), Value::from("relay")),
            ("resource_kind".to_owned(), Value::from("relay")),
            ("path".to_owned(), Value::from("C:/RelayDiag")),
            ("connection".to_owned(), Value::from("usb")),
            ("endpoint".to_owned(), Value::from("COM3")),
            ("process_isolation".to_owned(), Value::from(true)),
            (
                "traceability".to_owned(),
                Value::Object(BTreeMap::from([
                    ("serial_number".to_owned(), Value::from("HUR-123")),
                    ("firmware_version".to_owned(), Value::from("9.9.9")),
                ])),
            ),
            (
                "device_info".to_owned(),
                Value::Object(BTreeMap::from([(
                    "device_type".to_owned(),
                    Value::from("Relay"),
                )])),
            ),
            (
                "python_interfaces".to_owned(),
                Value::Object(BTreeMap::from([(
                    "relay-driver".to_owned(),
                    Value::from("8.0.0"),
                )])),
            ),
        ]),
    }
}

pub fn tracking_resource_opened_record() -> ProviderRecordOutcome {
    ProviderRecordOutcome {
        record_type: "resource.opened".to_owned(),
        timestamp: Timestamp::new("2026-01-01T00:00:02Z"),
        severity: RecordSeverity::Info,
        message: Some("Tracking resource opened.".to_owned()),
        error: None,
        details: BTreeMap::from([
            ("resource_name".to_owned(), Value::from("tracking_resource")),
            ("resource_kind".to_owned(), Value::from("tracking_tool")),
            (
                "traceability".to_owned(),
                Value::Object(BTreeMap::from([
                    (
                        "tracking_driver_version".to_owned(),
                        Value::from("2026.5.99"),
                    ),
                    ("serial_number".to_owned(), Value::from("SIM-PLT400-0001")),
                    ("production_number".to_owned(), Value::Null),
                    ("firmware_version".to_owned(), Value::from("R4.0.462")),
                ])),
            ),
            (
                "python_interfaces".to_owned(),
                Value::Object(BTreeMap::from([(
                    "tracking-driver".to_owned(),
                    Value::from("0.16.0"),
                )])),
            ),
        ]),
    }
}

pub fn samples(stream: &str, count: usize, scope: Option<RecordScope>) -> TelemetryBatch {
    let stream = super::identifier(stream);
    let samples = (0..count)
        .map(|index| {
            let mut sample = TelemetrySample::new(dokime_runtime::model::identity::new_id(), stream.clone(),
                0,
                Timestamp::new(format!(
                    "2026-01-01T{:02}:{:02}:{:02}Z",
                    index / 3_600 % 24,
                    index / 60 % 60,
                    index % 60
                )),
                "host.fixture",
            );
            sample.scope = scope.clone();
            for (column, value) in stream_columns(stream.as_str(), index) {
                sample.values.insert(column.to_owned(), value);
            }

            crate::service_support::contracts::typed_fixture_sample(sample)
        })
        .collect();

    TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream, samples)
}

/* === Private Functions === */
fn stream_columns(stream: &str, index: usize) -> Vec<(&'static str, Value)> {
    match stream {
        "host.wifi.channel_occupancy" => vec![
            ("sample_index", Value::Integer(index as i64)),
            ("wifi_channel", Value::Integer(6)),
            ("network_count", Value::Integer(index as i64)),
        ],
        "tracking" => vec![
            ("elapsed_s", Value::Number(index as f64)),
            ("target_locked", Value::Bool(true)),
        ],
        stream => panic!("no fixture columns for stream {stream}"),
    }
}

/* === Tests === */
