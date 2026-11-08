/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Feature-Gated SQLite State Access for Integration Tests

/* === Modules === */

/* === Re-Exports === */

pub use super::Settings;

pub use super::data::SavedData;

pub use super::data::telemetry::aggregates::NumericSummary;

pub use super::data::telemetry::{NumericRow, StreamFile};

pub use super::history::work::Row as WorkRow;

pub use super::history::{Page, Read};

pub use super::snapshots::{Cut, Scope};

pub use crate::storage::Error;

pub use crate::domains::evidence::{ScopeSelection, TelemetrySelection};

pub use crate::plot::PlotPayloadRequest;

pub use dokime_domain::model::settings::UpdateSettings;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use super::data::telemetry;

use crate::storage::{
    Manager,
    telemetry::{TelemetryEntry, TypedCodec},
};

use harmos::storage::{
    FileMetadata, FileReference, ScanPage, SchemaKey, StagedFile, StorageBatch, StorageCursor, StoreHistory, StoredRow,
    query::StreamSelection,
};

use rusqlite::Connection;

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::config::ApplicationMetadata;

use dokime_domain::model::identity::wire::Timestamp;

use dokime_domain::model::telemetry::TelemetryContract;

/* === Constants === */

pub const SAMPLE_POSITION: &str = telemetry::SAMPLE_POSITION;

pub const SCHEMA: &str = super::history::SCHEMA;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct State(pub super::State);

pub struct QueryWork(super::QueryWork);

pub struct Checkpoint(crate::plot::preview::index::Checkpoint);

pub struct ReportRequest {
    pub execution: String,
    pub cycle: String,
    pub at: StorageCursor,
    pub current: bool,
    pub rows: usize,
    pub streams: Vec<String>,
    pub percentiles: BTreeSet<(String, String, u8)>,
}

pub struct ReportEvidence {
    pub exact: Vec<NumericSummary>,
}

pub struct Tail<'a>(std::sync::MutexGuard<'a, telemetry::Tail>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::ops::Deref for State {
    type Target = super::State;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl QueryWork {
    pub fn document(config: crate::config::QueryConfig, operation: &'static str) -> Self {
        Self(super::QueryWork::document(config, operation))
    }
}

impl State {
    pub async fn open(root: impl AsRef<Path>) -> Result<Self, Error> {
        super::State::open(root).await.map(Self)
    }
    pub async fn open_with(root: impl AsRef<Path>, settings: Settings) -> Result<Self, Error> {
        super::State::open_with(root, settings).await.map(Self)
    }
    pub async fn writer(&self) -> tokio::sync::MappedMutexGuard<'_, Connection> {
        tokio::sync::MutexGuard::map(self.0.0.live.lock().await, |live| &mut live.as_mut().unwrap().connection)
    }
    pub async fn document_statistic(
        &self, execution: String, stream: String, field: String, at: StorageCursor, aggregate: harmos::Aggregate, work: QueryWork,
    ) -> Result<Option<Option<crate::Value>>, Error> {
        self.0.document_statistic(execution, stream, field, at, aggregate, work.0).await
    }

    pub async fn data_open(&self, expected: Option<StoreHistory>) -> Result<SavedData, Error> {
        self.0.data_open(expected).await
    }

    pub async fn data_saved(&self) -> Result<SavedData, Error> {
        self.0.data_saved().await
    }

    pub async fn read<T: Send + 'static>(
        &self, operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        self.0.read(operation).await
    }

    pub async fn initialize_settings(&self) -> Result<(), Error> {
        self.0.initialize_settings().await
    }

    pub async fn read_application_metadata(&self) -> Result<ApplicationMetadata, Error> {
        self.0.read_application_metadata().await
    }

    pub async fn telemetry_streams_between(
        &self, after: StorageCursor, at: StorageCursor,
    ) -> Result<std::collections::BTreeSet<String>, Error> {
        self.0.telemetry_streams_between(after, at).await
    }

    pub async fn telemetry_schemas(
        &self, at: StorageCursor, streams: Vec<dokime_domain::model::identity::IdentifierPattern>,
    ) -> Result<Vec<(SchemaKey, Arc<harmos::schema::graph::Schema>)>, Error> {
        self.0.telemetry_schemas(at, streams).await
    }

    pub async fn telemetry_files(&self, execution: String, at: StorageCursor) -> Result<Vec<StreamFile>, Error> {
        self.0.telemetry_files(execution, at).await
    }

    pub async fn arrow_write(&self, batch: StorageBatch<StoredRow>) -> Result<SavedData, Error> {
        self.0.arrow_write(batch).await
    }

    pub async fn finalize_telemetry(&self, execution: String) -> Result<(), Error> {
        self.0.finalize_telemetry(execution).await
    }

    pub async fn register_telemetry_contract(
        &self, contract: dokime_domain::model::telemetry::TelemetryContract, codec: Arc<TypedCodec>,
    ) -> Result<(), Error> {
        self.0.register_telemetry_contract(contract, codec).await
    }

    pub async fn telemetry_read(
        &self, at: StorageCursor, selection: StreamSelection, page: ScanPage,
    ) -> Result<Vec<(u64, TelemetryEntry)>, Error> {
        self.0.telemetry_read(at, selection, page).await
    }

    pub async fn acknowledge_sample(&self, execution: String, stream: String, sample_id: String, through: u64) -> Result<u64, Error> {
        self.0.acknowledge_sample(execution, stream, sample_id, through).await
    }

    pub async fn telemetry_predecessor_page(
        &self, at: StorageCursor, schema: SchemaKey, selection: TelemetrySelection, terms: Vec<(&'static str, rusqlite::types::Value)>,
        start: std::time::SystemTime, after: String,
    ) -> Result<(String, Vec<(u64, TelemetryEntry)>), Error> {
        self.0.telemetry_predecessor_page(at, schema, selection, terms, start, after).await
    }

    pub async fn telemetry_numeric(
        &self, at: StorageCursor, selection: StreamSelection, page: ScanPage, keys: Vec<String>, scope: ScopeSelection,
    ) -> Result<Vec<(u64, NumericRow)>, Error> {
        self.0.telemetry_numeric(at, selection, page, keys, scope).await
    }

    pub async fn stage_telemetry(&self, storage: Manager, through: StorageCursor, entries: Vec<TelemetryEntry>) -> Result<(), Error> {
        self.0.stage_telemetry(storage, through, entries).await
    }

    pub async fn numeric_summaries(&self, execution: String) -> Result<Vec<NumericSummary>, Error> {
        self.0.numeric_summaries(execution).await
    }

    pub async fn register_distinct_fields(&self, execution: String, fields: BTreeMap<String, BTreeSet<String>>) -> Result<(), Error> {
        self.0.register_distinct_fields(execution, fields).await
    }

    pub async fn snapshot_capture(
        &self, scope: Scope, at: Option<StorageCursor>, metadata: Option<u64>, expires: Timestamp, contracts: Vec<u8>, journal: Option<u64>,
    ) -> Result<Option<Cut>, Error> {
        self.0.snapshot_capture(scope, at, metadata, expires, contracts, journal).await
    }

    pub async fn snapshot_available(&self, cut: Cut) -> Result<bool, Error> {
        self.0.snapshot_available(cut).await
    }

    pub async fn data_finish_file(
        &self, root: PathBuf, staging: StagedFile, position: StorageCursor, length: u64, sha256: [u8; 32],
    ) -> Result<FileReference, Error> {
        self.0.data_finish_file(root, staging, position, length, sha256).await
    }

    pub async fn production_create(&self, root: PathBuf, metadata: FileMetadata) -> Result<StagedFile, Error> {
        self.0.production_create(root, metadata).await
    }

    pub async fn production_delete(&self, root: PathBuf, file: StagedFile) -> Result<(), Error> {
        self.0.production_delete(root, file).await
    }
}

impl Checkpoint {
    pub fn payload(
        &self, request: &PlotPayloadRequest, generated_at: &str, epochs: (Option<&str>, Option<&str>),
    ) -> crate::runtime::Result<crate::plot::PlotPayload> {
        self.0.payload(request, generated_at, epochs)
    }
}

impl State {
    pub async fn register_plot_previews(
        &self, execution: String, requests: Vec<PlotPayloadRequest>, contracts: Vec<TelemetryContract>, limits: super::PreviewSettings,
        context: BTreeMap<String, crate::Value>,
    ) -> Result<(), Error> {
        self.0.register_plot_previews(execution, requests, contracts, limits.into(), context).await
    }
    pub async fn report_evidence(&self, requests: Vec<ReportRequest>) -> Result<Vec<ReportEvidence>, Error> {
        let requests = requests
            .into_iter()
            .map(|request| telemetry::report::Request {
                execution: request.execution,
                cycle: request.cycle,
                at: request.at,
                current: request.current,
                rows: request.rows,
                streams: request.streams,
                percentiles: request.percentiles,
            })
            .collect();
        self.0.report_evidence(requests).await.map(|values| values.into_iter().map(|value| ReportEvidence { exact: value.exact }).collect())
    }
    pub async fn finish_telemetry(&self) -> Result<(), Error> {
        let owner = self.0.0.clone();
        self.0.call(move |connection| Ok(owner.telemetry.lock().unwrap().finish(connection, None))).await?
    }
    pub fn hold_telemetry(&self, entered: tokio::sync::oneshot::Sender<()>, released: tokio::sync::oneshot::Receiver<()>) {
        let _store = self.0.0.telemetry.lock().unwrap();
        entered.send(()).unwrap();
        released.blocking_recv().unwrap();
    }
    pub fn fail_next_sync(&self) {
        self.0.0.telemetry.lock().unwrap().fail_next_sync();
    }
    pub async fn settle_telemetry(&self) -> Result<(), Error> {
        for task in std::mem::take(&mut *self.0.0.telemetry_tasks.lock().await) {
            task.await??;
        }
        Ok(())
    }
    pub fn tail(&self) -> Tail<'_> {
        Tail(self.0.0.telemetry_tail.lock().unwrap())
    }
}

impl Tail<'_> {
    pub fn insert(&mut self, position: u64, entry: TelemetryEntry) {
        self.0.insert(position, entry);
    }
    pub fn set_running(&mut self, value: bool) {
        self.0.running = value;
    }
    pub fn positions(&self) -> Vec<u64> {
        self.0.entries.keys().copied().collect()
    }
    pub fn started(&self) -> Option<tokio::time::Instant> {
        self.0.entries.values().next().map(|value| value.0)
    }
}

/* === Public Functions === */

pub fn work_schema() -> SchemaKey {
    super::history::work::schema()
}

pub fn position(value: u64) -> String {
    super::data::position(value)
}

pub fn sequence(value: &str) -> Result<u64, Error> {
    super::data::sequence(value)
}

pub fn time_key(value: Option<Timestamp>) -> Vec<u8> {
    super::history::time_key(value)
}

pub fn check_format(connection: &Connection) -> Result<(), Error> {
    super::history::check_format(connection)
}

pub fn executions(
    connection: &Connection, selection: &dokime_domain::model::query::selection::ExecutionSelection, read: &Read,
) -> Result<Page<dokime_domain::model::query::page::ExecutionSummary>, Error> {
    super::history::executions(connection, selection, read)
}

pub fn cycles(
    connection: &Connection, selection: &dokime_domain::model::query::selection::CycleSelection, read: &Read,
) -> Result<Page<dokime_domain::model::query::page::CycleSummary>, Error> {
    super::history::cycles(connection, selection, read)
}

pub fn index(connection: &Connection, object: &harmos::storage::ObjectEntry) -> Result<(), Error> {
    super::history::index(connection, object)
}

pub fn index_work(connection: &Connection, record: &harmos::storage::StoredRecord) -> Result<(), Error> {
    super::history::work::index(connection, record)
}

pub fn published_content(connection: &Connection, reference: &FileReference) -> Result<bool, Error> {
    super::data::files::published_content(connection, reference)
}

pub fn checkpoints(connection: &Connection, execution: &str, at: u64) -> Result<BTreeMap<String, Checkpoint>, Error> {
    crate::plot::preview::index::checkpoints(connection, execution, at)
        .map(|values| values.into_iter().map(|(key, value)| (key, Checkpoint(value))).collect())
}

pub async fn project_saved(
    session: &Manager, local: &State, scope: &dokime_domain::model::execution::RecordScope, contracts: &[TelemetryContract],
) -> crate::runtime::Result<Vec<crate::streams::AggregateOutput>> {
    crate::domains::telemetry::output::project_saved(session, &local.0, scope, contracts).await
}

/* === Private Functions === */

/* === Tests === */
