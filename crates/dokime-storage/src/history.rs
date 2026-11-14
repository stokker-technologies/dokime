/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use rusqlite::{Connection, OptionalExtension, params};

// Local Imports
use crate::{Error, Execution, Result, Store};
use dokime_protocol::client::ExecutionsDeleteResponse;
use dokime_protocol::v1;
use harmos_protocol::v1::History;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn history_frontier(&self) -> Result<u64> {
        Ok(self.lock()?.query_row("SELECT COALESCE(MAX(sequence),0) FROM cycle_history", [], |row| row.get::<_, i64>(0))? as u64)
    }
    pub fn saved_cycles(&self, through: u64) -> Result<Vec<(u64, v1::CycleSummary)>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT h.sequence,h.payload FROM cycle_history h JOIN (SELECT cycle,MAX(sequence) AS sequence FROM cycle_history WHERE sequence<=?1 GROUP BY cycle) latest ON h.sequence=latest.sequence ORDER BY h.sequence")?;
        let rows = statement
            .query_map(
                [i64::try_from(through).map_err(|_| Error::Invalid("Browse History: Use a Returned Storage Frontier".into()))?],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        rows.into_iter().map(|(sequence, payload)| Ok((sequence as u64, v1::CycleSummary::decode(payload.as_slice())?))).collect()
    }
    pub fn sealed_cycle(&self, execution: &str) -> Result<(u64, v1::CycleSummary)> {
        let (sequence, payload): (i64, Vec<u8>) = self.lock()?.query_row(
            "SELECT sequence,payload FROM cycle_history WHERE execution=?1 ORDER BY sequence LIMIT 1",
            [execution],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok((sequence as u64, v1::CycleSummary::decode(payload.as_slice())?))
    }
    pub fn deleted_execution(&self, request: &str, execution: &str) -> Result<Option<ExecutionsDeleteResponse>> {
        let saved: Option<(String, Vec<u8>)> = self
            .lock()?
            .query_row("SELECT execution,response FROM deletions WHERE request=?1", [request], |row| Ok((row.get(0)?, row.get(1)?)))
            .optional()?;
        let Some((previous, payload)) = saved else {
            return Ok(None);
        };
        if previous != execution {
            return Err(Error::Conflict("Delete Execution: Use a New Request ID for a Different Execution".into()));
        }
        Ok(Some(ExecutionsDeleteResponse::decode(payload.as_slice())?))
    }
    pub fn delete_sealed_execution(
        &self, request: &str, execution: &str, expected: &[u8], response: &ExecutionsDeleteResponse,
    ) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let (state, seal): (i32, Vec<u8>) =
            transaction.query_row("SELECT state,seal FROM executions WHERE id=?1", [execution], |row| Ok((row.get(0)?, row.get(1)?)))?;
        if state != 2 || seal != expected || crate::store::digest(&transaction, execution)? != seal {
            return Err(Error::Invalid("Delete Execution: Inspect Its Current Seal Before Deleting".into()));
        }
        transaction
            .execute("DELETE FROM content_chunks WHERE object IN (SELECT id FROM content_objects WHERE execution=?1)", [execution])?;
        transaction.execute("DELETE FROM file_uploads WHERE id IN (SELECT id FROM content_objects WHERE execution=?1)", [execution])?;
        transaction.execute("DELETE FROM content_objects WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM notes WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM administration WHERE id LIKE ?1", [format!("plot-settings:{execution}/%")])?;
        transaction.execute("DELETE FROM record_fields WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM reports WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM read_positions WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM resource_requests WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM resource_values WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM attempts WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM evidence WHERE execution=?1", [execution])?;
        transaction.execute("INSERT INTO telemetry_garbage SELECT id FROM telemetry_files WHERE execution=?1", [execution])?;
        transaction
            .execute("DELETE FROM telemetry_previews WHERE file IN (SELECT id FROM telemetry_files WHERE execution=?1)", [execution])?;
        transaction
            .execute("DELETE FROM telemetry_summaries WHERE file IN (SELECT id FROM telemetry_files WHERE execution=?1)", [execution])?;
        transaction
            .execute("DELETE FROM telemetry_checkpoints WHERE file IN (SELECT id FROM telemetry_files WHERE execution=?1)", [execution])?;
        transaction
            .execute("DELETE FROM telemetry_batches WHERE file IN (SELECT id FROM telemetry_files WHERE execution=?1)", [execution])?;
        transaction.execute("DELETE FROM telemetry_files WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM streams WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM bindings WHERE execution=?1", [execution])?;
        transaction.execute("DELETE FROM executions WHERE id=?1", [execution])?;
        transaction.execute("INSERT INTO deletions VALUES (?1,?2,?3,?4)", params![request, execution, seal, response.encode_to_vec()])?;
        // A Global Evidence Cut Cannot Silently Lose a Deleted History.
        transaction.execute("DELETE FROM evidence_reads", [])?;
        let history = History {
            store: self.identity.to_string(),
            epoch: uuid::Uuid::now_v7().to_string(),
        };
        transaction.execute("UPDATE administration SET payload=?1 WHERE id='evidence-read-history'", [history.encode_to_vec()])?;
        transaction.commit()?;
        crate::telemetry::catalog::collect_garbage(self.database.parent().unwrap(), &connection)?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
}

/* === Public Functions === */

pub(crate) fn capture_cycle(connection: &Connection, id: &str, execution: &str) -> Result<()> {
    let completed: i64 = connection.query_row("SELECT COUNT(*) FROM executions WHERE cycle=?1 AND state=2", [id], |row| row.get(0))?;
    if completed == 0 {
        return Ok(());
    }
    let payload: Option<Vec<u8>> = connection.query_row("SELECT payload FROM cycles WHERE id=?1", [id], |row| row.get(0)).optional()?;
    let mut cycle = payload.map(|payload| v1::CycleSummary::decode(payload.as_slice())).transpose()?.unwrap_or_else(|| v1::CycleSummary {
        cycle_id: id.into(),
        ..Default::default()
    });
    let total: i64 = connection.query_row("SELECT COUNT(*) FROM executions WHERE cycle=?1", [id], |row| row.get(0))?;
    cycle.execution_count = total as u64;
    cycle.case_count = connection.query_row("SELECT COUNT(DISTINCT COALESCE(NULLIF(b.case_id,''),e.id)) FROM executions e LEFT JOIN bindings b ON b.execution=e.id WHERE e.cycle=?1", [id], |row| row.get::<_,i64>(0))? as u64;
    cycle.phase = if cycle.sealed { v1::CyclePhase::Sealed } else { v1::CyclePhase::Active } as i32;
    cycle.status = if total == completed { v1::CycleStatus::Completed } else { v1::CycleStatus::Running } as i32;
    let mut statement = connection.prepare("SELECT payload FROM executions WHERE cycle=?1")?;
    let payloads = statement.query_map([id], |row| row.get::<_, Vec<u8>>(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
    let executions: Vec<_> = payloads.iter().map(|payload| Execution::decode(payload.as_slice())).collect::<std::result::Result<_, _>>()?;
    let counts = crate::OutcomeCounts::from_outcomes(executions.iter().filter_map(|execution| execution.outcome.as_ref()));
    cycle.result = Some(v1::ResultSummary {
        status: counts.verdict.map_or(v1::ResultStatus::Pending as i32, |status| status as i32),
        not_evaluated: counts.not_evaluated as u32,
        declared: counts.declared as u32,
        ..Default::default()
    });
    cycle.observed_cursor = None;
    connection
        .execute("INSERT INTO cycle_history(cycle,execution,payload) VALUES (?1,?2,?3)", params![id, execution, cycle.encode_to_vec()])?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
