/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Typed Telemetry Storage Codec

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{borrow::Cow, collections::BTreeMap, num::NonZeroUsize};

// Third Party Imports
use arrow_array::{
    Array, StringArray, StructArray, TimestampNanosecondArray, UInt32Array, UInt64Array,
};
use harmos::{
    Message,
    message::postcard::{Reader, Value as PostcardValue},
    schema::{
        Cardinality, Described, Event, PathElement, Scalar, Validator,
        graph::{Field, Kind, Schema},
    },
    storage::{PayloadEncoding, SchemaKey, arrow::ArrowBatch},
};

// Local Imports
use super::{Sample, TelemetryEntry, sample};
use crate::{
    core::{Identifier, SessionId, Value},
    storage::{Error, Result, records::ScopeRow, values},
    telemetry::TelemetryContract,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug)]
pub struct TypedCodec {
    schema: SchemaKey,
    graph: Schema,
    validator: Validator,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
struct Metadata {
    epoch: String,
    ordinal: u64,
    sample_id: String,
    stream: String,
    schema_version: u32,
    sequence: u64,
    timestamp: String,
    timestamp_value: Vec<values::Cell>,
    received_at: String,
    source: String,
    scope: Option<ScopeRow>,
    summary: Vec<values::Cell>,
    metadata: Vec<values::Cell>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl TypedCodec {
    pub fn new(
        stream: &Identifier,
        version: u32,
        payload: &Schema,
        settings: &harmos::config::SchemaSettings,
    ) -> Result<Self> {
        let version = std::num::NonZeroU32::new(version).ok_or(Error::Invalid(
            "typed telemetry schema version must be positive",
        ))?;
        let validator = Validator::from_schema(payload, settings).map_err(|_| {
            Error::Invalid("typed telemetry graph is invalid or exceeds schema limits")
        })?;
        let metadata = Schema::from_declaration(Metadata::DEFINITION, settings)
            .map_err(|_| Error::Invalid("telemetry metadata graph exceeds schema limits"))?;
        let payload_offset = metadata
            .definitions
            .len()
            .checked_add(1)
            .ok_or(Error::Invalid("telemetry graph definition count overflows"))?;
        let mut definitions = vec![vec![
            Field {
                name: Cow::Borrowed("metadata"),
                tag: 1,
                kind: Kind::Message {
                    definition: metadata.root + 1,
                },
                cardinality: Cardinality::Required,
            },
            Field {
                name: Cow::Borrowed("values"),
                tag: 2,
                kind: Kind::Message {
                    definition: payload.root + payload_offset,
                },
                cardinality: Cardinality::Required,
            },
        ]];
        for (graph, offset) in [(&metadata, 1), (payload, payload_offset)] {
            for fields in &graph.definitions {
                let mut fields = fields.clone();
                for field in &mut fields {
                    if let Kind::Message { definition } = &mut field.kind {
                        *definition = definition
                            .checked_add(offset)
                            .ok_or(Error::Invalid("telemetry graph reference overflows"))?;
                    }
                }
                definitions.push(fields);
            }
        }
        let graph = Schema {
            root: 0,
            definitions,
        };
        graph
            .prepare(settings)
            .map_err(|_| Error::Invalid("composed telemetry graph exceeds schema limits"))?;
        let schema = SchemaKey {
            id: Identifier::new(format!("dokime.telemetry.{}", stream.as_str()))?,
            version,
            payload_encoding: PayloadEncoding::PostcardV1,
        };
        Ok(Self {
            schema,
            graph,
            validator,
        })
    }

    pub fn schema(&self) -> &SchemaKey {
        &self.schema
    }
    pub fn graph(&self) -> &Schema {
        &self.graph
    }

    pub fn encoding_memory(&self, entry: &TelemetryEntry) -> Result<usize> {
        let metadata = Sample::encoding_memory(&entry.sample)?;
        let payload = entry.sample.storage_payload.as_ref().ok_or(Error::Invalid(
            "typed telemetry sample has no original row payload",
        ))?;
        metadata
            .checked_add(entry.epoch.as_str().len())
            .and_then(|bytes| {
                payload
                    .len()
                    .checked_mul(2)
                    .and_then(|payload| bytes.checked_add(payload))
            })
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Metadata>()))
            .ok_or(Error::Invalid("typed telemetry encoding memory overflows"))
    }

    pub fn encode(&self, entry: &TelemetryEntry, maximum: usize) -> Result<Vec<u8>> {
        let sample = &entry.sample;
        if sample.schema_version != self.schema.version.get()
            || self.schema.id.as_str() != format!("dokime.telemetry.{}", sample.stream.as_str())
        {
            return Err(Error::Invalid(
                "typed telemetry sample differs from registered stream schema",
            ));
        }
        let payload = sample.storage_payload.as_deref().ok_or(Error::Invalid(
            "typed telemetry sample has no original row payload",
        ))?;
        self.validator
            .validate(payload)
            .map_err(|_| Error::Invalid("typed telemetry row does not match its declared graph"))?;
        let metadata = Metadata {
            epoch: sample::copy_text(entry.epoch.as_str())?,
            ordinal: entry.ordinal,
            sample_id: sample::copy_text(sample.sample_id.as_str())?,
            stream: sample::copy_text(sample.stream.as_str())?,
            schema_version: sample.schema_version,
            sequence: sample.sequence,
            timestamp: sample::copy_text(sample.timestamp.as_str())?,
            timestamp_value: values::encode(
                &sample
                    .values
                    .get("timestamp")
                    .map(|value| BTreeMap::from([("timestamp".to_owned(), value.clone())]))
                    .unwrap_or_default(),
            )?,
            received_at: sample::copy_text(sample.received_at.as_str())?,
            source: sample::copy_text(sample.source.as_str())?,
            scope: sample.scope.as_ref().map(ScopeRow::from),
            summary: values::encode(&sample.summary)?,
            metadata: values::encode(&sample.metadata)?,
        };
        sample.timestamp.to_datetime()?;
        sample.received_at.to_datetime()?;
        let length = metadata
            .encoded_len()
            .checked_add(payload.len())
            .filter(|length| *length <= maximum)
            .ok_or(Error::Invalid("typed telemetry frame exceeds byte limit"))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| Error::Invalid("typed telemetry allocation failed"))?;
        // Postcard Struct Fields Are Sequential; Nested Messages Add No Length Prefix.
        metadata.encode_into(&mut bytes);
        bytes.extend_from_slice(payload);
        Ok(bytes)
    }

    pub fn decoding_memory(&self, bytes: &[u8]) -> Result<usize> {
        let mut reader = Reader::new(bytes);
        let metadata = <Metadata as PostcardValue>::memory(&mut reader)?;
        let payload = reader.remaining();
        let projected = projection_memory(&self.validator, payload)?;
        metadata
            .checked_mul(3)
            .and_then(|bytes| bytes.checked_add(projected))
            .and_then(|bytes| bytes.checked_add(payload.len()))
            .ok_or(Error::Invalid("typed telemetry decoding memory overflows"))
    }

    pub fn decode(&self, bytes: &[u8], maximum: usize) -> Result<TelemetryEntry> {
        if bytes.len() > maximum {
            return Err(Error::Invalid("typed telemetry frame exceeds byte limit"));
        }
        let allowance = NonZeroUsize::new(self.decoding_memory(bytes)?).ok_or(Error::Invalid(
            "typed telemetry decoding allowance is empty",
        ))?;
        let mut reader = Reader::new(bytes);
        let metadata = <Metadata as PostcardValue>::read(&mut reader)?;
        let payload = reader.remaining();
        let metadata_length = bytes.len() - payload.len();
        if metadata.encode_to_vec() != bytes[..metadata_length]
            || metadata.epoch.bytes().any(|byte| byte.is_ascii_uppercase())
        {
            return Err(Error::Invalid("stored telemetry metadata is not canonical"));
        }
        let mut decoded = decode_stream_values(&metadata.stream, &self.validator, payload)?;
        let mut timestamp = values::decode(&metadata.timestamp_value)?;
        if timestamp.len() > 1 || timestamp.keys().any(|key| key != "timestamp") {
            return Err(Error::Invalid(
                "stored telemetry timestamp projection is invalid",
            ));
        }
        if let Some(timestamp) = timestamp.remove("timestamp") {
            if decoded
                .get("timestamp")
                .is_some_and(|value| value != &timestamp)
            {
                return Err(Error::Invalid(
                    "stored telemetry timestamp differs from its payload",
                ));
            }
            decoded.insert("timestamp".to_owned(), timestamp);
        }
        if metadata.schema_version != self.schema.version.get()
            || self.schema.id.as_str() != format!("dokime.telemetry.{}", metadata.stream)
        {
            return Err(Error::Invalid(
                "stored telemetry metadata differs from its schema",
            ));
        }
        let mut sample = Sample {
            sample_id: metadata.sample_id,
            stream: metadata.stream,
            schema_version: metadata.schema_version,
            sequence: metadata.sequence,
            timestamp: metadata.timestamp,
            received_at: metadata.received_at,
            source: metadata.source,
            scope: metadata.scope,
            values: values::encode(&Default::default())?,
            summary: metadata.summary,
            metadata: metadata.metadata,
        }
        .into_sample(allowance)?;
        sample.values = decoded;
        sample.storage_payload = Some(payload.to_vec());
        Ok(TelemetryEntry {
            epoch: SessionId::from_uuid(metadata.epoch)?,
            ordinal: metadata.ordinal,
            sample,
        })
    }
}

/* === Public Functions === */
pub fn codec(contract: &TelemetryContract) -> Result<TypedCodec> {
    let schema = contract.storage_schema.as_ref().ok_or(Error::Invalid(
        "captured telemetry has no typed storage schema",
    ))?;
    TypedCodec::new(
        &contract.stream,
        contract.schema_version,
        schema,
        &harmos::config::SchemaSettings::default(),
    )
}

pub fn decode_batch(codec: &TypedCodec, batch: &ArrowBatch) -> Result<Vec<TelemetryEntry>> {
    let encoded = encode_batch(codec, batch)?;
    let entries = encoded
        .iter()
        .map(|row| codec.decode(row, usize::MAX))
        .collect::<Result<Vec<_>>>()?;
    let storage = batch
        .as_record_batch()
        .column_by_name("_storage")
        .and_then(|column| column.as_any().downcast_ref::<StructArray>())
        .ok_or(Error::Invalid(
            "raw telemetry query has no storage envelope",
        ))?;
    for (row, entry) in entries.iter().enumerate() {
        check_envelope(codec, storage, row, entry)?;
    }
    Ok(entries)
}

pub fn encode_batch(codec: &TypedCodec, batch: &ArrowBatch) -> Result<Vec<Vec<u8>>> {
    let payload = batch
        .as_record_batch()
        .column_by_name("payload")
        .ok_or(Error::Invalid("query has no telemetry payload column"))?
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or(Error::Invalid(
            "query telemetry payload is not a typed struct",
        ))?;
    Ok(harmos::storage::arrow::encode_payloads(
        codec.graph(),
        payload,
        PayloadEncoding::PostcardV1,
        NonZeroUsize::MAX,
    )?)
}

pub fn decode_stream_values(
    stream: &str,
    validator: &Validator,
    bytes: &[u8],
) -> Result<BTreeMap<String, Value>> {
    let mut values = decode_values(validator, bytes)?;
    if let Ok(host) = dokime_contract::telemetry::HostTelemetryStream::try_from(stream)
        && host.as_str() == stream
    {
        dokime_contract::telemetry::host::contracts::project_host_values(host, &mut values)
            .map_err(|_| Error::Invalid("host telemetry values are invalid"))?;
    }
    Ok(values)
}

pub fn decode_values(validator: &Validator, bytes: &[u8]) -> Result<BTreeMap<String, Value>> {
    projected_values(validator, bytes)
        .map_err(|_| Error::Invalid("typed telemetry payload projection failed"))
}

/* === Private Functions === */
fn envelope_text<'a>(storage: &'a StructArray, field: &str, row: usize) -> Result<Option<&'a str>> {
    let array = storage
        .column_by_name(field)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .ok_or(Error::Invalid(
            "telemetry storage envelope string field is missing",
        ))?;
    Ok((!array.is_null(row)).then(|| array.value(row)))
}
fn check_envelope(
    codec: &TypedCodec,
    storage: &StructArray,
    row: usize,
    entry: &TelemetryEntry,
) -> Result<()> {
    if storage.is_null(row) {
        return Err(Error::Invalid("telemetry storage envelope is null"));
    }
    let sequence = storage
        .column_by_name("sequence")
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        .ok_or(Error::Invalid("telemetry storage sequence is missing"))?;
    let version = storage
        .column_by_name("schema_version")
        .and_then(|column| column.as_any().downcast_ref::<UInt32Array>())
        .ok_or(Error::Invalid(
            "telemetry storage schema version is missing",
        ))?;
    let time = storage
        .column_by_name("source_time")
        .and_then(|column| column.as_any().downcast_ref::<TimestampNanosecondArray>())
        .ok_or(Error::Invalid("telemetry storage source time is missing"))?;
    let sample = &entry.sample;
    let scope = sample
        .scope
        .as_ref()
        .and_then(|scope| scope.test_execution_id.as_ref())
        .map(|id| id.as_str());
    let timestamp = sample.timestamp.to_datetime()?.unix_timestamp_nanos();
    if envelope_text(storage, "stream", row)? != Some(sample.stream.as_str())
        || envelope_text(storage, "item", row)? != Some(sample.sample_id.as_str())
        || envelope_text(storage, "source", row)? != Some(entry.epoch.as_str())
        || envelope_text(storage, "scope", row)? != scope
        || envelope_text(storage, "schema_id", row)? != Some(codec.schema().id.as_str())
        || sequence.is_null(row)
        || sequence.value(row) != entry.ordinal
        || version.is_null(row)
        || version.value(row) != codec.schema().version.get()
        || time.is_null(row)
        || i128::from(time.value(row)) != timestamp
    {
        return Err(Error::Invalid(
            "typed telemetry row differs from storage envelope",
        ));
    }
    Ok(())
}

fn projection_memory(validator: &Validator, bytes: &[u8]) -> Result<usize> {
    let mut memory = std::mem::size_of::<BTreeMap<String, Value>>();
    validator
        .visit(bytes, |path, event| {
            let owned = match event {
                Event::Scalar(Scalar::String(value)) => value.len(),
                Event::Scalar(Scalar::Bytes(value)) => value
                    .len()
                    .checked_mul(std::mem::size_of::<Value>() * 2)
                    .ok_or(Error::Invalid("telemetry byte projection memory overflows"))?,
                _ => 0,
            };
            let field = match path.last() {
                Some(PathElement::Field { name, .. }) => name.len(),
                _ => 0,
            };
            memory = memory
                .checked_add(owned)
                .and_then(|memory| memory.checked_add(field))
                .and_then(|memory| memory.checked_add(8 * std::mem::size_of::<Value>() + 256))
                .ok_or(Error::Invalid("telemetry projection memory overflows"))?;
            Ok::<(), Error>(())
        })
        .map_err(|_| Error::Invalid("telemetry payload is invalid or exceeds projection memory"))?;
    Ok(memory)
}
fn projected_values(
    validator: &Validator,
    bytes: &[u8],
) -> std::result::Result<BTreeMap<String, Value>, String> {
    let mut containers = Vec::new();
    let mut root = None;
    validator
        .visit(bytes, |path, event| {
            match event {
                Event::BeginMessage => containers.push(Value::Object(BTreeMap::new())),
                Event::BeginRepeated { .. } => containers.push(Value::Array(Vec::new())),
                Event::EndMessage | Event::EndRepeated => {
                    let value = containers.pop().ok_or("missing telemetry container")?;
                    if containers.is_empty() {
                        root = Some(value);
                    } else {
                        attach(&mut containers, path, value)?;
                    }
                }
                Event::Optional { present: false } => {
                    attach(&mut containers, path, Value::Null)?;
                }
                Event::Scalar(scalar) => attach(&mut containers, path, value(scalar))?,
                Event::Optional { present: true } | Event::Variant { .. } => {}
            }
            Ok::<(), String>(())
        })
        .map_err(|error| error.to_string())?;
    match root {
        Some(Value::Object(values)) if containers.is_empty() => Ok(values),
        _ => Err("telemetry root is not a complete message".into()),
    }
}

fn attach(
    containers: &mut [Value],
    path: &[PathElement<'_>],
    value: Value,
) -> std::result::Result<(), String> {
    match (containers.last_mut(), path.last()) {
        (Some(Value::Object(fields)), Some(PathElement::Field { name, .. })) => {
            fields.insert((*name).to_owned(), value);
        }
        (Some(Value::Array(items)), Some(PathElement::Index(index))) if *index == items.len() => {
            items.push(value);
        }
        _ => return Err("telemetry value does not match its container".into()),
    }
    Ok(())
}

fn value(value: Scalar<'_>) -> Value {
    match value {
        Scalar::Bool(value) => Value::Bool(value),
        Scalar::I32(value) => Value::Integer(i64::from(value)),
        Scalar::I64(value) => Value::Integer(value),
        Scalar::U32(value) => Value::Unsigned(u64::from(value)),
        Scalar::U64(value) => Value::Unsigned(value),
        Scalar::F32(value) => Value::Number(f64::from(value)),
        Scalar::F64(value) => Value::Number(value),
        Scalar::String(value) => Value::Text(value.to_owned()),
        Scalar::Bytes(value) => Value::Array(
            value
                .iter()
                .map(|byte| Value::Integer(i64::from(*byte)))
                .collect(),
        ),
    }
}

/* === Tests === */
