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
use std::collections::BTreeSet;

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_protocol::context::evidence::{LogsAppendRequest, RecordsAppendRequest, RecordsListRequest};
use dokime_protocol::v1;
use dokime_storage::{ExecutionSnapshot, StatisticsRead};

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
    let catalog = harmos_protocol::v1::Catalog::decode(actual.execution.catalog.as_slice()).unwrap();
    let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(catalog.types.unwrap()).unwrap();
    let temperature = pool.get_message_by_name("thermal.v1.Temperature").expect("Retain the Temperature Stream Schema");
    for sample in &actual.samples {
        let decoded = prost_reflect::DynamicMessage::decode(temperature.clone(), sample.row.as_slice()).unwrap();
        assert_eq!(decoded.get_field_by_name("celsius").unwrap().as_ref(), &prost_reflect::Value::F64(84.0));
    }

    let mut origin = actual.execution.origin.clone().unwrap();
    let mut reference = expected.execution.origin.clone().unwrap();
    for value in [&mut origin, &mut reference] {
        let Some(v1::origin::Kind::PluginRun(run)) = value.kind.as_mut() else { panic!("Expected PluginRun") };
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
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "read:statistics" {
                    let mut value = StatisticsRead::decode(entry.payload.as_slice()).unwrap();
                    value.request.as_mut().unwrap().execution.clear();
                    let request_cut = value.request.as_mut().and_then(|request| request.at.as_mut());
                    let response_cut = value.response.as_mut().and_then(|response| response.resolved_at.as_mut());
                    assert!(response_cut.is_some());
                    for cut in request_cut.into_iter().chain(response_cut) {
                        let history = cut.history.as_mut().unwrap();
                        assert!(!history.store.is_empty() && !history.epoch.is_empty());
                        history.store.clear();
                        history.epoch.clear();
                    }
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "read:records" {
                    let mut value = RecordsListRequest::decode(entry.payload.as_slice()).unwrap();
                    value.execution.clear();
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
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "progress" {
                    let mut value = v1::ExecutionProgressValue::decode(entry.payload.as_slice()).unwrap();
                    value.label.clear();
                    entry.payload = value.encode_to_vec();
                }

                entry
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(normalize(actual), normalize(expected), "Ordered Evidence Including Step Boundaries and Record Bytes");
    let samples = |snapshot: &ExecutionSnapshot| {
        snapshot
            .samples
            .iter()
            .cloned()
            .map(|mut sample| {
                sample.receipt_ns = 0;
                sample
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(samples(actual), samples(expected), "Samples Including Source Times, Positions and Sequence");
    assert!(!actual.seal.is_empty());
}

/// Decode a Record Using Only Its Retained Schema and Exact Stored Payload.
pub fn frozen_record_schema_decodes_door(actual: &ExecutionSnapshot) {
    let catalog = harmos_protocol::v1::Catalog::decode(actual.execution.catalog.as_slice()).unwrap();
    let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(catalog.types.unwrap()).unwrap();
    let door = pool.get_message_by_name("thermal.v1.Door").expect("Retain the Door Record Schema");
    for record in actual.records("thermal.door") {
        let record = dokime_protocol::context::evidence::RecordsAppendRequest::decode(record.payload.as_slice()).unwrap();
        let decoded = prost_reflect::DynamicMessage::decode(door.clone(), record.payload.as_slice()).unwrap();
        assert_eq!(decoded.get_field_by_name("closed").unwrap().as_ref(), &prost_reflect::Value::Bool(true));
    }
}

/* === Private Functions === */

/* === Tests === */
