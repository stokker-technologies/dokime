/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Frozen Protobuf Decoding Bindings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::Display,
    num::{NonZeroU32, NonZeroUsize},
};

// Third Party Imports
use arrow_array::StructArray;
use harmos_domain::schema::{
    Cardinality, Described,
    graph::{Field, Kind, Schema},
    identity,
};
use harmos_domain::{Identified, Identifier};
use harmos_storage::{PayloadEncoding, SchemaKey};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage};
use prost_types::FileDescriptorSet;

// Local Imports
use super::records::Record;
use crate::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Original Protobuf Bytes and the Descriptor Set That Owns Their Interpretation.
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Protobuf {
    pub message: String,
    pub descriptors: Vec<u8>,
    pub payload: Vec<u8>,
}

/// A Plugin Declaration's Native Graph and Its Frozen Product Decoding Binding.
/// Samples Use This Graph Directly; They Never Persist a Protobuf Row Copy.
#[derive(Clone, Debug)]
pub struct Binding {
    pub key: SchemaKey,
    pub graph: Schema,
    pub message: String,
    pub descriptors: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Protobuf {
    pub fn decode(&self, bytes: NonZeroUsize) -> Result<DynamicMessage> {
        let retained = self.message.len().saturating_add(self.descriptors.len()).saturating_add(self.payload.len());
        if retained > bytes.get() {
            return Err(Error::Invalid("Decode Frozen Value: Select a Value Within the Message Bound".into()));
        }
        let pool = DescriptorPool::decode(self.descriptors.as_slice()).map_err(invalid)?;
        let descriptor = pool
            .get_message_by_name(&self.message)
            .ok_or_else(|| Error::Invalid("Decode Frozen Value: Restore Its Declared Message Descriptor".into()))?;
        let value = DynamicMessage::decode(descriptor, self.payload.as_slice())?;
        if value.unknown_fields().next().is_some() {
            return Err(Error::Invalid("Decode Frozen Value: Include Every Field in Its Frozen Descriptor Set".into()));
        }
        Ok(value)
    }

    pub fn product<M: Message>(message: &str, value: &M, bytes: NonZeroUsize) -> Result<Self> {
        let descriptor = dokime_protocol::descriptor_pool()
            .get_message_by_name(message)
            .ok_or_else(|| Error::Invalid("Freeze Product Value: Select a Declared Message".into()))?;
        let mut pending = vec![descriptor.parent_file()];
        let mut files = BTreeMap::new();
        while let Some(file) = pending.pop() {
            if files.contains_key(file.name()) {
                continue;
            }
            pending.extend(file.dependencies());
            files.insert(file.name().to_owned(), file.file_descriptor_proto().clone());
        }
        let saved = Self {
            message: message.into(),
            descriptors: FileDescriptorSet { file: files.into_values().collect() }.encode_to_vec(),
            payload: value.encode_to_vec(),
        };
        saved.decode(bytes)?;
        Ok(saved)
    }
}

impl Binding {
    pub fn validate(&self, bytes: NonZeroUsize) -> Result<()> {
        self.graph.prepare(&Default::default()).map_err(invalid)?;
        let value = Protobuf {
            message: self.message.clone(),
            descriptors: self.descriptors.clone(),
            payload: Vec::new(),
        };
        value.decode(bytes)?;
        Ok(())
    }

    /// Compose the Product Context With a Plugin's Typed Value Without a Byte Envelope.
    pub fn record(&self) -> Result<Self> {
        let mut graph = Schema::from_declaration(Record::DEFINITION, &Default::default()).map_err(invalid)?;
        let offset = graph.definitions.len();
        for definition in &self.graph.definitions {
            let fields = definition
                .iter()
                .cloned()
                .map(|mut field| {
                    if let Kind::Message { definition } = &mut field.kind {
                        *definition += offset;
                    }
                    field
                })
                .collect();
            graph.definitions.push(fields);
        }
        graph
            .representations
            .extend(self.graph.representations.iter().map(|(index, representation)| (index + offset, representation.clone())));
        let tag = graph.definitions[graph.root]
            .iter()
            .map(|field| field.tag)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| Error::Invalid("Declare Typed Record: Reserve a Value Field Tag".into()))?;
        graph.definitions[graph.root].push(Field {
            name: Cow::Borrowed("value"),
            tag,
            kind: Kind::Message { definition: self.graph.root + offset },
            cardinality: Cardinality::Required,
        });
        graph.prepare(&Default::default()).map_err(invalid)?;
        let digest = identity::fingerprint(graph.root().map_err(invalid)?, "postcard.v1").map_err(invalid)?;
        let id = Identifier::new(format!("dokime.record/{}/{digest}", self.key.id)).map_err(invalid)?;
        let version = NonZeroU32::new(1).unwrap();
        let key = SchemaKey {
            id,
            version,
            payload_encoding: PayloadEncoding::PostcardV1,
        };
        Ok(Self { key, graph, ..self.clone() })
    }

    /// Reuse the Library Codec for Typed Record or Sample Columns.
    pub fn encode_columns(&self, payload: &StructArray, bytes: NonZeroUsize) -> Result<Vec<Vec<u8>>> {
        harmos_storage::writes::convert::encode_payloads(&self.graph, payload, bytes).map_err(Error::Query)
    }
}

/* === Public Functions === */

pub fn schema<T: Identified + Described>() -> Result<(SchemaKey, Schema)> {
    let graph = Schema::from_declaration(T::DEFINITION, &Default::default()).map_err(invalid)?;
    let (identity, version) = T::definition();
    let key = SchemaKey {
        id: Identifier::new(identity.as_str()).map_err(|source| Error::Invalid(format!("Declare Persisted Schema: {source}")))?,
        version: NonZeroU32::new(version).ok_or_else(|| Error::Invalid("Declare Persisted Schema: Supply a Positive Version".into()))?,
        payload_encoding: PayloadEncoding::PostcardV1,
    };
    Ok((key, graph))
}

/* === Private Functions === */

fn invalid(source: impl Display) -> Error {
    Error::Invalid(format!("Freeze Persisted Binding: Restore Its Complete Schema and Descriptor Set: {source}"))
}

/* === Tests === */
