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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

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

pub(super) fn cursor(context: &Context, selection: String, rows: impl IntoIterator<Item = Vec<u8>>) -> v1::Cursor {
    let mut digest = Sha256::new();
    for row in rows {
        digest.update((row.len() as u64).to_le_bytes());
        digest.update(row);
    }
    let bytes = digest.finalize();
    v1::Cursor {
        epoch: context.host.identity().epoch,
        stream: selection,
        position: u64::from_le_bytes(bytes[..8].try_into().unwrap()),
    }
}

pub(super) fn page<T>(
    mut rows: Vec<T>, requested: u32, after: &str, supplied: Option<&v1::Cursor>, current: &v1::Cursor, maximum: usize,
    id: impl Fn(&T) -> &str,
) -> Result<(Vec<T>, Option<String>)> {
    if supplied.is_some_and(|cursor| cursor != current) || (!after.is_empty() && supplied.is_none()) {
        return Err(Error::Precondition("Restart Paging with the Same Selection and a Fresh Cursor".into()));
    }
    let start = if after.is_empty() {
        0
    } else {
        rows.iter()
            .position(|row| id(row) == after)
            .ok_or_else(|| Error::Invalid("Restart Paging: the Continuation Is Outside the Selection".into()))?
            + 1
    };
    let limit = super::projection::limit(requested, maximum);
    let remaining = rows.split_off(start);
    let more = remaining.len() > limit;
    let rows: Vec<T> = remaining.into_iter().take(limit).collect();
    let next = more.then(|| id(rows.last().unwrap()).to_owned());
    Ok((rows, next))
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
