/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Administration Objects

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use std::fmt::Display;

use crate::host::Context;
use crate::host::files::transfer;
use crate::{Error, Result};
use arrow_array::{Array, BinaryArray, StringArray, StructArray};
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::{ReadCut, Receipt};
use harmos_query::columnar::ColumnKind;
use harmos_runtime::Identified;
use harmos_runtime::builtins::storage::{ColumnSelection, ColumnSource, ListColumnRows, ListColumnRowsRequest};
use harmos_runtime::builtins::storage::{
    GetObject, GetObjectRequest, SchemaInput, SetObject, SetObjectRequest, set_object_response::Outcome,
};
use harmos_runtime::storage::{ObjectCatalog, ObjectDefinition, ObjectRetention, StoredObject};
use prost::Message;
use std::num::NonZeroUsize;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(harmos_runtime::Message, Clone, Debug, Default, PartialEq)]
pub struct Administration {
    pub key: String,
    pub payload: Vec<u8>,
}
pub struct Snapshot {
    pub payload: Vec<u8>,
    pub revision: Vec<u8>,
    pub resolved_at: Option<ReadCut>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Identified for Administration {
    const ID: &'static str = "dokime.administration";
}
impl StoredObject for Administration {
    type Key = String;
    const RETENTION: ObjectRetention = ObjectRetention::All;
    fn key(&self) -> &String {
        &self.key
    }
}

/* === Public Functions === */

pub fn catalog() -> ObjectCatalog {
    ObjectCatalog::new().entry(ObjectDefinition::of::<Administration>())
}
pub async fn read(context: &Context, key: &str, initial: &[u8], at: Option<ReadCut>) -> Result<Snapshot> {
    let definition = ObjectDefinition::of::<Administration>();
    let schema = definition.schema().map_err(|cause| invalid("Get Administration", cause))?;
    let item = catalog()
        .encode(&Administration { key: key.into(), payload: Vec::new() }, bound(context)?)
        .map_err(|cause| invalid("Get Administration", cause))?
        .item
        .to_string();
    let response = context.storage::<GetObject>(GetObjectRequest { schema: Some(schema), item, at }).await?;
    let payload = match response.value {
        Some(value) => {
            catalog().decode::<Administration>(&value, bound(context)?).map_err(|cause| invalid("Get Administration", cause))?.payload
        }
        None => initial.to_vec(),
    };
    Ok(Snapshot {
        payload,
        revision: response.revision,
        resolved_at: response.resolved_at,
    })
}
pub async fn replace(context: &Context, key: &str, expected: Vec<u8>, request_id: &str, payload: Vec<u8>) -> Result<(Snapshot, Receipt)> {
    let value = catalog()
        .encode(&Administration { key: key.into(), payload }, bound(context)?)
        .map_err(|cause| invalid("Set Administration", cause))?;
    let definition = ObjectDefinition::of::<Administration>();
    let graph = definition.graph(&Default::default()).map_err(|cause| invalid("Set Administration", cause))?;
    let mutation = transfer::mutation(context, key, request_id).await?;
    let request = SetObjectRequest {
        mutation: Some(mutation),
        create_if_absent: expected.is_empty(),
        expected_revision: expected,
        schema: Some(SchemaInput {
            key: Some(value.schema.clone()),
            graph: Some(graph),
        }),
        value: Some(value),
    };
    let response = context.storage::<SetObject>(request).await?;
    match response.outcome {
        Some(Outcome::Applied(applied)) => {
            let value = applied.value.ok_or_else(|| Error::Precondition("Set Administration: Return the Durable Value".into()))?;
            let receipt = applied.receipt.ok_or_else(|| Error::Precondition("Set Administration: Return Its Original Receipt".into()))?;
            let payload =
                catalog().decode::<Administration>(&value, bound(context)?).map_err(|cause| invalid("Set Administration", cause))?.payload;
            Ok((
                Snapshot {
                    payload,
                    revision: applied.revision,
                    resolved_at: None,
                },
                receipt,
            ))
        }
        Some(Outcome::Conflict(_)) => Err(Error::Conflict(format!("Set {key}: Refresh Its Revision Before Saving"))),
        None => Err(Error::Precondition(format!("Set {key}: Return the Mutation Outcome"))),
    }
}
pub async fn scan(context: &Context, prefix: &str, at: Option<ReadCut>) -> Result<(Vec<Administration>, Option<ReadCut>)> {
    let schema = ObjectDefinition::of::<Administration>().schema().map_err(|cause| invalid("List Administration", cause))?;
    let mut at = at;
    let mut token = String::new();
    let mut entries = Vec::new();
    loop {
        let response = context
            .storage::<ListColumnRows>(ListColumnRowsRequest {
                selection: Some(ColumnSelection {
                    source: Some(ColumnSource {
                        kind: ColumnKind::Objects,
                        schema: schema.clone(),
                    }),
                    columns: vec!["payload.key".into(), "payload.payload".into()],
                    predicate: None,
                    time: None,
                    start: 0,
                    work: None,
                }),
                at: at.clone(),
                page: Some(PageRequest {
                    page_size: context.settings.page_items as u32,
                    page_token: token,
                    ..Default::default()
                }),
            })
            .await?;
        at = response.resolved_at;
        if let Some(page) = response.page {
            let payload = page
                .batch
                .column_by_name("payload")
                .and_then(|column| column.as_any().downcast_ref::<StructArray>())
                .ok_or_else(|| Error::Precondition("List Administration: Restore Its Payload Columns".into()))?;
            let keys = payload
                .column_by_name("key")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| Error::Precondition("List Administration: Restore Its Key Column".into()))?;
            let values = payload
                .column_by_name("payload")
                .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
                .ok_or_else(|| Error::Precondition("List Administration: Restore Its Value Column".into()))?;
            for index in 0..keys.len() {
                if keys.is_null(index) || values.is_null(index) {
                    return Err(Error::Precondition("List Administration: Restore Its Complete Object".into()));
                }
                if keys.value(index).starts_with(prefix) {
                    entries.push(Administration {
                        key: keys.value(index).into(),
                        payload: values.value(index).to_vec(),
                    });
                }
            }
        }
        token = response.next_page_token;
        if token.is_empty() {
            break;
        }
    }
    entries.sort_by(|left, right| left.key.cmp(&right.key));
    Ok((entries, at))
}
pub async fn message<T: Message + Default>(context: &Context, key: &str) -> Result<(T, Vec<u8>)> {
    let initial = T::default().encode_to_vec();
    let snapshot = read(context, key, &initial, None).await?;
    Ok((T::decode(snapshot.payload.as_slice())?, snapshot.revision))
}

/* === Private Functions === */

fn bound(context: &Context) -> Result<NonZeroUsize> {
    NonZeroUsize::new(context.settings.message_bytes).ok_or_else(|| Error::Bound("Access Administration: Configure a Message Bound".into()))
}
fn invalid(action: &str, cause: impl Display) -> Error {
    Error::Invalid(format!("{action}: {cause}"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Administration, catalog};
    use std::num::NonZeroUsize;
    #[test]
    fn native_envelope_preserves_protobuf_bytes_and_distinct_object_keys() {
        let bound = NonZeroUsize::new(4096).unwrap();
        let original = Administration {
            key: "preferences".into(),
            payload: vec![8, 1, 18, 2, 255, 0],
        };
        let encoded = catalog().encode(&original, bound).unwrap();
        let other = catalog()
            .encode(
                &Administration {
                    key: "sources".into(),
                    payload: original.payload.clone(),
                },
                bound,
            )
            .unwrap();
        assert_ne!(encoded.item, other.item);
        assert_eq!(catalog().decode::<Administration>(&encoded, bound).unwrap(), original);
    }
}
