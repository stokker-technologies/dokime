/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "history.rs"]
mod history;

/* === Re-Exports === */
pub use history::CapturedRecord;

/* === Imports === */

// Standard Library Imports
use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

// Third Party Imports
use futures_util::StreamExt;
use harmos::{
    Identifier, Message,
    storage::{ReadLimits, RecordWrite, SchemaKey},
};
use tokio::sync::watch;
use tracing::Instrument;

// Local Imports
use super::error;
use crate::runtime::Error;
use crate::runtime::Result;
use crate::storage::records;
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */
const FRAME_BYTES: usize = 1024 * 1024;
const MAX_GROUP_ROWS: usize = 256;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Source {
    storage: crate::storage::Manager,
    schema: SchemaKey,
    epoch: Identifier,
    closed: watch::Sender<bool>,
    origin: AtomicU64,
}

pub(crate) struct Projection(pub(crate) Arc<Source>);

#[derive(serde::Deserialize)]
struct PersistedRecord {
    dokime_encoding: u32,
    payload: dokime_domain::model::records::evidence::Record,
}

#[derive(Default)]
pub(crate) struct Process {
    history: Option<history::History>,
    through: u64,
    before: u64,
    position: u64,
    physical: u64,
    last: Option<history::CapturedRecord>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Source {
    pub(crate) fn new(epoch: &dokime_domain::model::identity::SessionId, storage: crate::storage::Manager) -> Result<Self> {
        Ok(Self {
            storage,
            schema: records::schema(),
            epoch: Identifier::new(epoch.to_string()).map_err(error)?,
            closed: watch::channel(false).0,
            origin: AtomicU64::new(0),
        })
    }

    pub(crate) async fn initialize(&self) -> Result<()> {
        use harmos::schema::Described;
        let schema =
            harmos::schema::graph::Schema::from_declaration(records::RecordRow::DEFINITION, &harmos::config::SchemaSettings::default())
                .map_err(projection_error)?;
        let declared = self.storage.write_schema(&self.schema, &schema).await.map_err(Error::from)?;
        self.storage.flush_until(&declared).await.map_err(Error::from)?;
        Ok(())
    }

    fn close(&self) {
        self.closed.send_replace(true);
    }

    pub(crate) fn begin(&self, origin: harmos::state::Position) {
        self.origin.store(origin.0, Ordering::Release);
    }

    pub(crate) fn epoch(&self) -> &str {
        self.epoch.as_str()
    }

    pub(crate) fn origin(&self) -> u64 {
        self.origin.load(Ordering::Acquire)
    }

    fn healthy(&self, state_manager: &crate::app::StateManager) -> Result<()> {
        if let Some(failure) = self.storage.status().failure {
            return Err(error(failure));
        }
        state_manager.capture_status().map_err(error)
    }

    pub(crate) async fn wait_through(&self, state_manager: &crate::app::StateManager, frontier: u64) -> Result<()> {
        self.healthy(state_manager)?;
        state_manager.wait_until_captured(harmos::state::Position(frontier)).await.map_err(error)?;
        self.healthy(state_manager)
    }

    pub(crate) async fn wait_after(&self, state_manager: &crate::app::StateManager, after: u64) -> Result<Option<u64>> {
        let mut progress = self.closed.subscribe();
        loop {
            self.healthy(state_manager)?;
            let frontier = state_manager.applied().0;
            if frontier > after {
                self.wait_through(state_manager, frontier).await?;
                return Ok(Some(frontier));
            }
            if *progress.borrow_and_update() {
                return Ok(None);
            }
            let watched = state_manager.watch::<harmos::state::Recorded<crate::app::Dokime>>(harmos::state::Position(after));
            tokio::pin!(watched);
            tokio::select! {
                _ = progress.changed() => {},
                entry = watched.next() => {
                    // A racing burst may evict the watched tail. Only the applied
                    // frontier is needed here; primary rows own the actual replay.
                    if state_manager.applied().0 <= after && !*progress.borrow() {
                        match entry {
                            Some(Err(failure)) => return Err(error(failure)),
                            None => return Ok(None),
                            Some(Ok(_)) => {},
                        }
                    }
                }
            }
        }
    }
}

impl harmos::storage::RecordProjection for Projection {
    fn project<'a>(
        &'a self, envelope: &'a harmos::state::Envelope<&serde_json::Value>, output: &'a mut harmos::storage::ProjectedRecords<'_>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = harmos::Result<()>> + Send + 'a>> {
        let span = tracing::debug_span!(
            target: "crate::record_admission",
            "record_projection"
        );
        Box::pin(
            async move {
                let Some((item, payload)) = project(envelope)? else {
                    return Ok(());
                };
                output
                    .push(RecordWrite {
                        schema: &self.0.schema,
                        scope: Some(&self.0.epoch),
                        item: &item,
                        source: None,
                        source_time: None,
                        payload: &payload,
                    })
                    .await
            }
            .instrument(span),
        )
    }
}

impl Drop for Projection {
    fn drop(&mut self) {
        self.0.close();
    }
}

impl Process {
    pub(crate) fn release_completed(&mut self, after: u64) {
        if after >= self.through {
            self.history = None;
            self.last = None;
        }
    }

    pub(crate) async fn next(
        &mut self, source: &Arc<Source>, state_manager: &crate::app::StateManager, after: u64, through: u64,
    ) -> Result<Option<history::CapturedRecord>> {
        if after > through {
            return Err(error("invalid record capture range"));
        }
        source.wait_through(state_manager, through).await?;
        if through <= source.origin() || after == through {
            self.release_completed(after);
            return Ok(None);
        }
        let after = after.max(source.origin());
        if self.through == through
            && let Some(last) = &self.last
            && after >= self.before
            && after < last.sequence
        {
            return Ok(Some(last.clone()));
        }
        let rewind = after < self.position;
        if self.history.is_none() || self.through != through || rewind {
            let start = if through >= self.through && after >= self.position && self.physical > 0 {
                history::Start::Physical(self.physical)
            } else if after > source.origin() {
                history::Start::Logical(after)
            } else {
                history::Start::Physical(0)
            };
            // Release the previous view before admitting another bounded reader.
            self.history = None;
            self.last = None;
            self.position = after;
            self.physical = 0;
            self.history = Some(source.history_scoped(state_manager, start, through, Some(source.epoch.clone())).await?);
            self.through = through;
        }
        loop {
            let next = self.history.as_mut().unwrap().next().await;
            let record = match next {
                Ok(Some(row)) => row,
                Ok(None) => break,
                Err(failure) => {
                    // Retry Keeps The Same Fixed View And Original Pending Page
                    return Err(failure);
                }
            };
            self.position = record.sequence;
            self.physical = record.position();
            if record.sequence > after {
                self.before = after;
                self.last = Some(record.clone());
                return Ok(Some(record));
            }
        }
        self.position = through;
        self.release_completed(through);
        Ok(None)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn projection_error(source: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> harmos::Error {
    harmos::Error::Backend { operation: "project Dokime record", kind: harmos::BackendFailure::Permanent, source: source.into() }
}

// Restore One Journal Envelope As The Record Row It Captures, If Any.
fn project(envelope: &harmos::state::Envelope<&serde_json::Value>) -> harmos::Result<Option<(Identifier, Vec<u8>)>> {
    let record = if let Some(fact) =
        harmos::state::SystemFact::restore(&envelope.id, envelope.version, envelope.payload).map_err(projection_error)?
    {
        let harmos::state::FactSource::Sidecar(owner) = fact.source else {
            return Ok(None);
        };
        let persisted: PersistedRecord = serde_json::from_slice(&fact.payload).map_err(projection_error)?;
        if persisted.dokime_encoding != 2 {
            return Err(projection_error("unsupported Dokime persistence encoding"));
        }
        let record = persisted.payload;
        if record.source != owner || record.record_type.as_str() != fact.name || record.schema_version != fact.version {
            return Err(projection_error("native fact identity differs from its envelope"));
        }
        record
    } else {
        let Some(dokime_domain::state::records::Records::Record(record)) =
            dokime_domain::state::records::Records::restore(&envelope.id, envelope.version, envelope.payload).map_err(projection_error)?
        else {
            return Err(projection_error("unknown product record"));
        };
        record
    };
    let record = RecordEntry::from_record(envelope.position.0, record);
    let row = records::RecordRow::from_entry(&record).map_err(projection_error)?;
    let payload = row.encode_to_vec();
    if payload.len() > FRAME_BYTES {
        return Err(projection_error("record capture frame exceeds byte limit"));
    }
    let item = Identifier::new(record.record_id.as_str()).map_err(projection_error)?;
    Ok(Some((item, payload)))
}

pub(crate) fn group_limits() -> ReadLimits {
    ReadLimits { rows: NonZeroUsize::new(MAX_GROUP_ROWS).unwrap(), bytes: NonZeroUsize::new(4 * FRAME_BYTES).unwrap() }
}

/* === Tests === */
