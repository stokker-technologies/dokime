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
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(super) async fn definitions_survive_completion(runtime: &RuntimeHandle) {
    let custom_batch = tracking_batch();
    let custom_record = custom_record();

    assert!(runtime.publish_telemetry_batch(custom_batch.clone()).await.is_err());
    assert!(runtime.publish_record(custom_record.clone()).await.is_err());

    register_custom_suite(runtime, custom_suite()).await;
    let test_cycle_id = start_cycle(runtime, custom_suite()).await;
    let receipt = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;

    assert_eq!(receipt.status, CommandStatus::Accepted);
    wait_for_cycle_outcome(runtime, &test_cycle_id, ResultStatus::Passed).await;

    let evidence = runtime.evidence_snapshot().await.unwrap();
    assert!(evidence.records.iter().any(|record| record.record_type == custom_record.record_type));
    assert!(evidence.telemetry.iter().any(|sample| sample.stream.as_str() == "tracking"));
    let post_execution_publish = runtime.publish_telemetry_batch(custom_batch).await.unwrap();
    assert_eq!(post_execution_publish.status, PublishStatus::Accepted);
}

// The custom suite is already registered by the completion stage.
pub(super) async fn conflicting_contracts_rejected(runtime: &RuntimeHandle) {
    let mut other_suite = custom_suite();
    other_suite.test_case_templates[0].test_case_template_id = TemplateId::from_seed("evidence.other.template");
    let result = runtime
        .set_suites(
            "other",
            dokime_runtime::definition::SuiteSet::new([TestSuiteDefinition {
                suite_id: "custom.suite".to_owned(),
                tags: Vec::new(),
                workflow: Some("run".to_owned()),
                suite: other_suite,
                record_contracts: vec![custom_record_contract()],
                telemetry_contracts: vec![conflicting_tracking_contract()],
            }]),
        )
        .await;

    assert!(result.as_ref().is_err_and(|detail| { detail.to_string().contains("conflicting telemetry contract") }));
}

pub(super) async fn tracking_derives_kpis() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(TrackingTransitionExecutor)).await;

    register_custom_suite(&runtime, custom_suite()).await;
    let test_cycle_id = start_cycle(&runtime, custom_suite()).await;
    let receipt = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;

    let evidence = runtime.evidence_snapshot().await.unwrap();
    let record_types = evidence.records.iter().map(|record| record.record_type.as_str()).collect::<Vec<_>>();

    assert!(record_types.contains(&"target.lost"));
    assert!(record_types.contains(&"target.acquired"));
    assert!(record_types.contains(&"tool.connection.lost"));
    assert!(record_types.contains(&"tool.connection.acquired"));
    assert!(
        evidence
            .records
            .iter()
            .filter(|record| matches!(
                record.record_type.as_str(),
                "target.lost" | "target.acquired" | "tool.connection.lost" | "tool.connection.acquired"
            ))
            .all(|record| record.source == "runtime")
    );

    let tracking_samples = evidence.telemetry.iter().filter(|sample| sample.stream.as_str() == "tracking").collect::<Vec<_>>();
    assert_eq!(tracking_samples.len(), 5);
    assert!(!tracking_samples[0].values.contains_key("speed_m_per_s"));
    assert_eq!(tracking_samples[1].values.get("speed_m_per_s"), Some(&Value::Number(1.0 / 3.0)));
    assert_eq!(tracking_samples[1].values.get("velocity_x_m_per_s"), Some(&Value::Number(1.0 / 3.0)));
    assert_eq!(tracking_samples[1].values.get("velocity_y_m_per_s"), Some(&Value::Number(0.0)));
    assert_eq!(tracking_samples[2].values.get("target_reacquire_time_s"), Some(&Value::Number(5.0)));
    assert_eq!(tracking_samples[4].values.get("connection_acquired_time_s"), Some(&Value::Number(7.0)));
    assert_eq!(tracking_samples[4].values.get("connection_loss_distance_m"), Some(&Value::Number(4.0)));
    assert_eq!(tracking_samples[4].values.get("connection_acquired_distance_m"), Some(&Value::Number(6.0)));
}

/* === Tests === */
