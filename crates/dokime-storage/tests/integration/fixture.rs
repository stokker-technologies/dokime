/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Frozen Stream Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use prost_types::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    field_descriptor_proto::{Label, Type},
};

// Local Imports

use dokime_storage::{Execution, Sample};

use dokime_protocol::model::Import;
use dokime_protocol::model::Origin;
use dokime_protocol::model::origin;
use harmos_protocol::model::Declarations;
use harmos_protocol::model::StreamDeclaration;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
pub struct Row {
    #[prost(uint64, tag = "1")]
    pub value: u64,

    #[prost(string, tag = "2")]
    pub label: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn execution() -> Execution {
    let fields = [("value", Type::Uint64), ("label", Type::String)]
        .into_iter()
        .enumerate()
        .map(|(i, (name, kind))| FieldDescriptorProto {
            name: Some(name.into()),
            number: Some(i as i32 + 1),
            label: Some(Label::Optional as i32),
            r#type: Some(kind as i32),
            ..Default::default()
        })
        .collect();
    let types = FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("fixture.proto".into()),
            package: Some("fixture".into()),
            syntax: Some("proto3".into()),
            message_type: vec![DescriptorProto {
                name: Some("Row".into()),
                field: fields,
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    Execution {
        identifier: "execution".into(),
        cycle: "cycle".into(),
        origin: Some(Origin {
            kind: Some(origin::Kind::Import(Import::default())),
        }),
        catalog: Declarations {
            types: Some(types),
            streams: vec![StreamDeclaration {
                id: "readings".into(),
                row: "fixture.Row".into(),
                ..Default::default()
            }],
            ..Default::default()
        }
        .encode_to_vec(),
        ..Default::default()
    }
}
pub fn samples(count: usize) -> Vec<Sample> {
    (0..count)
        .map(|i| Sample {
            stream: "readings".into(),
            source_ns: i as i64,
            receipt_ns: i as i64,
            sequence: i as u64,
            row: Row {
                value: i as u64,
                label: format!("group{}", i % 3),
            }
            .encode_to_vec(),
            ..Default::default()
        })
        .collect()
}

/* === Private Functions === */

/* === Tests === */
