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

pub use crate::runtime::storage::record_capture::CapturedRecord;

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::{
    runtime::Result,
    runtime::storage::record_capture::{Process, Source},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct RecordSubscription {
    source: Arc<Source>,
    state_manager: crate::app::StateManager,
    delivered: u64,
    reader: Process,
}

pub struct CapturedRecords {
    source: Arc<Source>,
    state_manager: crate::app::StateManager,
    epoch: String,
    frontier: u64,
    reader: tokio::sync::Mutex<Process>,
    local: crate::storage::sqlite::State,
    scope: crate::evidence_runtime::ScopeSelection,
    selected: tokio::sync::Mutex<(u64, std::collections::VecDeque<CapturedRecord>)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RecordSubscription {
    pub(crate) fn new(source: &Arc<Source>, state_manager: &crate::app::StateManager, after: harmos::state::Position) -> Self {
        Self {
            source: source.clone(),
            state_manager: state_manager.clone(),
            delivered: after.0,
            reader: source.process(),
        }
    }

    pub async fn next(&mut self) -> Option<Result<CapturedRecord>> {
        loop {
            self.reader.release_completed(self.delivered);
            let through = match self.source.wait_after(&self.state_manager, self.delivered).await {
                Ok(Some(through)) => through,
                Ok(None) => return None,
                Err(error) => return Some(Err(error)),
            };
            match self.reader.next(&self.source, &self.state_manager, self.delivered, through).await {
                Ok(Some(record)) => {
                    self.delivered = record.sequence;
                    return Some(Ok(record));
                }
                Ok(None) => self.delivered = through,
                Err(error) => return Some(Err(error)),
            }
        }
    }
}

impl CapturedRecords {
    pub fn select_scope(mut self, scope: crate::evidence_runtime::ScopeSelection) -> Self {
        self.scope = scope;
        self
    }

    pub fn epoch(&self) -> &str {
        &self.epoch
    }

    pub fn frontier(&self) -> u64 {
        self.frontier
    }

    pub async fn next(&self, after: u64, through: u64) -> Result<Option<CapturedRecord>> {
        if self.scope.test_execution_id.is_some() || self.scope.test_cycle_id.is_some() {
            if after > through {
                return Err(crate::runtime::Error::Operation("invalid record capture range".into()));
            }
            self.source.wait_through(&self.state_manager, through).await?;
            let mut pending = self.selected.lock().await;
            if after < pending.0 || pending.1.front().is_some_and(|record| record.sequence > through) {
                pending.1.clear();
            }
            while pending.1.front().is_some_and(|record| record.sequence <= after) {
                pending.1.pop_front();
            }
            if pending.1.is_empty() {
                let at = self.source.save_accepted(&self.state_manager).await?;
                pending.1.extend(
                    self.local
                        .scoped_records(at, self.scope.clone(), Some(self.epoch.clone()), 0, after, through)
                        .await?
                        .into_iter()
                        .map(|row| CapturedRecord::from_entry(row.position, row.record)),
                );
            }
            pending.0 = after;
            return Ok(pending.1.front().cloned());
        }
        let mut reader = self.reader.lock().await;
        reader.release_completed(after);
        reader.next(&self.source, &self.state_manager, after, through).await
    }

    pub async fn wait_after(&self, after: u64) -> Result<Option<u64>> {
        self.reader.lock().await.release_completed(after);
        self.source.wait_after(&self.state_manager, after).await
    }
}

impl super::RuntimeHandle {
    pub fn process_epoch(&self) -> dokime_domain::model::identity::SessionId {
        self.state_read().process_epoch().clone()
    }

    pub fn captured_records(&self) -> CapturedRecords {
        self.captured_records_in(&self.storage().clone())
    }

    pub fn captured_records_in(&self, session: &crate::storage::Manager) -> CapturedRecords {
        CapturedRecords {
            source: self.record_source.clone(),
            state_manager: self.state_manager.clone(),
            epoch: self.record_source.epoch().to_owned(),
            frontier: self.state_manager.applied().0,
            reader: tokio::sync::Mutex::new(Process::new(session.clone())),
            local: self.local_storage.clone(),
            scope: Default::default(),
            selected: tokio::sync::Mutex::new((0, std::collections::VecDeque::new())),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
