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
use prost::Message;

// Local Imports
use dokime_protocol::v1;
use dokime_storage::ExecutionSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Compare every shared evidence field, excluding only per-run identity and receipt time.
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
    }
    assert_eq!(origin, reference, "Frozen Origin Including the Complete Test Case");
    assert_eq!(actual.state, expected.state);
    assert_eq!(actual.execution.termination, expected.execution.termination);
    assert_eq!(actual.execution.outcome, expected.execution.outcome);
    let normalize = |snapshot: &ExecutionSnapshot| {
        snapshot
            .evidence
            .iter()
            .cloned()
            .map(|mut entry| {
                entry.receipt_ns = 0;
                if entry.kind.starts_with("record:") {
                    let mut value = dokime_protocol::context::evidence::RecordsAppendRequest::decode(entry.payload.as_slice()).unwrap();
                    value.execution.clear();
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "read:statistics" {
                    let mut value = dokime_protocol::context::EvidenceStatisticsRequest::decode(entry.payload.as_slice()).unwrap();
                    value.execution.clear();
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "read:records" {
                    let mut value = dokime_protocol::context::evidence::RecordsListRequest::decode(entry.payload.as_slice()).unwrap();
                    value.execution.clear();
                    entry.payload = value.encode_to_vec();
                }
                if entry.kind == "log" {
                    let mut value = dokime_protocol::context::evidence::LogsAppendRequest::decode(entry.payload.as_slice()).unwrap();
                    value.execution.clear();
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
