/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story

/* === Modules === */
mod contention;
mod files;
mod ingestion;
mod query;
mod transforms;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use criterion::Criterion;
use harmos::{Application, Identifier, Message, Runtime, state::Never, storage::*};

// Local Imports
use dokime::storage::sqlite::{Backend, Limits};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct App;
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Measurement {
    value: f64,
}

struct Fixture {
    _root: tempfile::TempDir,
    storage: StorageManager<()>,
    backend: Arc<Backend<()>>,
    runtime: Runtime<App>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Application for App {
    type State = ();
    type Transaction = Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

impl Fixture {
    async fn new(concurrency: usize) -> Self {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(
            Backend::<()>::primary(
                root.path().to_owned(),
                Limits {
                    entry_bytes: nz(1024 * 1024),
                    batch_bytes: nz(16 * 1024 * 1024),
                },
            )
            .await
            .unwrap(),
        );
        let mut policy = dokime_app::config::StorageSettings::default();
        policy.budget.read_memory_bytes = nz(128 * 1024 * 1024);
        policy.request.max_active_queries = nz(concurrency);
        policy.interactive.memory_bytes = nz(64 * 1024 * 1024);
        policy.ceiling.memory_bytes = nz(128 * 1024 * 1024);
        policy.decoded_bytes = nz(64 * 1024 * 1024);
        policy.page_bytes_ceiling = nz(128 * 1024 * 1024);
        let settings = policy.harmos(Duration::from_millis(10), nz(16 * 1024 * 1024));
        let storage = StorageManager::new(
            StorageBackends {
                state: Arc::new(harmos::storage::state::file::FileBackend::new(
                    root.path().to_owned(),
                    harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
                    &settings,
                )),
                primary: backend.clone(),
                replicas: vec![],
            },
            settings,
        )
        .unwrap();
        let runtime = Runtime::<App>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let graph = harmos::schema::graph::Schema::from_declaration(
            <Measurement as harmos::schema::Described>::DEFINITION,
            &harmos::config::SchemaSettings::default(),
        )
        .unwrap();
        storage.write_schema(&schema(), &graph).await.unwrap();
        storage.flush().await.unwrap();
        Self {
            _root: root,
            storage,
            runtime,
            backend,
        }
    }

    fn plan(
        &self,
        transforms: Vec<harmos::storage::query::transform::Transform>,
        slice: harmos::storage::query::RowSlice,
    ) -> harmos::storage::query::Query {
        use harmos::storage::query::{Query, Source, StreamSelection};
        Query {
            at: self.storage.cursor_saved(),
            source: Source::Stream(StreamSelection {
                schema: schema(),
                scope: Some(id("execution")),
                item: None,
                stream: Some(id("voltage")),
                source_start: None,
                source_end: None,
            }),
            transforms,
            slice,
        }
    }

    async fn collect(&self, plan: harmos::storage::query::Query) -> (u64, Duration) {
        use harmos::storage::query::QueryOutput;
        let started = std::time::Instant::now();
        let mut reader = self.storage.query(plan).await.unwrap();
        let mut first = None;
        while let Some(output) = reader.next().await.unwrap() {
            if matches!(output, QueryOutput::Batch(_)) && first.is_none() {
                first = Some(started.elapsed());
            }
        }
        let completion = reader.completion().unwrap();
        (completion.rows, first.unwrap_or_else(|| started.elapsed()))
    }

    async fn append(&self, start: u64, payloads: &[Vec<u8>]) -> StorageCursor {
        let items = (start..start + payloads.len() as u64)
            .map(|index| id(&format!("sample-{index}")))
            .collect::<Vec<_>>();
        self.append_with_ids(start, payloads, &items).await
    }

    async fn append_with_ids(
        &self,
        start: u64,
        payloads: &[Vec<u8>],
        items: &[Identifier],
    ) -> StorageCursor {
        let schema = schema();
        let scope = id("execution");
        let stream = id("voltage");
        let writes = payloads
            .iter()
            .zip(items)
            .enumerate()
            .map(|(index, (payload, item))| StreamWrite {
                stream: &stream,
                sequence: start + index as u64,
                record: RecordWrite {
                    schema: &schema,
                    scope: Some(&scope),
                    item,
                    source: None,
                    source_time: Some(
                        std::time::UNIX_EPOCH + Duration::from_micros(start + index as u64),
                    ),
                    payload,
                },
            })
            .collect::<Vec<_>>();
        self.storage.write_stream(&writes).await.unwrap()
    }
}

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    ingestion::benchmark(criterion);
    query::benchmark(criterion);
    transforms::benchmark(criterion);
    files::benchmark(criterion);
    contention::benchmark(criterion);
}

/* === Private Functions === */
fn nz(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}
fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}
fn schema() -> SchemaKey {
    SchemaKey {
        id: id("bench.measurement"),
        version: std::num::NonZeroU32::MIN,
        payload_encoding: PayloadEncoding::PostcardV1,
    }
}
fn samples(rows: usize) -> Vec<Vec<u8>> {
    (0..rows)
        .map(|index| {
            Measurement {
                value: 325.0 * (std::f64::consts::TAU * index as f64 / 64.0).sin(),
            }
            .encode_to_vec()
        })
        .collect()
}
fn executor() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
}

/* === Tests === */
