/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Configured Arrow Query Scan Budgets

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// Third Party Imports

// Local Imports
use crate::{
    config::QueryConfig,
    storage::sqlite::{State, snapshots::Cut},
};
use dokime_domain::model::{
    query::{
        error::{EffectiveLimit, Failure, Reason},
        selection::Window,
    },
    telemetry::TelemetrySample,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct QueryRead {
    pub cut: Cut,
    pub execution: String,
    pub stream: String,
    pub versions: Vec<u32>,
    pub fields: Vec<String>,
    pub between: Option<Window>,
    pub after: u64,
}

pub(crate) struct QueryWork {
    pub config: QueryConfig,
    pub operation: &'static str,
    pub rows: u64,
    pub bytes: u64,
    deadline: std::time::Instant,
    cancelled: Arc<AtomicBool>,
}

struct Cancellation(Arc<AtomicBool>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl QueryWork {
    pub fn new(config: QueryConfig, operation: &'static str) -> Self {
        let deadline = std::time::Instant::now() + config.limits().active_time;
        Self {
            config,
            operation,
            rows: 0,
            bytes: 0,
            deadline,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn check(&self) -> Result<(), Box<Failure>> {
        if self.cancelled.load(Ordering::Acquire) || std::time::Instant::now() >= self.deadline {
            return Err(Failure::new(Reason::Deadline, self.operation, "Query exceeded its active-work deadline or was cancelled"));
        }
        Ok(())
    }

    pub fn charge(&mut self, rows: u64, bytes: u64) -> Result<(), Box<Failure>> {
        self.check()?;
        self.rows = self
            .rows
            .checked_add(rows)
            .ok_or_else(|| Failure::new(Reason::WorkLimit, self.operation, "Query scan row counter overflow"))?;
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or_else(|| Failure::new(Reason::WorkLimit, self.operation, "Query scan byte counter overflow"))?;
        for (used, maximum, name) in [
            (self.rows, self.config.limits().scanned_rows, "scanned_rows"),
            (self.bytes, self.config.limits().scanned_bytes, "scanned_bytes"),
        ] {
            if used > maximum {
                let mut failure = Failure::new(Reason::WorkLimit, self.operation, "Query scan budget exhausted; narrow the selection");
                failure.limit = Some(EffectiveLimit { name: name.into(), value: maximum });
                return Err(failure);
            }
        }
        Ok(())
    }
}

impl Drop for Cancellation {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

impl State {
    pub(crate) async fn query_telemetry_contracts(
        &self, at: u64, maximum: usize, operation: &str,
    ) -> Result<Vec<dokime_domain::model::telemetry::TelemetryContract>, Box<Failure>> {
        let owned_operation = operation.to_owned();
        self.read(move |connection| {
            let operation = owned_operation;
            Ok((|| {
                let mut statement = connection
                    .prepare("SELECT DISTINCT contract FROM streams WHERE first_position<=?1")
                    .map_err(|e| storage_error(e.into(), &operation))?;
                let mut rows = statement.query([super::super::position(at)]).map_err(|e| storage_error(e.into(), &operation))?;
                let mut contracts = Vec::new();
                let mut bytes = 0usize;
                while let Some(row) = rows.next().map_err(|e| storage_error(e.into(), &operation))? {
                    let body = row
                        .get_ref(0)
                        .and_then(|value| value.as_str().map_err(Into::into))
                        .map_err(|e| storage_error(e.into(), &operation))?;
                    bytes = bytes.saturating_add(body.len());
                    if bytes > maximum {
                        return Err(Failure::new(
                            Reason::ResourceLimit,
                            &operation,
                            "Captured telemetry contracts exceed the schema budget",
                        ));
                    }
                    let contract = serde_json::from_str(body)
                        .map_err(|_| Failure::new(Reason::Corruption, &operation, "Invalid retained telemetry contract"))?;
                    contracts.push(contract);
                }
                Ok(contracts)
            })())
        })
        .await
        .map_err(|e| storage_error(e, operation))?
    }

    pub(crate) async fn query_telemetry<T: Send + 'static>(
        &self, read: QueryRead, mut work: QueryWork, mut output: T,
        mut consume: impl FnMut(&mut T, u64, TelemetrySample) -> Result<Option<u64>, Box<Failure>> + Send + 'static,
    ) -> Result<(T, QueryWork, bool), Box<Failure>> {
        let operation = work.operation;
        let root = self.0.readers.database.parent().expect("storage root").to_owned();
        work.cancelled = Arc::new(AtomicBool::new(false));
        let _cancel = Cancellation(work.cancelled.clone());
        self.read(move |connection| {
            Ok((|| {
                if !crate::storage::sqlite::snapshots::available(connection, &read.cut).map_err(|e| storage_error(e, operation))? {
                    return Err(Failure::new(Reason::SnapshotUnavailable, operation, "The exact query cut is unavailable"));
                }
                let complete =
                    super::reader::query(connection, &root, &read, &mut work, |position, sample| consume(&mut output, position, sample))?;
                Ok((output, work, complete))
            })())
        })
        .await
        .map_err(|e| storage_error(e, operation))?
    }
}

/* === Public Functions === */

pub(crate) fn storage_error(error: crate::storage::Error, operation: &str) -> Box<Failure> {
    use crate::storage::Error;
    let reason = match error {
        Error::Data(_) => Reason::ResourceLimit,
        Error::Corrupt(_) | Error::Decode(_) | Error::Json(_) => Reason::Corruption,
        Error::Incompatible(_) | Error::QuerySnapshotUnavailable => Reason::SnapshotUnavailable,
        _ => Reason::StorageUnavailable,
    };
    Failure::new(reason, operation, "Local Arrow storage could not satisfy the query")
}

/* === Private Functions === */

/* === Tests === */
