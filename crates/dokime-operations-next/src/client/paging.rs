/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Paging

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use crate::client::Context;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn bytes<M: prost::Message + prost_reflect::ReflectMessage>(message: &M) -> Vec<u8> {
    let value = prost_reflect::DynamicMessage::decode(message.descriptor(), message.encode_to_vec().as_slice())
        .expect("Encoded Protocol Message Matches Its Descriptor");
    let mut digest = Sha256::new();
    hash_message(&value, &mut digest);
    digest.finalize().to_vec()
}

pub(super) fn cursor(context: &Context, selection: String, rows: impl IntoIterator<Item = Vec<u8>>) -> harmos_protocol::v1::ReadCut {
    let mut digest = Sha256::new();
    for row in rows {
        digest.update((row.len() as u64).to_le_bytes());
        digest.update(row);
    }
    let bytes = digest.finalize();
    harmos_protocol::v1::ReadCut {
        positions: Vec::new(),
        revisions: vec![harmos_protocol::v1::ResourceRevision {
            name: format!("{}/{selection}", context.host.identity().backend_id),
            revision: bytes.to_vec(),
        }],
    }
}

/* === Private Functions === */

fn hash_message(message: &prost_reflect::DynamicMessage, digest: &mut Sha256) {
    use prost_reflect::ReflectMessage;
    let name = message.descriptor().full_name().to_owned();
    digest.update((name.len() as u64).to_le_bytes());
    digest.update(name.as_bytes());
    let mut fields: Vec<_> = message.fields().collect();
    fields.sort_by_key(|(field, _)| field.number());
    digest.update((fields.len() as u64).to_le_bytes());
    for (field, value) in fields {
        digest.update(field.number().to_le_bytes());
        hash_value(value, digest);
    }
}
fn hash_value(value: &prost_reflect::Value, digest: &mut Sha256) {
    use prost_reflect::Value;
    match value {
        Value::Message(message) => hash_message(message, digest),
        Value::List(values) => {
            digest.update((values.len() as u64).to_le_bytes());
            for value in values {
                hash_value(value, digest);
            }
        }
        Value::Map(values) => {
            let mut values: Vec<_> = values.iter().map(|(key, value)| (format!("{key:?}"), value)).collect();
            values.sort_by(|left, right| left.0.cmp(&right.0));
            digest.update((values.len() as u64).to_le_bytes());
            for (key, value) in values {
                digest.update((key.len() as u64).to_le_bytes());
                digest.update(key.as_bytes());
                hash_value(value, digest);
            }
        }
        value => {
            let value = format!("{value:?}");
            digest.update((value.len() as u64).to_le_bytes());
            digest.update(value.as_bytes());
        }
    }
}

/* === Tests === */
