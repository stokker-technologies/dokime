/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

use crate::domains::evidence::{ScopeSelection, TelemetrySelection};

use harmos::storage::{ScanPage, StorageCursor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct State(crate::storage::sqlite::State);

pub struct PreparedSample {
    pub sample: Arc<crate::telemetry::TelemetrySample>,
    pub lease: Option<crate::RecordScope>,
    pub contract: usize,
    pub ordinal: u64,
}

pub struct Rows(Vec<(usize, ScopedTelemetry)>);

pub struct Admission {
    pub storage: Manager,
    pub local: State,
    pub rows: Rows,
}

pub struct Committed {
    pub accepted: Vec<bool>,
    pub failure: Option<Error>,
}

pub struct File(crate::storage::sqlite::StreamFile);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl File {
    pub fn rows(&self) -> u64 {
        self.0.rows
    }
    pub fn path(&self) -> &std::path::Path {
        &self.0.path
    }
    pub fn summary(&self, signal: &str) -> Option<(u64, f64)> {
        self.0.summaries.iter().find(|summary| summary.signal == signal).map(|summary| (summary.count, summary.sum))
    }
}

impl std::ops::Deref for State {
    type Target = crate::storage::sqlite::State;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl State {
    pub async fn read<T: Send + 'static>(
        &self, operation: impl FnOnce(&mut rusqlite::Connection) -> std::result::Result<T, rusqlite::Error> + Send + 'static,
    ) -> crate::storage::Result<T> {
        self.0.read(operation).await
    }

    pub fn new(state: crate::storage::sqlite::State) -> Self {
        Self(state)
    }
    pub async fn register_telemetry_contract(
        &self, contract: crate::telemetry::TelemetryContract, codec: Arc<telemetry::TypedCodec>,
    ) -> crate::storage::Result<()> {
        self.0.register_telemetry_contract(contract, codec).await
    }
    pub async fn telemetry_schemas(
        &self, at: StorageCursor, streams: Vec<crate::IdentifierPattern>,
    ) -> crate::storage::Result<Vec<(SchemaKey, Arc<harmos::schema::graph::Schema>)>> {
        self.0.telemetry_schemas(at, streams).await
    }
    pub async fn telemetry_numeric(
        &self, at: StorageCursor, selection: StreamSelection, page: ScanPage, keys: Vec<String>, scope: ScopeSelection,
    ) -> crate::storage::Result<Vec<(u64, Vec<Option<f64>>)>> {
        let rows = self.0.telemetry_numeric(at, selection, page, keys, scope).await?;
        Ok(rows.into_iter().map(|(position, row)| (position, row.values)).collect())
    }
    pub async fn latest_telemetry_time(
        &self, at: StorageCursor, schema: SchemaKey, terms: Vec<(&'static str, rusqlite::types::Value)>, selection: TelemetrySelection,
        codec: telemetry::TypedCodec,
    ) -> crate::storage::Result<Option<std::time::SystemTime>> {
        self.0.latest_telemetry_time(at, schema, terms, selection, codec).await
    }
    pub async fn telemetry_predecessor_page(
        &self, at: StorageCursor, schema: SchemaKey, selection: TelemetrySelection, terms: Vec<(&'static str, rusqlite::types::Value)>,
        start: std::time::SystemTime, after: String,
    ) -> crate::storage::Result<(String, Vec<(u64, TelemetryEntry)>)> {
        self.0.telemetry_predecessor_page(at, schema, selection, terms, start, after).await
    }
    pub async fn telemetry_files(&self, execution: String, at: StorageCursor) -> crate::storage::Result<Vec<File>> {
        self.0.telemetry_files(execution, at).await.map(|files| files.into_iter().map(File).collect())
    }
    pub async fn telemetry_read(
        &self, at: StorageCursor, selection: StreamSelection, page: ScanPage,
    ) -> crate::storage::Result<Vec<(u64, TelemetryEntry)>> {
        self.0.telemetry_read(at, selection, page).await
    }
}

impl Admission {
    pub async fn commit(self, observe_closure: bool) -> Result<Committed> {
        let sampling = observe_closure.then(crate::services::telemetry::Sampling::new);
        let committed = TelemetryAdmission {
            storage: self.storage,
            local: self.local.0,
            rows: self.rows.0,
            _owners: Vec::new(),
        }
        .commit(sampling.as_ref())
        .await?;
        Ok(Committed {
            accepted: committed.accepted,
            failure: committed.failure,
        })
    }
}

/* === Public Functions === */

pub fn decode_payload(
    codec: &telemetry::TypedCodec, payload: &[u8],
) -> crate::storage::Result<std::collections::BTreeMap<String, crate::Value>> {
    codec.decode_payload(payload)
}

pub fn stage_rows(epoch: &crate::SessionId, samples: &[PreparedSample], codecs: &[Arc<telemetry::TypedCodec>]) -> Result<Rows> {
    let samples = samples
        .iter()
        .map(|sample| crate::domains::telemetry::PreparedSample {
            sample: sample.sample.clone(),
            lease: sample.lease.clone(),
            contract: sample.contract,
            ordinal: sample.ordinal,
        })
        .collect::<Vec<_>>();
    super::stage_rows(epoch, &samples, codecs, &crate::domains::telemetry::Payloads::Unchecked).map(Rows)
}

pub async fn saved_codecs(
    session: &Manager, at: &StorageCursor, schema: Option<&SchemaKey>, settings: &harmos::config::SchemaSettings,
    selected: impl Fn(&str, u32) -> bool + Send + Sync,
) -> Result<harmos::storage::ReadBatch<Vec<telemetry::TypedCodec>>> {
    super::saved_codecs(session, at, schema, settings, selected).await
}

pub async fn collect_saved(
    session: &Manager, at: &StorageCursor, selection: &TelemetrySelection, settings: &harmos::config::SchemaSettings, local: &State,
) -> Result<harmos::storage::ReadBatch<Vec<crate::telemetry::TelemetrySample>>> {
    super::collect_saved(session, at, selection, settings, Default::default(), &local.0).await
}

pub async fn collect_page(
    session: &Manager, at: &StorageCursor, selection: &TelemetrySelection, settings: &harmos::config::SchemaSettings, local: &State,
) -> Result<TelemetryPage> {
    super::collect_page(session, at, selection, settings, Default::default(), &local.0).await
}

/* === Private Functions === */

/* === Tests === */
