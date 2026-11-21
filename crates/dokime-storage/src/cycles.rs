/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycle and Execution Presentation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::result;

// Third Party Imports
use prost::Message;
use prost_types::Timestamp;
use rusqlite::{OptionalExtension, params};

// Local Imports
use crate::ExecutionState;
use crate::{Error, Evidence, Execution, Result, Store};
use dokime_protocol::host::CycleSummary;
use dokime_protocol::host::{CyclePhase, ExecutionOutcome, UnevaluatedOutcome, execution_outcome};
use dokime_protocol::host::{CyclesSetRequest, CyclesSetResponse};
use dokime_protocol::model::Termination;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Message)]
struct CycleRequest {
    #[prost(bytes, tag = "1")]
    fingerprint: Vec<u8>,

    #[prost(message, optional, tag = "2")]
    response: Option<CyclesSetResponse>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn seal_unstarted(&self, execution: &str, at: Timestamp) -> Result<()> {
        let cut = self.cut(execution)?;
        if cut.state != ExecutionState::Open || cut.frontier != 0 {
            return Err(Error::Invalid(format!("Seal Execution {execution} as Not Run: Select Evidence That Has Never Started")));
        }
        self.close(
            execution,
            Termination::NotRun,
            ExecutionOutcome {
                attribution: Some(execution_outcome::Attribution::NotEvaluated(UnevaluatedOutcome {
                    termination: Termination::NotRun as i32,
                    reason: "Cycle Sealed Before Execution Started".into(),
                })),
            },
        )?;
        self.seal(execution, at)
    }
    pub fn cycle_request(&self, request: &CyclesSetRequest) -> Result<Option<CyclesSetResponse>> {
        if request.request_id.is_empty() {
            return Err(Error::Invalid("Set Cycle: Supply a Request ID".into()));
        }
        let key = format!("cycle/set/{}/{}", request.name, request.request_id);
        let saved = self.administration::<CycleRequest>(&key)?;
        saved
            .map(|saved| {
                if saved.fingerprint != request.encode_to_vec() {
                    return Err(Error::Conflict("Set Cycle: Use a New Request ID for Changed Content".into()));
                }
                saved.response.ok_or(Error::Data("Set Cycle: Read Its Saved Mutation Response"))
            })
            .transpose()
    }
    pub fn save_cycle_request(
        &self, request: &CyclesSetRequest, expected: Option<&CycleSummary>, cycle: &CycleSummary, response: &CyclesSetResponse,
    ) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let current: Option<Vec<u8>> =
            transaction.query_row("SELECT payload FROM cycles WHERE id=?1", [&request.name], |row| row.get(0)).optional()?;
        if current != expected.map(Message::encode_to_vec) {
            return Err(Error::Conflict("Set Cycle: Read the Changed Cycle Before Replacing It".into()));
        }
        transaction.execute(
            "INSERT INTO cycles VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
            params![cycle.cycle_id, cycle.encode_to_vec()],
        )?;
        crate::history::capture_cycle(&transaction, &cycle.cycle_id, "")?;
        let saved = CycleRequest {
            fingerprint: request.encode_to_vec(),
            response: Some(response.clone()),
        };
        transaction.execute(
            "INSERT INTO administration VALUES (?1,?2)",
            params![format!("cycle/set/{}/{}", request.name, request.request_id), saved.encode_to_vec()],
        )?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn execution_times(&self, execution: &str) -> Result<(Option<i64>, Option<i64>)> {
        let connection = self.lock()?;
        let first: Option<Vec<u8>> = connection
            .query_row("SELECT payload FROM evidence WHERE execution=?1 ORDER BY position LIMIT 1", [execution], |row| row.get(0))
            .optional()?;
        let last: Option<Vec<u8>> = connection
            .query_row("SELECT payload FROM evidence WHERE execution=?1 ORDER BY position DESC LIMIT 1", [execution], |row| row.get(0))
            .optional()?;
        Ok((
            first.map(|bytes| Evidence::decode(bytes.as_slice()).map(|entry| entry.receipt_ns)).transpose()?,
            last.map(|bytes| Evidence::decode(bytes.as_slice()).map(|entry| entry.receipt_ns)).transpose()?,
        ))
    }
    pub fn sample_count(&self) -> Result<u64> {
        Ok(self.lock()?.query_row("SELECT COALESCE(SUM(rows),0) FROM telemetry_batches", [], |row| row.get::<_, i64>(0))? as u64)
    }
    pub fn save_cycle(&self, cycle: &CycleSummary) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "INSERT INTO cycles VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
            params![cycle.cycle_id, cycle.encode_to_vec()],
        )?;
        crate::history::capture_cycle(&transaction, &cycle.cycle_id, "")?;
        transaction.commit()?;
        Ok(())
    }
    pub fn client_cycle(&self, id: &str) -> Result<Option<CycleSummary>> {
        let payload: Option<Vec<u8>> =
            self.lock()?.query_row("SELECT payload FROM cycles WHERE id=?1", [id], |row| row.get(0)).optional()?;
        Ok(payload.map(|payload| CycleSummary::decode(payload.as_slice())).transpose()?)
    }
    pub fn client_cycles(&self) -> Result<Vec<CycleSummary>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT payload FROM cycles ORDER BY rowid")?;
        let payloads = statement.query_map([], |row| row.get::<_, Vec<u8>>(0))?.collect::<result::Result<Vec<_>, _>>()?;
        Ok(payloads.iter().map(|payload| CycleSummary::decode(payload.as_slice())).collect::<result::Result<_, _>>()?)
    }
    pub fn bind_executions(&self, cycle: &str, executions: &[(Execution, String, String)]) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let payload: Vec<u8> = transaction.query_row("SELECT payload FROM cycles WHERE id=?1", [cycle], |row| row.get(0))?;
        let summary = CycleSummary::decode(payload.as_slice())?;
        if summary.sealed || summary.phase == CyclePhase::Sealing as i32 {
            return Err(Error::Invalid("Append Executions: Create a New Cycle; This Cycle Is Sealed".into()));
        }
        for (execution, case, label) in executions {
            transaction.execute(
                "INSERT INTO executions VALUES (?1,?2,0,?3,X'')",
                params![execution.identifier, cycle, execution.encode_to_vec()],
            )?;
            transaction.execute("INSERT INTO bindings VALUES (?1,?2,?3,0)", params![execution.identifier, case, label])?;
        }
        transaction.commit()?;
        Ok(())
    }
    pub fn execution_presentation(&self, execution: &str) -> Result<(String, String, bool)> {
        Ok(self
            .lock()?
            .query_row("SELECT case_id,label,archived FROM bindings WHERE execution=?1", [execution], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .optional()?
            .unwrap_or_default())
    }
    pub fn set_execution_presentation(&self, execution: &str, label: Option<&str>, archived: Option<bool>) -> Result<()> {
        self.read(execution)?;
        let (case, previous_label, previous_archived) = self.execution_presentation(execution)?;
        self.lock()?.execute("INSERT INTO bindings VALUES (?1,?2,?3,?4) ON CONFLICT(execution) DO UPDATE SET label=excluded.label,archived=excluded.archived", params![execution,case,label.unwrap_or(&previous_label),archived.unwrap_or(previous_archived)])?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
