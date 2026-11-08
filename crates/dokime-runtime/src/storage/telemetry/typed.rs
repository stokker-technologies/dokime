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
use std::{
    borrow::Cow,
    collections::BTreeMap,
    sync::{Arc, Mutex, PoisonError},
};

// Third Party Imports
use arrow_array::{Array, StringArray, UInt32Array};
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
use crate::storage::Error;
use crate::storage::Result;
use crate::storage::records::ScopeRow;
use crate::storage::values;
use crate::telemetry::TelemetryContract;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct TypedCodec {
    schema: SchemaKey,
    graph: Arc<Schema>,
    validator: Validator,
}

// Composed Codecs Retained By Stream And Immutable Schema Revision.
#[derive(Default)]
pub struct CodecCache {
    entries: Mutex<BTreeMap<(Identifier, u32), CachedCodec>>,
}

struct CachedCodec {
    payload: Schema,
    codec: Arc<TypedCodec>,
    registered: bool,
}

pub(crate) struct Admitted {
    pub entry: TelemetryEntry,
    pub metadata: String,
    pub strings: Vec<(Vec<String>, String)>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    signals: Vec<Signal>,
    scope: Option<ScopeRow>,
    captured_scopes: Vec<ScopeRow>,
    summary: Vec<values::Cell>,
    metadata: Vec<values::Cell>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Signal {
    name: String,
    numeric_bits: Option<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TypedCodec {
    pub fn new(stream: &Identifier, version: u32, payload: &Schema, settings: &harmos::config::SchemaSettings) -> Result<Self> {
        let version = std::num::NonZeroU32::new(version).ok_or(Error::Invalid("typed telemetry schema version must be positive"))?;
        let validation = Validator::from_schema(payload, settings);
        let invalid_graph = Error::Invalid("typed telemetry graph is invalid or exceeds schema limits");
        let validator = validation.map_err(|_| invalid_graph)?;
        let declaration = Schema::from_declaration(Metadata::DEFINITION, settings);
        let invalid_metadata = Error::Invalid("telemetry metadata graph exceeds schema limits");
        let metadata = declaration.map_err(|_| invalid_metadata)?;

        // Compose Metadata and Payload Definitions
        let payload_offset =
            metadata.definitions.len().checked_add(1).ok_or(Error::Invalid("telemetry graph definition count overflows"))?;
        let mut definitions = vec![vec![
            Field {
                name: Cow::Borrowed("metadata"),
                tag: 1,
                kind: Kind::Message { definition: metadata.root + 1 },
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
        let mut representations = BTreeMap::new();
        for (graph, offset) in [(&metadata, 1), (payload, payload_offset)] {
            representations.extend(graph.representations.iter().map(|(index, value)| (index + offset, value.clone())));
            append_definitions(&mut definitions, graph, offset)?;
        }

        // Validate and Name the Composed Graph
        let graph = Schema { root: 0, definitions, representations };
        let prepared = graph.prepare(settings);
        let invalid_composition = Error::Invalid("composed telemetry graph exceeds schema limits");
        prepared.map_err(|_| invalid_composition)?;
        let identifier = format!("dokime.telemetry.{}", stream.as_str());
        let schema = SchemaKey {
            id: Identifier::new(identifier)?,
            version,
            payload_encoding: PayloadEncoding::PostcardV1,
        };
        Ok(Self {
            schema,
            graph: Arc::new(graph),
            validator,
        })
    }

    pub(crate) fn from_stored(schema: SchemaKey, graph: Arc<Schema>, settings: &harmos::config::SchemaSettings) -> Result<Self> {
        let definition = values_definition(&graph)?;
        let validator = Validator::from_schema(&graph, settings)
            .and_then(|validator| validator.at(definition))
            .map_err(|_| Error::Invalid("stored telemetry schema is invalid"))?;
        Ok(Self { schema, graph, validator })
    }

    pub(crate) fn encode_arrow(&self, entry: &TelemetryEntry, original_metadata: &str, original_strings: &str) -> Result<Vec<u8>> {
        let root = values_definition(&self.graph)?;
        let mut values = entry.sample.values.clone();
        let strings: Vec<(Vec<String>, String)> = serde_json::from_str(original_strings)?;
        super::encode::restore_strings(&mut values, &strings)?;
        let payload = super::encode::payload(&self.graph, root, &values)?;
        let metadata: Metadata = serde_json::from_str(original_metadata)?;
        let length = metadata.encoded_size().checked_add(payload.len()).ok_or(Error::Invalid("telemetry frame length overflows"))?;
        encode_metadata(metadata, &payload, length)
    }

    pub fn schema(&self) -> &SchemaKey {
        &self.schema
    }
    pub fn graph(&self) -> &Arc<Schema> {
        &self.graph
    }

    // Staged Rows Carry Payloads Their Admission Already Checked Or Decoded With This Codec.
    // Encodes A Staged Row And, When Its Values Are This Codec's Projection Of Its Payload, Also Returns The Admission The
    // Storage Writer Would Decode From Those Bytes, So The Writer Need Not Read The Payload Again.
    pub(crate) fn encode_retained(
        &self, epoch: &SessionId, ordinal: u64, sample: &crate::telemetry::TelemetrySample, scope: Option<&RecordScope>,
        projected: &[Signal], decoded: bool,
    ) -> Result<(Vec<u8>, Option<Admitted>)> {
        let metadata = self.metadata(epoch, ordinal, sample, scope, projected.to_vec())?;
        let payload = sample.storage_payload.as_deref().ok_or(Error::Invalid("typed telemetry sample has no original row payload"))?;
        let length = metadata.encoded_size().checked_add(payload.len()).ok_or(Error::Invalid("typed telemetry frame length overflows"))?;
        let retained = if decoded && !self.payload_has_timestamp()? {
            // The Payload Projection Never Holds A Timestamp, So Admission Added Any Present One Beside It.
            let mut values = sample.values.clone();
            values.remove("timestamp");
            Some((metadata.clone(), values))
        } else {
            None
        };
        let bytes = encode_metadata(metadata, payload, length)?;
        let admitted = retained.map(|(metadata, values)| self.admit(metadata, values, payload)).transpose()?;
        Ok((bytes, admitted))
    }

    pub fn encode(&self, entry: &TelemetryEntry, maximum: usize) -> Result<Vec<u8>> {
        let payload =
            entry.sample.storage_payload.as_deref().ok_or(Error::Invalid("typed telemetry sample has no original row payload"))?;
        let projected = self.payload_signals(payload, &entry.sample.values)?;
        let metadata = self.metadata(&entry.epoch, entry.ordinal, &entry.sample, entry.sample.scope.as_ref(), projected)?;
        let length = metadata
            .encoded_size()
            .checked_add(payload.len())
            .filter(|length| *length <= maximum)
            .ok_or(Error::Invalid("typed telemetry frame exceeds byte limit"))?;
        encode_metadata(metadata, payload, length)
    }

    // The One Payload Pass For A Row Whose Values Arrived Beside Its Bytes.
    pub(crate) fn payload_signals(&self, payload: &[u8], values: &BTreeMap<String, Value>) -> Result<Vec<Signal>> {
        let mut decoded = self.decode_payload(payload)?;
        if let Some(timestamp) = values.get("timestamp") {
            restore_timestamp(&mut decoded, BTreeMap::from([("timestamp".to_owned(), timestamp.clone())]))?;
        }
        Ok(signals(&decoded))
    }

    // The One Payload Pass For A Row Whose Values Are Projected From Its Bytes.
    pub(crate) fn decode_payload(&self, payload: &[u8]) -> Result<BTreeMap<String, Value>> {
        payload_pass();
        self.project_payload(payload)
    }

    fn payload_has_timestamp(&self) -> Result<bool> {
        let root = values_definition(&self.graph)?;
        let fields = self.graph.definitions.get(root).ok_or(Error::Invalid("telemetry values schema is absent"))?;
        Ok(fields.iter().any(|field| field.name == "timestamp"))
    }

    fn project_payload(&self, payload: &[u8]) -> Result<BTreeMap<String, Value>> {
        projected_values(&self.validator, payload).map_err(|reason| {
            let stream = self.schema.id.as_str().strip_prefix("dokime.telemetry.").unwrap_or(self.schema.id.as_str()).to_owned();
            let column = values_definition(&self.graph)
                .ok()
                .and_then(|root| self.graph.definitions.get(root))
                .map(|fields| fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>().join(", "))
                .unwrap_or_else(|| "unknown".into());
            tracing::error!(%stream, %column, %reason, "typed telemetry payload projection failed");
            Error::TelemetryProjection { stream, column }
        })
    }

    fn metadata(
        &self, epoch: &SessionId, ordinal: u64, sample: &crate::telemetry::TelemetrySample, scope: Option<&RecordScope>,
        projected: Vec<Signal>,
    ) -> Result<Metadata> {
        if self.schema.id.as_str().strip_prefix("dokime.telemetry.") != Some(sample.stream.as_str()) {
            return Err(Error::Invalid("typed telemetry sample differs from registered stream schema"));
        }
        let epoch = sample::copy_text(epoch.as_str())?;
        let sample_id = sample::copy_text(sample.sample_id.as_str())?;
        let stream = sample::copy_text(sample.stream.as_str())?;
        let timestamp = sample::copy_text(sample.timestamp.as_str())?;

        // Encode the Timestamp Projection Before the Remaining Metadata
        let timestamp_values = sample.values.get("timestamp").map(|value| {
            let entry = ("timestamp".to_owned(), value.clone());
            BTreeMap::from([entry])
        });
        let timestamp_values = timestamp_values.unwrap_or_default();
        let timestamp_value = values::encode(&timestamp_values)?;
        drop(timestamp_values);
        let metadata = Metadata {
            epoch,
            ordinal,
            sample_id,
            stream,
            schema_version: self.schema.version.get(),
            sequence: sample.sequence,
            timestamp,
            timestamp_value,
            received_at: sample::copy_text(sample.received_at.as_str())?,
            source: sample::copy_text(sample.source.as_str())?,
            signals: projected,
            scope: scope.map(ScopeRow::from),
            captured_scopes: sample.captured_scopes.iter().map(ScopeRow::from).collect(),
            summary: values::encode(&sample.summary)?,
            metadata: values::encode(&sample.metadata)?,
        };
        sample.timestamp.to_datetime()?;
        sample.received_at.to_datetime()?;
        Ok(metadata)
    }

    pub(crate) fn decode_admission(&self, row: &harmos::storage::StoredRow) -> Result<Admitted> {
        self.admission(row, None)
    }

    // Checks A Row's Admission Against Its Storage Envelope, Decoding The Payload Only Without A Retained Admission.
    pub(crate) fn admission(&self, row: &harmos::storage::StoredRow, retained: Option<Admitted>) -> Result<Admitted> {
        // A Retained Admission Whose Payload Is Not This Row's Is Stale And Falls Back To Decoding.
        let retained = retained.filter(|admitted| {
            admitted.entry.sample.storage_payload.as_deref().is_some_and(|payload| row.record.payload.ends_with(payload))
        });
        let admitted = match retained {
            Some(admitted) => admitted,
            None => self.decode_arrow(&row.record.payload, usize::MAX)?,
        };
        let entry = &admitted.entry;
        let sample = &entry.sample;
        let scope = sample.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()).map(|id| id.as_str());
        let timestamp: std::time::SystemTime = sample.timestamp.to_datetime()?.into();
        if row.record.schema != self.schema
            || row.stream.as_str() != sample.stream.as_str()
            || row.record.item.as_str() != sample.sample_id.as_str()
            || row.record.source.as_ref().map(|id| id.as_str()) != Some(entry.epoch.as_str())
            || row.record.scope.as_ref().map(|id| id.as_str()) != scope
            || row.sequence != entry.ordinal
            || row.record.source_time != Some(timestamp)
        {
            return Err(Error::Invalid("typed telemetry row differs from storage envelope"));
        }
        Ok(admitted)
    }

    pub fn decode(&self, bytes: &[u8], maximum: usize) -> Result<TelemetryEntry> {
        Ok(self.decode_arrow(bytes, maximum)?.entry)
    }

    pub(crate) fn decode_arrow(&self, bytes: &[u8], maximum: usize) -> Result<Admitted> {
        if bytes.len() > maximum {
            return Err(Error::Invalid("typed telemetry frame exceeds byte limit"));
        }

        // Decode and Verify Canonical Metadata
        let mut reader = Reader::new(bytes);
        let metadata = <Metadata as PostcardValue>::read(&mut reader)?;
        let payload = reader.remaining();
        let metadata_length = bytes.len() - payload.len();
        let canonical_metadata = metadata.encode();
        if canonical_metadata != bytes[..metadata_length] {
            return Err(Error::Invalid("stored telemetry metadata is not canonical"));
        }
        drop(canonical_metadata);
        let is_uppercase = |byte: u8| byte.is_ascii_uppercase();
        let uppercase_epoch = metadata.epoch.bytes().any(is_uppercase);
        if uppercase_epoch {
            return Err(Error::Invalid("stored telemetry metadata is not canonical"));
        }
        let decoded = self.project_payload(payload)?;
        self.admit(metadata, decoded, payload)
    }

    // Restores A Row From Its Canonical Metadata And The Projection Of Its Payload.
    fn admit(&self, metadata: Metadata, mut decoded: BTreeMap<String, Value>, payload: &[u8]) -> Result<Admitted> {
        let original_metadata = serde_json::to_string(&metadata)?;
        let mut strings = Vec::new();
        for (name, value) in &decoded {
            collect_strings(value, &mut vec![name.clone()], &mut strings);
        }

        let timestamp = values::decode(&metadata.timestamp_value)?;
        if timestamp.len() > 1 || timestamp.keys().any(|key| key != "timestamp") {
            return Err(Error::Invalid("stored telemetry timestamp projection is invalid"));
        }
        restore_timestamp(&mut decoded, timestamp)?;
        if metadata.signals != signals(&decoded) {
            return Err(Error::Invalid("stored telemetry signal projection differs from its payload"));
        }
        if metadata.schema_version != self.schema.version.get()
            || self.schema.id.as_str() != format!("dokime.telemetry.{}", metadata.stream)
        {
            return Err(Error::Invalid("stored telemetry metadata differs from its schema"));
        }
        // Restore the Sample Under the Decoding Allowance
        let sample = Sample {
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
        };
        let mut sample = sample.into_sample()?;
        sample.captured_scopes = metadata.captured_scopes.into_iter().map(TryInto::try_into).collect::<Result<_>>()?;
        sample.values = decoded;
        sample.storage_payload = Some(payload.to_vec());
        Ok(Admitted {
            entry: TelemetryEntry {
                epoch: SessionId::from_uuid(metadata.epoch)?,
                ordinal: metadata.ordinal,
                sample,
            },
            metadata: original_metadata,
            strings,
        })
    }
}

impl std::ops::Deref for Admitted {
    type Target = TelemetryEntry;
    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl CodecCache {
    pub(crate) fn registered(&self, contract: &TelemetryContract) -> Result<Arc<TypedCodec>> {
        let entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = entries
            .iter()
            .find(|((stream, version), _)| stream == &contract.stream && *version == contract.schema_version)
            .map(|(_, entry)| entry)
            .ok_or(Error::Invalid("telemetry producer schema is not registered"))?;
        let same = contract.storage_schema.as_ref().map(|payload| same_graph(payload, &entry.payload)).transpose()?.unwrap_or(false);
        if !entry.registered || !same {
            return Err(Error::Invalid("telemetry producer schema differs from its registered graph"));
        }
        Ok(Arc::clone(&entry.codec))
    }

    pub fn codec(&self, contract: &TelemetryContract, settings: &harmos::config::SchemaSettings) -> Result<Arc<TypedCodec>> {
        let reuse = |entry: &mut CachedCodec| entry.codec.clone();
        self.with_entry(contract, settings, reuse)
    }

    // The Codec Whose Schema This Process Has Not Yet Saved, If Any.
    pub fn unregistered(&self, contract: &TelemetryContract, settings: &harmos::config::SchemaSettings) -> Result<Option<Arc<TypedCodec>>> {
        self.with_entry(contract, settings, |entry| {
            let codec = (!entry.registered).then_some(&entry.codec);
            codec.cloned()
        })
    }

    pub fn mark_registered(&self, contract: &TelemetryContract, saved: &Arc<TypedCodec>) {
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        let key = (contract.stream.clone(), contract.schema_version);
        if let Some(entry) = entries.get_mut(&key)
            && Arc::ptr_eq(&entry.codec, saved)
        {
            entry.registered = true;
        }
    }

    fn with_entry<T>(
        &self, contract: &TelemetryContract, settings: &harmos::config::SchemaSettings, use_entry: impl FnOnce(&mut CachedCodec) -> T,
    ) -> Result<T> {
        let payload = contract.storage_schema.as_ref().ok_or(Error::Invalid("captured telemetry has no typed storage schema"))?;
        let key = (contract.stream.clone(), contract.schema_version);
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(entry) = entries.get(&key)
            && !same_graph(&entry.payload, payload)?
        {
            return Err(Error::TelemetryIntegrity { stream: contract.stream.to_string() });
        }
        if !entries.contains_key(&key) {
            let payload = payload.clone();
            let codec = codec(contract, settings)?;
            let entry = CachedCodec {
                payload,
                codec: Arc::new(codec),
                registered: false,
            };
            entries.insert(key.clone(), entry);
        }
        let entry = entries.get_mut(&key).expect("codec entry inserted");
        Ok(use_entry(entry))
    }
}

/* === Public Functions === */

pub(crate) fn same_graph(left: &Schema, right: &Schema) -> Result<bool> {
    if left == right {
        return Ok(true);
    }
    let canonical = |graph: &Schema| {
        graph
            .root()
            .and_then(harmos::schema::identity::canonical)
            .map_err(|_| Error::Invalid("telemetry schema has no canonical definition"))
    };
    Ok(canonical(left)? == canonical(right)?)
}

pub fn codec(contract: &TelemetryContract, settings: &harmos::config::SchemaSettings) -> Result<TypedCodec> {
    let schema = contract.storage_schema.as_ref().ok_or(Error::Invalid("captured telemetry has no typed storage schema"))?;
    TypedCodec::new(&contract.stream, contract.schema_version, schema, settings)
}

pub fn stored_codecs_where(
    batch: ArrowBatch, settings: &harmos::config::SchemaSettings, selected: impl Fn(&str, u32) -> bool,
) -> Result<harmos::storage::ReadBatch<Vec<TypedCodec>>> {
    // Decode the Selected Graphs
    let decoded = batch.try_map(|batch| {
        let (ids, versions, definitions) = schema_columns(&batch)?;
        let mut schemas = Vec::new();
        let allocation = schemas.try_reserve_exact(batch.num_rows());
        let allocation_failed = Error::Invalid("telemetry schema allocation failed");
        allocation.map_err(|_| allocation_failed)?;
        let selected_row = |row: &usize| {
            let id = ids.value(*row);
            let version = versions.value(*row);
            selected(id, version)
        };
        for row in (0..batch.num_rows()).filter(selected_row) {
            schema_encoding(&batch, row)?;
            let schema = SchemaKey {
                id: harmos::Identifier::new(ids.value(row))?,
                version: std::num::NonZeroU32::new(versions.value(row)).ok_or(Error::Invalid("telemetry schema version is zero"))?,
                payload_encoding: PayloadEncoding::PostcardV1,
            };
            let definition = definitions.value(row);
            let graph = Schema::decode(definition)?;
            schemas.push((schema, graph));
        }
        Ok::<_, Error>(schemas)
    })?;
    // Compile the Values Roots
    decoded.try_map(|schemas| {
        let mut codecs = Vec::new();
        let allocation = codecs.try_reserve_exact(schemas.len());
        let allocation_failed = Error::Invalid("telemetry codec allocation failed");
        allocation.map_err(|_| allocation_failed)?;
        for (schema, graph) in schemas {
            let definition = values_definition(&graph)?;
            let at_values = |validator: Validator| validator.at(definition);
            let construction = Validator::from_schema(&graph, settings).and_then(at_values);
            let invalid_schema = Error::Invalid("stored telemetry schema is invalid");
            let validator = construction.map_err(|_| invalid_schema)?;
            let graph = Arc::new(graph);
            codecs.push(TypedCodec { schema, graph, validator });
        }
        Ok(codecs)
    })
}

pub(crate) fn selected_schema(id: &str, streams: &[dokime_domain::model::identity::IdentifierPattern]) -> bool {
    let Some(stream) = id.strip_prefix("dokime.telemetry.") else {
        return false;
    };
    if streams.is_empty() {
        return true;
    }
    let matches = |pattern: &dokime_domain::model::identity::IdentifierPattern| pattern.matches(stream);
    streams.iter().any(matches)
}

fn schema_encoding(batch: &arrow_array::RecordBatch, row: usize) -> Result<()> {
    for (field, expected) in [("payload_encoding", "postcard_v1"), ("schema_encoding", "harmos.graph.postcard.v1")] {
        check_schema_encoding(batch, row, field, expected)?;
    }
    Ok(())
}

fn schema_columns(batch: &arrow_array::RecordBatch) -> Result<(&StringArray, &UInt32Array, &arrow_array::BinaryArray)> {
    let ids =
        batch.column_by_name("schema_id").and_then(as_array::<StringArray>).ok_or(Error::Invalid("schema query has no identifiers"))?;
    let versions =
        batch.column_by_name("schema_version").and_then(as_array::<UInt32Array>).ok_or(Error::Invalid("schema query has no versions"))?;
    let definitions = batch
        .column_by_name("definition")
        .and_then(as_array::<arrow_array::BinaryArray>)
        .ok_or(Error::Invalid("schema query has no definitions"))?;
    Ok((ids, versions, definitions))
}

pub fn decode_values(validator: &Validator, bytes: &[u8]) -> Result<BTreeMap<String, Value>> {
    let projection = projected_values(validator, bytes);
    let invalid_projection = Error::Invalid("typed telemetry payload projection failed");
    projection.map_err(|_| invalid_projection)
}

pub(crate) fn signals(values: &BTreeMap<String, Value>) -> Vec<Signal> {
    values
        .iter()
        .filter(|(_, value)| !matches!(value, Value::Null))
        .map(|(name, value)| {
            let numeric = match value {
                Value::Integer(value) => Some(*value as f64),
                Value::Unsigned(value) => Some(*value as f64),
                Value::Number(value) => Some(*value),
                _ => None,
            };
            Signal {
                name: name.clone(),
                numeric_bits: numeric.filter(|value| value.is_finite()).map(f64::to_bits),
            }
        })
        .collect()
}

/* === Private Functions === */

fn collect_strings(value: &Value, path: &mut Vec<String>, strings: &mut Vec<(Vec<String>, String)>) {
    match value {
        Value::Text(text) => strings.push((path.clone(), text.clone())),
        Value::Object(fields) => {
            for (name, value) in fields {
                path.push(name.clone());
                collect_strings(value, path, strings);
                path.pop();
            }
        }
        Value::Array(items) => {
            for (index, value) in items.iter().enumerate() {
                path.push(index.to_string());
                collect_strings(value, path, strings);
                path.pop();
            }
        }
        _ => {}
    }
}

// Counts Admission Payload Passes On This Thread So Tests Can Prove Each Row Is Read Once.
#[cfg(test)]
thread_local! {
    pub(crate) static PAYLOAD_PASSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn payload_pass() {
    #[cfg(test)]
    PAYLOAD_PASSES.with(|passes| passes.set(passes.get() + 1));
}

fn append_definitions(target: &mut Vec<Vec<Field>>, graph: &Schema, offset: usize) -> Result<()> {
    for fields in &graph.definitions {
        let fields = shifted_fields(fields, offset)?;
        target.push(fields);
    }
    Ok(())
}

fn shifted_fields(fields: &[Field], offset: usize) -> Result<Vec<Field>> {
    let mut fields = fields.to_vec();
    let messages = fields.iter_mut().filter_map(|field| {
        let Kind::Message { definition } = &mut field.kind else {
            return None;
        };
        Some(definition)
    });
    for definition in messages {
        *definition = definition.checked_add(offset).ok_or(Error::Invalid("telemetry graph reference overflows"))?;
    }
    Ok(fields)
}

fn restore_timestamp(decoded: &mut BTreeMap<String, Value>, mut timestamp: BTreeMap<String, Value>) -> Result<()> {
    let Some(timestamp) = timestamp.remove("timestamp") else {
        return Ok(());
    };
    let differs = match decoded.get("timestamp") {
        Some(value) => !same_timestamp(value, &timestamp),
        None => false,
    };
    if differs {
        return Err(Error::Invalid("stored telemetry timestamp differs from its payload"));
    }
    decoded.insert("timestamp".to_owned(), timestamp);
    Ok(())
}

fn values_definition(graph: &Schema) -> Result<usize> {
    let root = graph.definitions.get(graph.root).ok_or(Error::Invalid("telemetry schema root is absent"))?;
    let values = root
        .iter()
        .find(|field| field.name == "values" && field.tag == 2)
        .ok_or(Error::Invalid("telemetry schema values root is absent"))?;
    let Kind::Message { definition } = values.kind else {
        return Err(Error::Invalid("telemetry values schema is not a message"));
    };
    Ok(definition)
}

fn as_array<T: Array + 'static>(column: &arrow_array::ArrayRef) -> Option<&T> {
    column.as_any().downcast_ref::<T>()
}

fn check_schema_encoding(batch: &arrow_array::RecordBatch, row: usize, field: &str, expected: &str) -> Result<()> {
    let values =
        batch.column_by_name(field).and_then(as_array::<StringArray>).ok_or(Error::Invalid("telemetry schema encoding is absent"))?;
    if values.is_null(row) || values.value(row) != expected {
        return Err(Error::Invalid("telemetry schema encoding is unsupported"));
    }
    Ok(())
}

fn encode_metadata(metadata: Metadata, payload: &[u8], length: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let allocation = bytes.try_reserve_exact(length);
    let allocation_failed = Error::Invalid("typed telemetry allocation failed");
    allocation.map_err(|_| allocation_failed)?;
    metadata.encode_into(&mut bytes);
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

// A row may carry its timestamp as RFC 3339 text that values hold as a datetime.
fn same_timestamp(payload: &Value, stored: &Value) -> bool {
    match (payload, stored) {
        (Value::Text(text), Value::Datetime(instant)) | (Value::Datetime(instant), Value::Text(text)) => {
            let timestamp = dokime_domain::model::time::Timestamp::new(text.as_str());
            timestamp.to_datetime().is_ok_and(|parsed| parsed == *instant)
        }
        _ => payload == stored,
    }
}

fn projected_values(validator: &Validator, bytes: &[u8]) -> std::result::Result<BTreeMap<String, Value>, String> {
    let mut containers = Vec::new();
    let mut root = None;
    let visit = validator.visit(bytes, |path, event| {
        match event {
            Event::BeginMessage => {
                let fields = BTreeMap::new();
                containers.push(Value::Object(fields));
            }
            Event::BeginRepeated { .. } => {
                let items = Vec::new();
                containers.push(Value::Array(items));
            }
            Event::EndMessage | Event::EndRepeated => {
                let value = containers.pop().ok_or("missing telemetry container")?;
                if containers.is_empty() {
                    root = Some(value);
                    return Ok(());
                }
                attach(&mut containers, path, value)?;
            }
            Event::Optional { present: false } => {
                attach(&mut containers, path, Value::Null)?;
            }
            Event::Scalar(scalar) => {
                let value = value(scalar);
                attach(&mut containers, path, value)?;
            }
            Event::Optional { present: true } | Event::Variant { .. } => {}
        }
        Ok::<(), String>(())
    });
    if let Err(error) = visit {
        return Err(error.to_string());
    }
    match root {
        Some(Value::Object(values)) if containers.is_empty() => Ok(values),
        _ => Err("telemetry root is not a complete message".into()),
    }
}

fn attach(containers: &mut [Value], path: &[PathElement<'_>], value: Value) -> std::result::Result<(), String> {
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
        Scalar::Bytes(value) => {
            let integer = |byte: &u8| {
                let integer = i64::from(*byte);
                Value::Integer(integer)
            };
            let values = value.iter().map(integer).collect();
            Value::Array(values)
        }
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[harmos::message]
    #[derive(Clone, PartialEq)]
    struct Reading {
        value: f64,
    }

    #[harmos::message]
    #[derive(Clone, PartialEq)]
    struct Renamed {
        label: String,
    }

    #[test]
    fn cache_reuses_codecs_until_the_graph_changes() {
        let cache = CodecCache::default();
        let settings = harmos::config::SchemaSettings::default();
        let contract = TelemetryContract::declared("cache.fixture", "Cache").typed::<Reading>(1);

        let first = cache.codec(&contract, &settings).unwrap();
        let reused = cache.codec(&contract, &settings).unwrap();
        assert!(Arc::ptr_eq(&first, &reused));
        assert!(cache.unregistered(&contract, &settings).unwrap().is_some());
        cache.mark_registered(&contract, &first);
        assert!(cache.unregistered(&contract, &settings).unwrap().is_none());

        // A Changed Row Receives Its Own Revision And Retains The Previous Codec.
        let redefined = TelemetryContract::declared("cache.fixture", "Cache").typed::<Renamed>(1);
        let rebuilt = cache.codec(&redefined, &settings).unwrap();
        assert!(!Arc::ptr_eq(&first, &rebuilt));
        assert_ne!(first.schema(), rebuilt.schema());
        assert!(Arc::ptr_eq(&cache.registered(&contract).unwrap(), &first));
        // An Old Registration Cannot Mark The New Revision Saved.
        cache.mark_registered(&contract, &first);
        assert!(cache.unregistered(&redefined, &settings).unwrap().is_some());
    }

    #[test]
    fn stream_revision_collision_cannot_replace_a_registered_codec() {
        let cache = CodecCache::default();
        let settings = Default::default();
        let old = TelemetryContract::declared("collision.reading", "Reading").typed::<Reading>(1);
        let mut changed = TelemetryContract::declared("collision.reading", "Reading").typed::<Renamed>(1);
        let codec = cache.codec(&old, &settings).unwrap();
        cache.mark_registered(&old, &codec);
        changed.schema_version = old.schema_version;
        assert!(matches!(cache.codec(&changed, &settings), Err(Error::TelemetryIntegrity { stream }) if stream == old.stream.as_str()));
        assert!(Arc::ptr_eq(&cache.registered(&old).unwrap(), &codec));
    }

    #[test]
    fn stream_revision_projection_errors_identify_the_stream_and_columns() {
        let contract = TelemetryContract::declared("projection.reading", "Reading").typed::<Reading>(1);
        let codec = codec(&contract, &Default::default()).unwrap();
        assert!(matches!(codec.decode_payload(&[]), Err(Error::TelemetryProjection { stream, column })
            if stream == contract.stream.as_str() && column == "value"));
    }

    #[test]
    fn numeric_projection_is_verified_against_payload() {
        let contract = TelemetryContract::declared("fixture.numeric", "Numeric").typed::<Reading>(1);
        let codec = codec(&contract, &harmos::config::SchemaSettings::default()).unwrap();
        let payload = Reading { value: 7.0 }.encode();
        let values = BTreeMap::from([("value".to_owned(), Value::Number(7.0))]);
        assert!(codec.payload_signals(&payload, &values).unwrap() == signals(&values));
        assert!(codec.payload_signals(&payload, &BTreeMap::from([("value".to_owned(), Value::Number(8.0))])).unwrap() == signals(&values));
        let mut sample = crate::telemetry::TelemetrySample::new(
            crate::model::identity::new_id(),
            contract.stream,
            1,
            crate::model::time::now_utc(),
            "fixture.numeric",
        )
        .typed(&Reading { value: 7.0 });
        sample.values = values.clone();
        let entry = TelemetryEntry {
            epoch: SessionId::from_seed("numeric.epoch"),
            ordinal: 1,
            sample,
        };
        let bytes = codec.encode(&entry, usize::MAX).unwrap();
        assert_eq!(codec.decode(&bytes, usize::MAX).unwrap().sample.values, values);
        let mut reader = Reader::new(&bytes);
        let mut metadata = <Metadata as PostcardValue>::read(&mut reader).unwrap();
        metadata.signals[0].numeric_bits = Some(8.0_f64.to_bits());
        let mut corrupted = metadata.encode();
        corrupted.extend_from_slice(reader.remaining());
        assert!(codec.decode(&corrupted, usize::MAX).is_err());
    }

    #[harmos::message]
    #[derive(Clone, PartialEq)]
    struct Labelled {
        label: String,
        value: f64,
    }

    #[harmos::message]
    #[derive(Clone, PartialEq)]
    struct Stamped {
        timestamp: String,
        value: f64,
    }

    #[test]
    fn retained_admissions_equal_the_rows_storage_decodes() {
        let settings = harmos::config::SchemaSettings::default();
        let contract = TelemetryContract::declared("fixture.retained", "Retained").typed::<Labelled>(1);
        let codec = codec(&contract, &settings).unwrap();
        let epoch = SessionId::from_seed("retained.epoch");
        let scope = RecordScope {
            test_execution_id: Some(dokime_domain::model::identity::TestExecutionId::from_seed("retained.execution")),
            ..Default::default()
        };
        for (ordinal, label) in ["first", "second reading", ""].into_iter().enumerate() {
            let at = crate::model::time::now_utc();
            let mut sample = crate::telemetry::TelemetrySample::new(
                crate::model::identity::new_id(),
                contract.stream.clone(),
                ordinal as u64,
                at.clone(),
                "fixture.retained",
            )
            .typed(&Labelled { label: label.to_owned(), value: ordinal as f64 + 0.5 });
            // The Host's One Payload Pass: Projected Values Plus The Admission Timestamp.
            sample.values = codec.decode_payload(sample.storage_payload.as_deref().unwrap()).unwrap();
            sample.values.insert("timestamp".into(), Value::from(at));
            sample.scope = Some(scope.clone());
            let projected = signals(&sample.values);
            let (bytes, retained) = codec.encode_retained(&epoch, ordinal as u64, &sample, Some(&scope), &projected, true).unwrap();
            let retained = retained.expect("projected values are retained");
            let decoded = codec.decode_arrow(&bytes, usize::MAX).unwrap();
            assert_eq!(retained.entry, decoded.entry);
            assert_eq!(retained.metadata, decoded.metadata);
            assert_eq!(retained.strings, decoded.strings);
            let (unchecked, none) = codec.encode_retained(&epoch, ordinal as u64, &sample, Some(&scope), &projected, false).unwrap();
            assert_eq!((unchecked, none.is_none()), (bytes, true));
        }

        // A Payload That Carries Its Own Timestamp Cannot Be Told Apart From The Admission's, So It Is Decoded.
        let stamped = TelemetryContract::declared("fixture.stamped", "Stamped").typed::<Stamped>(1);
        let codec = super::codec(&stamped, &settings).unwrap();
        let at = crate::model::time::now_utc();
        let mut sample =
            crate::telemetry::TelemetrySample::new(crate::model::identity::new_id(), stamped.stream.clone(), 1, at.clone(), "fixture.stamped")
                .typed(&Stamped { timestamp: at.as_str().to_owned(), value: 1.0 });
        sample.values = codec.decode_payload(sample.storage_payload.as_deref().unwrap()).unwrap();
        let projected = signals(&sample.values);
        assert!(codec.encode_retained(&epoch, 1, &sample, None, &projected, true).unwrap().1.is_none());
    }
}
