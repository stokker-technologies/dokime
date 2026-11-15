/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SPDX-License-Identifier: Apache-2.0

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::ExecutionProgressValue;
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_storage::ExecutionSnapshot;
use dokime_storage::StatisticsRead;
use harmos_protocol::model::Declarations;
use prost::Message;
use prost_reflect::DescriptorPool;
use prost_reflect::DynamicMessage;
use prost_reflect::Value;
use std::collections::BTreeSet;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Compare Shared Evidence Values and Ordering Without Per-Run Identities or Authored Prose.
pub fn equivalent(expected: &ExecutionSnapshot, actual: &ExecutionSnapshot) {
    // Resolve the Retained Schemas Without a Live Plugin
    let catalog = Declarations::decode(actual.execution.catalog.as_slice()).unwrap();
    let pool = DescriptorPool::from_file_descriptor_set(catalog.types.unwrap()).unwrap();
    let temperature = pool
        .get_message_by_name(&catalog.streams.iter().find(|stream| stream.id == "thermal.temperature").unwrap().row)
        .expect("Retain the Temperature Stream Schema");
    for sample in actual.samples.iter().filter(|sample| sample.stream == "thermal.temperature") {
        let decoded = DynamicMessage::decode(temperature.clone(), sample.row.as_slice()).unwrap();
        assert_eq!(decoded.get_field_by_name("celsius").unwrap().as_ref(), &Value::F64(84.0));
    }

    let mut origin = actual.execution.origin.clone().unwrap();
    let mut reference = expected.execution.origin.clone().unwrap();
    for value in [&mut origin, &mut reference] {
        let Some(OriginKind::PluginRun(run)) = value.kind.as_mut() else { panic!("Expected PluginRun") };
        run.digest.clear();
        run.started_at = None;
        let case = run.test_case.as_mut().unwrap();
        for metadata in case.steps.iter_mut().filter_map(|step| step.metadata.as_mut()) {
            metadata.label.clear();
            metadata.description.clear();
        }
        for metadata in case.criteria.iter_mut().filter_map(|criterion| criterion.metadata.as_mut()) {
            metadata.label.clear();
            metadata.description.clear();
        }
    }
    assert_eq!(origin, reference, "Frozen Origin Including the Complete Test Case");
    assert_eq!(actual.state, expected.state);
    assert_eq!(actual.execution.termination, expected.execution.termination);
    assert_eq!(actual.execution.outcome, expected.execution.outcome);
    let normalize = |snapshot: &ExecutionSnapshot| {
        let mut request_ids = BTreeSet::new();
        snapshot
            .evidence
            .iter()
            .cloned()
            .map(|mut entry| {
                entry.receipt_ns = 0;
                if entry.kind.starts_with("record:") {
                    let mut value = RecordsAppendRequest::decode(entry.payload.as_slice()).unwrap();
                    assert!(!value.request_id.is_empty());
                    assert!(request_ids.insert(value.request_id.clone()));
                    value.request_id.clear();
                    value.execution.clear();
                    value.source_at = None;
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "read:statistics" {
                    let mut value = StatisticsRead::decode(entry.payload.as_slice()).unwrap();
                    let request = value.request.as_mut().unwrap();
                    request.scope = None;
                    if request.between.as_ref().is_some_and(|window| window == &Default::default()) {
                        request.between = None;
                    }
                    let selected = snapshot
                        .samples
                        .iter()
                        .filter(|sample| sample.stream == request.stream && snapshot.step_contains(&request.step, sample.position))
                        .collect::<Vec<_>>();
                    assert_eq!(value.positions, selected.iter().map(|sample| sample.position).collect::<Vec<_>>());
                    value.positions = selected.iter().map(|sample| sample.sequence).collect();
                    let request_cut = value.request.as_mut().and_then(|request| request.at.as_mut());
                    let response_cut = value.response.as_mut().and_then(|response| response.resolved_at.as_mut());
                    assert!(response_cut.is_some());
                    for cut in request_cut.into_iter().chain(response_cut) {
                        for position in &mut cut.positions {
                            let history = position.history.as_mut().unwrap();
                            assert!(!history.store.is_empty() && !history.epoch.is_empty());
                            history.store.clear();
                            history.epoch.clear();
                        }
                    }
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "read:records" {
                    let mut value = RecordsListRequest::decode(entry.payload.as_slice()).unwrap();
                    value.scope = None;
                    if let Some(scope) = value.selection.as_mut().and_then(|selection| selection.scope.as_mut()) {
                        scope.execution_id = None;
                    }
                    if value.page.as_ref().is_some_and(|page| page == &Default::default()) {
                        value.page = None;
                    }
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "log" {
                    let mut value = LogsAppendRequest::decode(entry.payload.as_slice()).unwrap();
                    assert!(!value.request_id.is_empty());
                    assert!(request_ids.insert(value.request_id.clone()));
                    value.request_id.clear();
                    value.message.clear();
                    value.execution.clear();
                    value.at = None;
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "progress" {
                    let mut value = ExecutionProgressValue::decode(entry.payload.as_slice()).unwrap();
                    value.label.clear();
                    entry.payload = value.encode_to_vec();
                }

                entry
            })
            .collect::<Vec<_>>()
    };
    let actual_evidence = normalize(actual);
    let expected_evidence = normalize(expected);
    assert_eq!(actual_evidence.len(), expected_evidence.len());
    for (actual, expected) in actual_evidence.iter().zip(&expected_evidence) {
        assert_eq!(actual, expected, "Ordered Evidence Including Step Boundaries and Record Bytes");
    }
    let samples = |snapshot: &ExecutionSnapshot| {
        let mut samples = snapshot
            .samples
            .iter()
            .cloned()
            .map(|mut sample| {
                sample.receipt_ns = 0;
                sample.position = 0;
                sample
            })
            .collect::<Vec<_>>();
        samples.sort_by(|left, right| (&left.stream, left.sequence).cmp(&(&right.stream, right.sequence)));
        samples
    };
    let actual_samples = samples(actual);
    let expected_samples = samples(expected);
    assert_eq!(actual_samples.len(), expected_samples.len());
    for (actual, expected) in actual_samples.iter().zip(&expected_samples) {
        assert_eq!(actual, expected, "Samples Including Source Times and Stream Sequence");
    }
    assert!(!actual.seal.is_empty());
}

/// Decode a Record Using Only Its Retained Schema and Exact Stored Payload.
pub fn frozen_record_schema_decodes_door(actual: &ExecutionSnapshot) {
    let catalog = Declarations::decode(actual.execution.catalog.as_slice()).unwrap();
    let pool = DescriptorPool::from_file_descriptor_set(catalog.types.unwrap()).unwrap();
    let door = pool.all_messages().find(|message| message.name() == "Door").expect("Retain the Door Record Schema");
    for record in actual.records("thermal.door") {
        let record = RecordsAppendRequest::decode(record.payload.as_slice()).unwrap();
        let decoded = DynamicMessage::decode(door.clone(), record.payload.as_slice()).unwrap();
        assert_eq!(decoded.get_field_by_name("closed").unwrap().as_ref(), &Value::Bool(true));
    }
}

/* === Private Functions === */

/* === Tests === */
