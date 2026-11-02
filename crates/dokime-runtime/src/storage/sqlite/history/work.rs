/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Journal Work Revisions And Native Checkpoints Share One Versioned Projection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::{
    Message,
    storage::{SchemaKey, StoredRecord},
};
use rusqlite::{Connection, OptionalExtension, params};

// Local Imports
use super::{Error, advance, time_key};
use dokime_domain::{
    model::query::page::{CycleSummary, ExecutionSummary},
    state::records::work::{Cycle, Execution, Work},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Row {
    pub body: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn schema() -> SchemaKey {
    SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: harmos::Identifier::new("dokime.capture.work").unwrap(),
        version: 1.try_into().unwrap(),
    }
}

pub(super) fn execution(connection: &Connection, row: &mut ExecutionSummary) -> Result<(), Error> {
    if let Some(body) = latest(connection, "work_execution", &row.execution_id)? {
        let work: Execution = serde_json::from_slice(&body)?;
        super::execution_work(row, &work)?;
    }
    Ok(())
}

pub(super) fn cycle(connection: &Connection, row: &mut CycleSummary) -> Result<(), Error> {
    if let Some(body) = latest(connection, "work_cycle", &row.cycle_id)? {
        let work: Cycle = serde_json::from_slice(&body)?;
        row.status = super::cycle_status(work.status);
    }
    Ok(())
}

pub(crate) fn index(connection: &Connection, record: &StoredRecord) -> Result<(), Error> {
    if record.schema != schema() {
        return Ok(());
    }
    let row = Row::decode(&record.payload)?;
    let work: Work = serde_json::from_slice(&row.body)?;
    let position = super::super::data::position(record.position.sequence);
    for execution in work.executions {
        let revision = remember(connection, "work_execution", execution.id.as_str(), &position, &execution)?;
        let Some(body) = latest(connection, "execution", execution.id.as_str())? else { continue };
        let mut summary: ExecutionSummary = serde_json::from_slice(&body)?;
        super::execution_work(&mut summary, &execution)?;
        let body = serde_json::to_vec(&summary)?;
        let status = serde_json::to_string(&summary.status)?;
        let time = time_key(summary.started_at);
        connection.execute(
            "INSERT INTO query_history SELECT kind,id,?1,?2,cycle,template,?3,archived,?4,?5,?6 FROM query_history
             WHERE kind='execution' AND id=?7 ORDER BY revision DESC LIMIT 1",
            params![revision, position, status, summary.started_at.is_none(), time, body, execution.id.as_str()],
        )?;
    }
    for cycle in work.cycles {
        let revision = remember(connection, "work_cycle", cycle.id.as_str(), &position, &cycle)?;
        let Some(body) = latest(connection, "cycle", cycle.id.as_str())? else { continue };
        let mut summary: CycleSummary = serde_json::from_slice(&body)?;
        summary.status = super::cycle_status(cycle.status);
        let body = serde_json::to_vec(&summary)?;
        connection.execute(
            "INSERT INTO query_members SELECT cycle,?1,execution FROM query_members WHERE cycle=?2 AND position=(
                SELECT position FROM query_history WHERE kind='cycle' AND id=?2 ORDER BY revision DESC LIMIT 1)",
            params![position, cycle.id.as_str()],
        )?;
        connection.execute(
            "INSERT INTO query_history SELECT kind,id,?1,?2,cycle,template,status,archived,missing,time,?3 FROM query_history
             WHERE kind='cycle' AND id=?4 ORDER BY revision DESC LIMIT 1",
            params![revision, position, body, cycle.id.as_str()],
        )?;
    }
    Ok(())
}

/* === Private Functions === */

fn latest(connection: &Connection, kind: &str, id: &str) -> Result<Option<Vec<u8>>, Error> {
    Ok(connection
        .query_row("SELECT body FROM query_history WHERE kind=?1 AND id=?2 ORDER BY revision DESC LIMIT 1", params![kind, id], |row| {
            row.get(0)
        })
        .optional()?)
}

// Retain Lifecycle Intent So A Later Checkpoint Cannot Restore An Older Work State.
fn remember(connection: &Connection, kind: &str, id: &str, position: &str, work: &impl serde::Serialize) -> Result<i64, Error> {
    let revision = advance(connection)?;
    connection.execute(
        "INSERT INTO query_history VALUES(?1,?2,?3,?4,NULL,NULL,'',0,1,zeroblob(16),?5)",
        params![kind, id, revision, position, serde_json::to_vec(work)?],
    )?;
    Ok(revision)
}

/* === Tests === */
