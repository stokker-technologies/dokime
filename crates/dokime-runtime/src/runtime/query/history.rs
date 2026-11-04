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
use harmos::Message;

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
        &self, selection: ExecutionSelection, cut: super::snapshots::Cut, after: Option<Key>, limits: PageLimits,
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
        self.validate(cut.catalog, &after, &limits, operation)?;
        let Page { rows, next } = self
            .local
            .history_executions(
                selection,
                Read {
                    revision: cut.catalog,
                    cut: Some(cut),
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
        &self, selection: CycleSelection, cut: super::snapshots::Cut, after: Option<Key>, limits: PageLimits,
    ) -> Result<HistoryPage<CycleSummary>, Box<Failure>> {
        let operation = "dokime.query.list_cycles";
        self.config.bound(selection.cycle_ids.len(), self.config.limits().alternatives, "cycles", operation)?;
        self.config.bound(selection.phases.len(), self.config.limits().alternatives, "phases", operation)?;
        self.config.bound(selection.statuses.len(), self.config.limits().alternatives, "statuses", operation)?;
        if selection.statuses.contains(&dokime_domain::model::execution::wire::CycleStatus::Unspecified) {
            return Err(Failure::new(Reason::InvalidSelection, operation, "Select a declared cycle status"));
        }
        for id in &selection.cycle_ids {
            TestCycleId::from_uuid(id).map_err(|_| Failure::new(Reason::InvalidSelection, operation, "Invalid cycle identifier"))?;
        }
        if let Some(window) = &selection.created_at {
            window.validate(operation)?;
        }
        self.validate(cut.catalog, &after, &limits, operation)?;
        let Page { rows, next } = self
            .local
            .history_cycles(
                selection,
                Read {
                    revision: cut.catalog,
                    cut: Some(cut),
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

impl RuntimeHandle {
    pub async fn query_list_executions(
        &self, request: dokime_operations::query::ListExecutions, message_bytes: usize,
    ) -> Result<dokime_domain::model::query::page::ExecutionPage, Box<Failure>> {
        let operation = "dokime.query.list_executions";
        tokio::time::timeout(self.query_config().limits().active_time, async {
            request.validate_shape()?;
            let limits = self.query_config().page_limits(&request.page, message_bytes, operation)?;
            let binding = super::tokens::binding(&request, self.query_config().limits().working_bytes as usize, operation)?;
            let snapshot = self
                .query_snapshot(super::snapshots::Scope::History, request.at.as_ref(), request.page.after.as_deref(), binding, operation)
                .await?;
            let after = history_key(snapshot.after.as_ref(), operation)?;
            let reserve = self.query_continuation(&snapshot, binding, reserved_key(), operation).await?;
            let mut response = dokime_domain::model::query::page::ExecutionPage {
                rows: Vec::new(),
                schema: None,
                snapshot: Some(snapshot.snapshot.clone()),
                limits: Some(limits.clone()),
                next: Some(reserve),
                complete: false,
            };
            let admitted = page_budget(&response, &limits, operation)?;
            let page = self.query_history().executions(request.selection, snapshot.cut.clone(), after, admitted).await?;
            response.next = match page.next {
                Some(key) => Some(self.query_continuation(&snapshot, binding, super::snapshots::Seek::History(key), operation).await?),
                None => None,
            };
            response.rows = page.rows;
            response.complete = response.next.is_none();
            check_page(&response, &limits, operation)?;
            Ok(response)
        })
        .await
        .map_err(|_| Failure::new(Reason::Deadline, operation, "History query exceeded its configured active-work deadline"))?
    }

    pub async fn query_list_cycles(
        &self, request: dokime_operations::query::ListCycles, message_bytes: usize,
    ) -> Result<dokime_domain::model::query::page::CyclePage, Box<Failure>> {
        let operation = "dokime.query.list_cycles";
        tokio::time::timeout(self.query_config().limits().active_time, async {
            request.validate_shape()?;
            let limits = self.query_config().page_limits(&request.page, message_bytes, operation)?;
            let binding = super::tokens::binding(&request, self.query_config().limits().working_bytes as usize, operation)?;
            let snapshot = self
                .query_snapshot(super::snapshots::Scope::History, request.at.as_ref(), request.page.after.as_deref(), binding, operation)
                .await?;
            let after = history_key(snapshot.after.as_ref(), operation)?;
            let reserve = self.query_continuation(&snapshot, binding, reserved_key(), operation).await?;
            let mut response = dokime_domain::model::query::page::CyclePage {
                rows: Vec::new(),
                schema: None,
                snapshot: Some(snapshot.snapshot.clone()),
                limits: Some(limits.clone()),
                next: Some(reserve),
                complete: false,
            };
            let admitted = page_budget(&response, &limits, operation)?;
            let page = self.query_history().cycles(request.selection, snapshot.cut.clone(), after, admitted).await?;
            response.next = match page.next {
                Some(key) => Some(self.query_continuation(&snapshot, binding, super::snapshots::Seek::History(key), operation).await?),
                None => None,
            };
            response.rows = page.rows;
            response.complete = response.next.is_none();
            check_page(&response, &limits, operation)?;
            Ok(response)
        })
        .await
        .map_err(|_| Failure::new(Reason::Deadline, operation, "History query exceeded its configured active-work deadline"))?
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn reserved_key() -> super::snapshots::Seek {
    super::snapshots::Seek::History(Key {
        missing: false,
        time: vec![255; 16],
        id: "f".repeat(36),
    })
}

fn history_key(seek: Option<&super::snapshots::Seek>, operation: &str) -> Result<Option<Key>, Box<Failure>> {
    match seek {
        None => Ok(None),
        Some(super::snapshots::Seek::History(key)) => Ok(Some(key.clone())),
        _ => Err(Failure::new(Reason::InvalidContinuation, operation, "Continuation does not identify a history key")),
    }
}

fn page_budget(value: &impl serde::Serialize, limits: &PageLimits, operation: &str) -> Result<PageLimits, Box<Failure>> {
    let overhead = super::tokens::bounded_json(value, limits.bytes as usize, operation)?.len() as u64;
    let bytes = limits
        .bytes
        .checked_sub(overhead + u64::from(limits.rows))
        .filter(|bytes| *bytes > 0)
        .ok_or_else(|| Failure::new(Reason::ResourceLimit, operation, "Page metadata exceeds the effective byte ceiling"))?;
    Ok(PageLimits { rows: limits.rows, bytes })
}

fn check_page(value: &(impl serde::Serialize + Message), limits: &PageLimits, operation: &str) -> Result<(), Box<Failure>> {
    super::tokens::bounded_json(value, limits.bytes as usize, operation)?;
    if value.encoded_size() as u64 > limits.bytes {
        return Err(Failure::new(Reason::ResourceLimit, operation, "Encoded page exceeds the effective byte ceiling"));
    }
    Ok(())
}

pub(super) fn storage_error(error: crate::storage::Error) -> Box<Failure> {
    let reason = match error {
        crate::storage::Error::Data(_) => Reason::ResourceLimit,
        crate::storage::Error::Corrupt(_) | crate::storage::Error::Decode(_) | crate::storage::Error::Json(_) => Reason::Corruption,
        crate::storage::Error::Incompatible(_) | crate::storage::Error::QuerySnapshotUnavailable => Reason::SnapshotUnavailable,
        _ => Reason::StorageUnavailable,
    };
    Failure::new(reason, "dokime.query.history", "Local query storage could not satisfy the requested cut")
}

/* === Tests === */
