/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Evidence Queries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use prost::Message;
use rusqlite::params;

// Local Imports
use crate::{Error, Evidence, Execution, ExecutionState, Result, Sample, Store, store};

use dokime_protocol::context::{EvidenceStatisticsRequest, EvidenceStatisticsResponse};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionCut {
    pub execution: Execution,
    pub state: ExecutionState,
    pub seal: Vec<u8>,
    pub frontier: u64,
}

#[derive(Clone, PartialEq, Message)]
pub struct StatisticsRead {
    #[prost(message, optional, tag = "1")]
    pub request: Option<EvidenceStatisticsRequest>,

    #[prost(uint64, repeated, tag = "2")]
    pub positions: Vec<u64>,

    #[prost(message, optional, tag = "3")]
    pub response: Option<EvidenceStatisticsResponse>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn cached_statistics(
        &self, execution: &str, request: &EvidenceStatisticsRequest, after: u64, through: u64,
    ) -> Result<Option<StatisticsRead>> {
        let key = (execution.into(), request.stream.clone(), request.field.clone(), after);
        let cache = self.statistics.lock().map_err(|_| Error::Invalid("Read Statistics: Restart After the Query Cache Failed".into()))?;
        Ok(cache.get(&key).filter(|(frontier, _)| *frontier == through).map(|(_, read)| read.clone()))
    }
    pub fn cache_statistics(
        &self, execution: &str, request: &EvidenceStatisticsRequest, after: u64, through: u64, read: StatisticsRead,
    ) -> Result<()> {
        let key = (execution.into(), request.stream.clone(), request.field.clone(), after);
        self.statistics
            .lock()
            .map_err(|_| Error::Invalid("Read Statistics: Restart After the Query Cache Failed".into()))?
            .insert(key, (through, read));
        Ok(())
    }
    pub fn cut(&self, execution: &str) -> Result<ExecutionCut> {
        let connection = self.lock()?;
        let (state, payload, seal): (i32, Vec<u8>, Vec<u8>) =
            connection.query_row("SELECT state,payload,seal FROM executions WHERE id=?1", [execution], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
        let state = match state {
            0 => ExecutionState::Open,
            1 => ExecutionState::Closed,
            2 => ExecutionState::Sealed,
            _ => return Err(Error::Incompatible("Invalid Execution State".into())),
        };
        if state != ExecutionState::Sealed && !seal.is_empty() {
            return Err(Error::Invalid("Inspect Execution: Seal Does Not Match Evidence; Restore a Valid Backup".into()));
        }
        if state == ExecutionState::Sealed {
            let mut verified =
                self.verified.lock().map_err(|_| Error::Invalid("Inspect Execution: Restart After Seal Verification Failed".into()))?;
            if verified.get(execution) != Some(&seal) {
                if store::digest(&connection, execution)? != seal {
                    return Err(Error::Invalid("Inspect Execution: Seal Does Not Match Evidence; Restore a Valid Backup".into()));
                }
                crate::telemetry::catalog::verify_files(self.database.parent().unwrap(), &connection, execution)?;
                verified.insert(execution.into(), seal.clone());
            }
        }
        let frontier = store::saved_position(&connection, execution)?;
        Ok(ExecutionCut {
            execution: Execution::decode(payload.as_slice())?,
            state,
            seal,
            frontier,
        })
    }
    pub fn sample_page(&self, execution: &str, stream: &str, after: u64, through: u64, limit: NonZeroUsize) -> Result<Vec<Sample>> {
        self.sample_window(crate::SampleWindow {
            execution,
            streams: &[stream.into()],
            after: &[(stream.into(), after)],
            through,
            start_ns: i64::MIN,
            end_ns: i64::MAX,
            latest: false,
            limit,
        })
    }
    pub fn evidence_page(&self, execution: &str, after: u64, through: u64, limit: NonZeroUsize) -> Result<Vec<Evidence>> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare("SELECT payload FROM evidence WHERE execution=?1 AND position>?2 AND position<=?3 ORDER BY position LIMIT ?4")?;
        let bytes = statement
            .query_map(params![execution, position(after)?, position(through)?, position(limit.get() as u64)?], |row| {
                row.get::<_, Vec<u8>>(0)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        bytes.iter().map(|bytes| Evidence::decode(bytes.as_slice()).map_err(Into::into)).collect()
    }
    pub fn step_range(&self, execution: &str, step: &str, frontier: u64, limit: NonZeroUsize) -> Result<(u64, u64)> {
        if step.is_empty() {
            return Ok((0, frontier));
        }
        let mut after = 0;
        let mut start = None;
        let mut end = None;
        loop {
            let page = self.evidence_page(execution, after, frontier, limit)?;
            if page.is_empty() {
                break;
            }
            for evidence in page {
                after = evidence.position;
                if evidence.step != step {
                    continue;
                }
                if evidence.kind == "step:start" {
                    start.get_or_insert(evidence.position);
                }
                if evidence.kind == "step:end" {
                    end = Some(evidence.position.saturating_sub(1));
                }
            }
        }
        let start = start.ok_or_else(|| Error::Invalid(format!("Read Step Statistics: Select a Started Step {step}")))?;
        Ok((start, end.unwrap_or(frontier)))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn position(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| Error::Invalid("Query Evidence: Choose Positions and a Page Size Within Range".into()))
}

/* === Tests === */
