/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Local History Selection And Bounded Seek Pages

/* === Modules === */

/* === Re-Exports === */

pub use crate::storage::sqlite::history::Key;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{
    RuntimeHandle,
    config::QueryConfig,
    storage::sqlite::{
        State,
        history::{Page, Read},
    },
};
use dokime_domain::model::{
    execution::wire::ExecutionStatus,
    identity::{Identifier, TestCycleId},
    query::{
        error::{Failure, Reason},
        page::{CycleSummary, ExecutionSummary, PageLimits},
        selection::{CycleSelection, ExecutionSelection},
    },
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct History {
    local: State,
    config: QueryConfig,
}

pub struct HistoryPage<T> {
    pub rows: Vec<T>,
    pub next: Option<Key>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub fn query_history(&self) -> History {
        History {
            local: self.local_storage.clone(),
            config: self.query_config().clone(),
        }
    }
}

impl History {
    pub async fn revision(&self) -> Result<i64, Box<Failure>> {
        self.local.history_revision().await.map_err(storage_error)
    }

    pub async fn executions(
        &self, selection: ExecutionSelection, revision: i64, after: Option<Key>, limits: PageLimits,
    ) -> Result<HistoryPage<ExecutionSummary>, Box<Failure>> {
        let operation = "dokime.query.list_executions";
        self.config.bound(selection.template_ids.len(), self.config.limits().alternatives, "templates", operation)?;
        self.config.bound(selection.cycle_ids.len(), self.config.limits().alternatives, "cycles", operation)?;
        self.config.bound(selection.statuses.len(), self.config.limits().alternatives, "statuses", operation)?;
        for id in &selection.template_ids {
            Identifier::new(id).map_err(|_| Failure::new(Reason::InvalidSelection, operation, "Invalid template identifier"))?;
        }
        for id in &selection.cycle_ids {
            TestCycleId::from_uuid(id).map_err(|_| Failure::new(Reason::InvalidSelection, operation, "Invalid cycle identifier"))?;
        }
        if selection.statuses.contains(&ExecutionStatus::Unspecified) {
            return Err(Failure::new(Reason::InvalidSelection, operation, "Select a declared execution status"));
        }
        if let Some(window) = &selection.started_at {
            window.validate(operation)?;
        }
        self.validate(revision, &after, &limits, operation)?;
        let Page { rows, next } = self
            .local
            .history_executions(
                selection,
                Read {
                    revision,
                    after,
                    rows: limits.rows,
                    bytes: limits.bytes,
                },
            )
            .await
            .map_err(storage_error)?;
        Ok(HistoryPage { rows, next })
    }

    pub async fn cycles(
        &self, selection: CycleSelection, revision: i64, after: Option<Key>, limits: PageLimits,
    ) -> Result<HistoryPage<CycleSummary>, Box<Failure>> {
        let operation = "dokime.query.list_cycles";
        self.config.bound(selection.cycle_ids.len(), self.config.limits().alternatives, "cycles", operation)?;
        self.config.bound(selection.phases.len(), self.config.limits().alternatives, "phases", operation)?;
        for id in &selection.cycle_ids {
            TestCycleId::from_uuid(id).map_err(|_| Failure::new(Reason::InvalidSelection, operation, "Invalid cycle identifier"))?;
        }
        if let Some(window) = &selection.created_at {
            window.validate(operation)?;
        }
        self.validate(revision, &after, &limits, operation)?;
        let Page { rows, next } = self
            .local
            .history_cycles(
                selection,
                Read {
                    revision,
                    after,
                    rows: limits.rows,
                    bytes: limits.bytes,
                },
            )
            .await
            .map_err(storage_error)?;
        Ok(HistoryPage { rows, next })
    }

    fn validate(&self, revision: i64, after: &Option<Key>, limits: &PageLimits, operation: &str) -> Result<(), Box<Failure>> {
        if revision < 0
            || limits.rows == 0
            || limits.bytes == 0
            || limits.rows > self.config.limits().max_page_rows
            || limits.bytes > self.config.limits().max_page_bytes
            || after.as_ref().is_some_and(|key| key.time.len() != 16 || key.id.len() > self.config.limits().token_bytes)
        {
            return Err(Failure::new(Reason::InvalidSelection, operation, "Invalid history cut, seek key or page limits"));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(super) fn storage_error(error: crate::storage::Error) -> Box<Failure> {
    let reason = match error {
        crate::storage::Error::Data(_) => Reason::ResourceLimit,
        crate::storage::Error::Corrupt(_) | crate::storage::Error::Decode(_) | crate::storage::Error::Json(_) => Reason::Corruption,
        crate::storage::Error::Incompatible(_) => Reason::SnapshotUnavailable,
        _ => Reason::StorageUnavailable,
    };
    Failure::new(reason, "dokime.query.history", "Local query storage could not satisfy the requested cut")
}

/* === Tests === */
