/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Lifecycle Proof Through the Product Backend

/* === Modules === */

mod transitions;
mod sealing;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{convert::Infallible, error::Error, fs, num::NonZeroUsize, path::Path, result, sync::Arc, time::Duration};

// Third Party Imports
use harmos_operations::storage::{AppendSamples, AppendSamplesRequest, Capabilities, CapabilitiesRequest, CompoundOutcome, DataBatch, GetSchema, GetSchemaRequest, Mutate, MutateRequest, MutationId, MutationItem};
use harmos_protocol::model::ReadCut;
use harmos_query::{source, arrow::ArrowBatch};
use arrow_array::{ArrayRef, RecordBatch, StringArray, StructArray, UInt32Array, UInt64Array, Int64Array};
use harmos_runtime::{Application, Identified, Identifier, Lifecycle, Message, Runtime, Setup, Startup};
use harmos_runtime::operations::Lane;
use harmos_runtime::state::Never;
use harmos_runtime::storage::{RecordInput, StorageFilesystemSettings, StorageManager, StoragePolicy, StorageRequestSettings, StorageResponseSettings, StorageSettings};

// Local Imports
use super::{io, register};
use dokime_protocol::model::{ExecutionProgressValue, Metadata};
use dokime_storage::{Backend, BackendLease, Store};
use dokime_storage::persisted::{self, Scope, bindings::{self, Binding, Protobuf}, records, shapes};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Proof<T = ()> = result::Result<T, Box<dyn Error>>;

/* === Structs === */

struct Model;

#[derive(harmos_runtime::Message, Clone, Debug, PartialEq)]
struct Reading {
    completed: u64,
    total: u64,
    label: String,
}

struct Fixture {
    runtime: Runtime<Model>,
    _lease: BackendLease,
    _store: Store,
    _root: tempfile::TempDir,
    bytes: NonZeroUsize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Lifecycle for Model {
    type Config = ();
    type Error = Infallible;
    type Context = Startup<Self>;
    async fn prepare(_: (), _: &mut Setup<Self>) -> result::Result<Self, Infallible> {
        Ok(Self)
    }
}
impl Application for Model {
    type State = ();
    type Transaction = Never;
    type Record = ();
}

impl Identified for Reading {
    const ID: &'static str = "thermal.progress";
}

impl Fixture {
    async fn open() -> Proof<Self> {
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        fs::create_dir_all(&target)?;
        let root = tempfile::tempdir_in(target)?;
        Self::at(root).await
    }

    async fn at(root: tempfile::TempDir) -> Proof<Self> {
        let store = Store::open(root.path())?;
        let catalog = persisted::catalog();
        let (backends, lease) = Backend::open_with_execution(&store, catalog.clone(), Default::default())?;
        let rows = NonZeroUsize::new(8).unwrap();
        let capacity = NonZeroUsize::new(128).unwrap();
        let bytes = NonZeroUsize::new(4 * 1024 * 1024).unwrap();
        let storage = StorageManager::with_catalog(catalog, backends, StorageSettings {
            transfer: Default::default(), policy: StoragePolicy::default(),
            request: StorageRequestSettings {
                max_file_productions: capacity, max_mutation_receipts: capacity, max_pending_queries: capacity,
                max_active_queries: capacity, max_pending_write_batches: capacity, max_active_file_operations: capacity,
                max_pending_write_bytes: bytes,
            },
            response: StorageResponseSettings { batch_rows: rows, batch_bytes: bytes },
            filesystem: StorageFilesystemSettings { poll_interval: Duration::from_millis(100), max_entries: capacity, event_capacity: capacity },
        })?;
        let runtime = Runtime::<Model>::start_with((), (), |setup| {
            setup.settings().storage(&storage);
            setup.resource::<StorageManager>(storage.clone());
        }).await?;
        runtime.operations.register_catalog(harmos_operations::storage::storage_catalog(), Lane::default()).await?;
        Ok(Self { runtime, _lease: lease, _store: store, _root: root, bytes })
    }

    async fn reopen(self) -> Proof<Self> {
        self.runtime.stop().await?;
        drop(self.runtime);
        self._lease.close()?;
        drop(self._lease);
        drop(self._store);
        Self::at(self._root).await
    }

    async fn mutation(&self, key: &str) -> Proof<MutationId> {
        let capabilities = io::call::<Capabilities>(&self.runtime.operations, CapabilitiesRequest {}).await?;
        Ok(MutationId { window: capabilities.mutations.ok_or("Read the Native Mutation Window")?.id, key: Identifier::new(key)? })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn registered_shapes_round_trip_at_native_cuts() -> Proof {
    // Register Frozen Graphs Through Library Operations Against a Fresh Product Root.
    let fixture = Fixture::open().await?;
    let operations = &fixture.runtime.operations;
    let mutation = fixture.mutation("schemas").await?;
    register(operations, &mutation).await?;
    register(operations, &mutation).await?;
    for definition in persisted::catalog().definitions() {
        let schema = definition.schema()?;
        let response = io::call::<GetSchema>(operations, GetSchemaRequest { key: Some(schema), at: None }).await?;
        assert!(response.schema.and_then(|schema| schema.graph).is_some());
    }
    for (schema, _) in [bindings::schema::<records::CycleEvent>()?, bindings::schema::<records::ExecutionEvent>()?,
        bindings::schema::<records::EvidenceEvent>()?, bindings::schema::<records::HealthEvent>()?,
        bindings::schema::<records::PromptEvent>()?, bindings::schema::<records::AnswerEvent>()?,
        bindings::schema::<records::DocumentPart>()?, bindings::schema::<records::ReportPublished>()?] {
        let response = io::call::<GetSchema>(operations, GetSchemaRequest { key: Some(schema), at: None }).await?;
        assert!(response.schema.and_then(|schema| schema.graph).is_some());
    }

    // Persist an Exact-Scope Value and Decode Its Frozen Protobuf Binding.
    let value = Protobuf::product("dokime.model.v1.Metadata", &Metadata::default(), fixture.bytes)?;
    let saved = shapes::Root { key: "backend".into(), scope: Scope::Root("backend").identifier()?, value };
    let mutation = fixture.mutation("root").await?;
    let response = io::call::<Mutate>(operations, MutateRequest {
        mutation: Some(mutation), items: vec![MutationItem::Create(io::encode(&saved, fixture.bytes)?)],
    }).await?;
    let at = response.receipt.and_then(|receipt| receipt.durable_at).ok_or("Return the Durable Receipt")?;
    let at = ReadCut { positions: vec![at], revisions: Vec::new() };
    let read = super::read::<shapes::Root>(operations, &saved.key, Some(at), fixture.bytes).await?.ok_or("Read the Native Root")?;
    assert_eq!(read.value, saved);
    read.value.value.decode(fixture.bytes)?;

    // Every Object Codec Participates in One Native Compound Mutation.
    let value = saved.value.clone();
    let scope = Scope::Execution("execution").identifier()?;
    let cycle_scope = Scope::Cycle("cycle").identifier()?;
    let execution = shapes::Execution {
        key: "execution".into(), scope: scope.clone(), case: "case".into(), cycle: "cycle".into(), origin: value.clone(),
        catalog: Vec::new(), state: shapes::State::Open, outcome: None, termination: 0, seal: None, previews: Vec::new(),
    };
    let values = vec![
        io::encode(&shapes::Cycle { key: "cycle".into(), scope: cycle_scope, metadata: value.clone(), sealed: false, outcomes: Default::default() }, fixture.bytes)?,
        io::encode(&execution, fixture.bytes)?,
        io::encode(&shapes::Parameters { key: "execution".into(), scope: scope.clone(), value: value.clone() }, fixture.bytes)?,
        io::encode(&shapes::Steps { key: "execution".into(), scope: scope.clone(), value: value.clone() }, fixture.bytes)?,
        io::encode(&shapes::Progress { key: "execution".into(), scope: scope.clone(), value: value.clone() }, fixture.bytes)?,
        io::encode(&shapes::Stream { key: "execution/stream".into(), scope: scope.clone(), declaration: "stream".into(), schema: "stream".into(), version: 1, message: value.message.clone(), descriptors: value.descriptors.clone(), closed: false }, fixture.bytes)?,
        io::encode(&shapes::Prompt { key: "execution/prompt".into(), scope: scope.clone(), value: value.clone() }, fixture.bytes)?,
        io::encode(&shapes::Document { key: "execution/document".into(), scope: scope.clone(), value: value.clone() }, fixture.bytes)?,
        io::encode(&shapes::PreparedReport { key: "request".into(), scope: scope.clone(), value: value.clone(), evidence_cut: Vec::new() }, fixture.bytes)?,
        io::encode(&shapes::Report { key: "execution/report".into(), scope: scope.clone(), value: value.clone(), evidence_cut: Vec::new(), file: value.clone() }, fixture.bytes)?,
        io::encode(&shapes::Artifact { key: "artifact".into(), scope: scope.clone(), value: value.clone(), file: value.clone() }, fixture.bytes)?,
        io::encode(&shapes::ExecutionFile { key: "execution/file".into(), scope: scope.clone(), logical_path: "file".into(), file: value.clone() }, fixture.bytes)?,
        io::encode(&shapes::Preview { key: "execution/preview".into(), scope, source_cut: Vec::new(), kind: "preview".into(), selection: Vec::new(), payload: Vec::new() }, fixture.bytes)?,
    ];
    for value in &values {
        persisted::catalog().validate_value(value, fixture.bytes)?;
    }
    let response = io::call::<Mutate>(operations, MutateRequest {
        mutation: Some(fixture.mutation("objects").await?), items: values.into_iter().map(MutationItem::Create).collect(),
    }).await?;
    assert!(matches!(response.outcome, CompoundOutcome::Applied { .. }));
    let read = super::read::<shapes::Execution>(operations, "execution", None, fixture.bytes).await?.ok_or("Read the Native Execution")?;
    assert_eq!(read.value, execution);

    // Register a Per-Stream Graph and a Distinct Typed Record Graph.
    let (key, graph) = bindings::schema::<Reading>()?;
    let frozen = Protobuf::product("dokime.model.v1.ExecutionProgressValue", &ExecutionProgressValue::default(), fixture.bytes)?;
    let binding = Binding { key, graph, message: frozen.message, descriptors: frozen.descriptors };
    super::register_binding(operations, &fixture.mutation("stream-schema").await?, &binding, fixture.bytes).await?;
    let record = binding.record()?;
    super::register_binding(operations, &fixture.mutation("record-schema").await?, &record, fixture.bytes).await?;
    assert_ne!(record.key, binding.key);
    let event = records::ExecutionEvent { state: shapes::State::Open, kind: "created".into(), step: String::new(), operation: String::new(), attempt: 0, receipt_ns: 0 };
    assert_eq!(records::ExecutionEvent::decode(&event.encode())?, event);
    fixture.runtime.stop().await?;
    Ok(())
}

#[tokio::test]
async fn typed_records_and_streams_use_native_column_codecs() -> Proof {
    // Freeze One Plugin Value Schema for Both Native Record Context and Samples.
    let fixture = Fixture::open().await?;
    let operations = &fixture.runtime.operations;
    let (key, graph) = bindings::schema::<Reading>()?;
    let frozen = Protobuf::product("dokime.model.v1.ExecutionProgressValue", &ExecutionProgressValue::default(), fixture.bytes)?;
    let binding = Binding { key, graph, message: frozen.message, descriptors: frozen.descriptors };
    let record = binding.record()?;
    super::register_binding(operations, &fixture.mutation("sample-schema").await?, &binding, fixture.bytes).await?;
    super::register_binding(operations, &fixture.mutation("record-schema").await?, &record, fixture.bytes).await?;

    // The Library Encodes the Typed Columns, Including the Product Record Context.
    let fields = source::payload_fields(&binding.graph)?;
    let columns = fields.iter().map(|field| -> ArrayRef {
        match field.name().as_str() {
            "completed" => Arc::new(UInt64Array::from(vec![7])),
            "total" => Arc::new(UInt64Array::from(vec![11])),
            "label" => Arc::new(StringArray::from(vec!["reading"])),
            _ => unreachable!(),
        }
    }).collect();
    let sample = StructArray::new(fields, columns, None);
    let payloads = binding.encode_columns(&sample, fixture.bytes)?;
    assert_eq!(Reading::decode(&payloads[0])?, Reading { completed: 7, total: 11, label: "reading".into() });
    let fields = source::payload_fields(&record.graph)?;
    let columns = fields.iter().map(|field| -> ArrayRef {
        match field.name().as_str() {
            "step" | "operation" => Arc::new(StringArray::from(vec![""])),
            "attempt" => Arc::new(UInt32Array::from(vec![0])),
            "receipt_ns" => Arc::new(Int64Array::from(vec![0])),
            "value" => Arc::new(sample.clone()),
            _ => unreachable!(),
        }
    }).collect();
    let payload = StructArray::new(fields, columns, None);
    let payload = record.encode_columns(&payload, fixture.bytes)?.remove(0);
    let scope = Scope::Execution("execution").identifier()?;
    let input = RecordInput {
        schema: record.key, scope: Some(scope.clone()), item: Identifier::new("reading")?, source: Some(Identifier::new("thermal.progress")?), source_time: None, payload,
    };

    // Samples Persist Only the Native Arrow Batch Under Its Exact Execution Scope.
    let metadata = RecordBatch::try_from_iter(vec![
        ("item", Arc::new(StringArray::from(vec!["execution"])) as ArrayRef),
        ("sequence", Arc::new(UInt64Array::from(vec![0])) as ArrayRef),
    ])?;
    let batch = RecordBatch::try_from_iter(vec![
        ("_storage", Arc::new(StructArray::from(metadata)) as ArrayRef),
        ("payload", Arc::new(sample) as ArrayRef),
    ])?;
    let data = ArrowBatch::try_new(batch)?;
    let batch = DataBatch { sequence: 0, destination: Identifier::new("thermal.progress")?, schema: binding.key, data: data.into() };
    let request = AppendSamplesRequest {
        mutation: Some(fixture.mutation("samples").await?), scope: Some(scope.to_string()), batch: Some(batch),
    };
    io::call::<AppendSamples>(operations, request.clone()).await?;
    // A Later Durable Compound Commit Fences the Accepted Sample Batch Before Replay.
    let response = io::call::<Mutate>(operations, MutateRequest {
        mutation: Some(fixture.mutation("record").await?), items: vec![MutationItem::Append(input)],
    }).await?;
    assert!(response.receipt.is_some_and(|receipt| receipt.completed));
    let response = io::call::<AppendSamples>(operations, request).await?;
    assert!(response.receipt.is_some_and(|receipt| receipt.completed && receipt.durable_at.is_some()));
    fixture.runtime.stop().await?;
    Ok(())
}
