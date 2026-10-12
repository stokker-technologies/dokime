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
use prost::Message;
// Local Imports
use dokime_protocol::{FILE_DESCRIPTOR_SET, v1};

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

pub(crate) fn signed_integer_precision() {
    for integer in [i64::MIN, -(1_i64 << 53) - 1, 0, (1_i64 << 53) + 1, i64::MAX] {
        let value = v1::Value {
            kind: Some(v1::value::Kind::IntegerValue(integer)),
        };
        let decoded = v1::Value::decode(value.encode_to_vec().as_slice()).unwrap();
        assert_eq!(decoded, value);
    }
}

pub(crate) fn default_presence_contract() {
    let absent = v1::RecordField::default();
    let null = v1::RecordField {
        default_value: Some(v1::Value {
            kind: Some(v1::value::Kind::NullValue(v1::NullValue::Null as i32)),
        }),
        ..Default::default()
    };
    let zero = v1::RecordField {
        default_value: Some(v1::Value {
            kind: Some(v1::value::Kind::IntegerValue(0)),
        }),
        ..Default::default()
    };
    assert_ne!(absent.encode_to_vec(), null.encode_to_vec());
    assert_ne!(null.encode_to_vec(), zero.encode_to_vec());
    for field in [absent, null, zero] {
        assert_eq!(
            v1::RecordField::decode(field.encode_to_vec().as_slice()).unwrap(),
            field
        );
    }
}

pub(crate) fn descriptors_include_services() {
    let set = prost_types::FileDescriptorSet::decode(FILE_DESCRIPTOR_SET).unwrap();
    assert!(set.file.iter().all(|file| file.source_code_info.is_none()));
    let backend = set
        .file
        .iter()
        .flat_map(|file| &file.service)
        .find(|service| service.name.as_deref() == Some("Backend"))
        .unwrap();
    let methods: Vec<_> = backend.method.iter().map(|method| method.name()).collect();
    assert_eq!(
        methods,
        [
            "GetIdentity",
            "GetHealth",
            "GetCapabilities",
            "GetDiagnostics",
            "CollectDiagnostics",
            "WatchLogs"
        ]
    );
}

pub(crate) fn retry_receipt_contract() {
    let absent = v1::RunCycleRequest::default();
    let empty = v1::RunCycleRequest {
        idempotency_key: Some(String::new()),
        ..Default::default()
    };
    assert_ne!(absent.encode_to_vec(), empty.encode_to_vec());
    let receipt = v1::RunRequestReceipt {
        planned_count: u64::MAX,
        phase: v1::RunRequestPhase::Interrupted as i32,
        interruption: Some("restart".into()),
        ..Default::default()
    };
    assert_eq!(
        v1::RunRequestReceipt::decode(receipt.encode_to_vec().as_slice()).unwrap(),
        receipt
    );
    let set = prost_types::FileDescriptorSet::decode(FILE_DESCRIPTOR_SET).unwrap();
    let mutation = set
        .file
        .iter()
        .flat_map(|file| &file.message_type)
        .find(|message| message.name() == "MutationContext")
        .unwrap();
    assert_eq!(
        mutation
            .field
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        ["correlation_id"]
    );
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: signed_integer_precision");
        signed_integer_precision();
    }
    {
        eprintln!("Scenario: default_presence_contract");
        default_presence_contract();
    }
    {
        eprintln!("Scenario: descriptors_include_services");
        descriptors_include_services();
    }
    {
        eprintln!("Scenario: retry_receipt_contract");
        retry_receipt_contract();
    }
}
