/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Versioned Local Catalog Projections At Durable Cuts

/* === Modules === */

pub(crate) mod work;

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::{Message, storage::ObjectEntry};
use rusqlite::{Connection, params, types::Value};
use serde::{Deserialize, Serialize};

// Local Imports
use super::{Error, State};
use dokime_domain::model::{
    execution::{
        result,
        wire::{self, ArchiveScope},
    },
    history::wire::ExecutionAvailability,
    identity::wire::Timestamp,
    query::{
        page::{CycleSummary, ExecutionSummary},
        selection::{CycleSelection, ExecutionSelection, Window},
    },
};

/* === Constants === */

pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS query_history_head (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1), revision INTEGER NOT NULL
);
INSERT OR IGNORE INTO query_history_head VALUES(1,0);
CREATE TABLE IF NOT EXISTS query_history (
 kind TEXT NOT NULL, id TEXT NOT NULL, revision INTEGER NOT NULL,
 position TEXT NOT NULL, cycle TEXT, template TEXT, status TEXT NOT NULL,
 archived INTEGER NOT NULL, missing INTEGER NOT NULL, time BLOB NOT NULL,
 body BLOB NOT NULL, PRIMARY KEY(kind,id,revision)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS query_history_order ON query_history(kind,archived,missing,time,id,revision);
CREATE INDEX IF NOT EXISTS query_history_all ON query_history(kind,missing,time,id,revision);
CREATE INDEX IF NOT EXISTS query_history_template ON query_history(kind,template,status,archived,missing,time,id,revision);
CREATE INDEX IF NOT EXISTS query_history_template_only ON query_history(kind,template,archived,missing,time,id,revision);
CREATE INDEX IF NOT EXISTS query_history_status ON query_history(kind,status,archived,missing,time,id,revision);
CREATE INDEX IF NOT EXISTS query_history_cycle ON query_history(kind,cycle,archived,missing,time,id,revision);
CREATE TABLE IF NOT EXISTS query_members (
 cycle TEXT NOT NULL, position TEXT NOT NULL, execution TEXT NOT NULL,
 PRIMARY KEY(cycle,position,execution)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS query_members_execution ON query_members(execution,cycle,position);
CREATE TABLE IF NOT EXISTS query_archives (
 kind TEXT NOT NULL, id TEXT NOT NULL, PRIMARY KEY(kind,id)
) WITHOUT ROWID;
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub missing: bool,
    pub time: Vec<u8>,
    pub id: String,
}

#[derive(Clone)]
pub(crate) struct Read {
    pub revision: i64,
    pub cut: Option<super::snapshots::Cut>,
    pub after: Option<Key>,
    pub rows: u32,
    pub bytes: u64,
}

pub(crate) struct Page<T> {
    pub rows: Vec<T>,
    pub next: Option<Key>,
}

pub(super) struct Plan {
    pub sql: String,
    pub arguments: Vec<Value>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn history_revision(&self) -> Result<i64, Error> {
        self.read(|connection| connection.query_row("SELECT revision FROM query_history_head", [], |row| row.get(0))).await
    }

    pub(crate) async fn history_executions(&self, selection: ExecutionSelection, read: Read) -> Result<Page<ExecutionSummary>, Error> {
        self.read(move |connection| Ok(executions(connection, &selection, &read))).await?
    }

    pub(crate) async fn history_cycles(&self, selection: CycleSelection, read: Read) -> Result<Page<CycleSummary>, Error> {
        self.read(move |connection| Ok(cycles(connection, &selection, &read))).await?
    }

    // The Journal Owns Archive Intent. Reconcile Before Serving After Recovery And
    // Before A Successful Archive Mutation Returns; One SQL Commit Publishes It.
    pub(crate) async fn history_archives(&self, cycles: Vec<String>, executions: Vec<String>) -> Result<(), Error> {
        self.call(move |connection| Ok((|| {
            let transaction = connection.transaction()?;
            transaction.execute_batch("CREATE TEMP TABLE IF NOT EXISTS query_archive_input(kind TEXT,id TEXT,PRIMARY KEY(kind,id)) WITHOUT ROWID; DELETE FROM query_archive_input;")?;
            for (kind, ids) in [("cycle", cycles), ("execution", executions)] {
                for id in ids {
                    transaction.execute("INSERT INTO query_archive_input VALUES(?1,?2)", params![kind,id])?;
                }
            }
            let changed: bool = transaction.query_row("SELECT EXISTS(SELECT * FROM query_archive_input EXCEPT SELECT * FROM query_archives) OR EXISTS(SELECT * FROM query_archives EXCEPT SELECT * FROM query_archive_input)", [], |row|row.get(0))?;
            if changed {
                let revision = advance(&transaction)?;
                transaction.execute_batch("DELETE FROM query_archives; INSERT INTO query_archives SELECT * FROM query_archive_input;")?;
                transaction.execute(
                    "INSERT INTO query_history SELECT h.kind,h.id,?1,h.position,h.cycle,h.template,h.status,
                     EXISTS(SELECT 1 FROM query_archives a WHERE (a.kind=h.kind AND a.id=h.id) OR (a.kind='cycle' AND a.id=h.cycle)),
                     h.missing,h.time,h.body FROM query_history h WHERE NOT EXISTS(
                         SELECT 1 FROM query_history n WHERE n.kind=h.kind AND n.id=h.id AND n.revision>h.revision)
                     AND h.archived!=EXISTS(SELECT 1 FROM query_archives a WHERE (a.kind=h.kind AND a.id=h.id) OR (a.kind='cycle' AND a.id=h.cycle))",
                    [revision],
                )?;
            }
            super::commit(transaction)?;
            Ok(())
        })())).await?
    }
}

/* === Public Functions === */

pub(super) fn check_format(connection: &Connection) -> Result<(), Error> {
    for sql in [
        "SELECT revision FROM query_history_head LIMIT 0",
        "SELECT kind,id,revision,position,cycle,template,status,archived,missing,time,body FROM query_history LIMIT 0",
        "SELECT kind,id FROM query_archives LIMIT 0",
        "SELECT cycle,position,execution FROM query_members LIMIT 0",
    ] {
        connection.prepare(sql).map_err(|_| Error::Incompatible("local catalog lacks versioned history projections".into()))?;
    }
    Ok(())
}

pub(super) fn index(connection: &Connection, object: &ObjectEntry) -> Result<(), Error> {
    if matches!(object.value.schema.id.as_str(), "dokime.object.execution" | "dokime.object.cycle")
        && object.value.schema.version.get() != 1
    {
        return Err(Error::Incompatible("unsupported history object version".into()));
    }
    let position = super::data::position(object.position.sequence);
    let mut members = Vec::new();
    let (kind, id, cycle, template, status, time, body) = match object.value.schema.id.as_str() {
        "dokime.object.execution" => {
            let native = crate::storage::objects::execution::Execution::decode(&object.value.payload)?;
            let execution = native.into_execution()?;
            if execution.test_execution_id.as_str() != object.value.item.as_str() {
                return Err(Error::Corrupt("history execution identity differs"));
            }
            let mut summary = execution_summary(&execution)?;
            work::execution(connection, &mut summary)?;
            (
                "execution",
                summary.execution_id.clone(),
                summary.cycle_id.clone(),
                summary.template_id.clone(),
                serde_json::to_string(&summary.status)?,
                summary.started_at,
                serde_json::to_vec(&summary)?,
            )
        }
        "dokime.object.cycle" => {
            let native = crate::storage::objects::cycle::Cycle::decode(&object.value.payload)?;
            let cycle = native.restore()?;
            if cycle.test_cycle_id.as_str() != object.value.item.as_str() {
                return Err(Error::Corrupt("history cycle identity differs"));
            }
            members = cycle.execution_ids.iter().map(ToString::to_string).collect();
            let mut summary = CycleSummary {
                cycle_id: cycle.test_cycle_id.to_string(),
                label: Some(cycle.metadata.label.clone()),
                phase: cycle.phase,
                created_at: Some(timestamp(&cycle.created_at)?),
                updated_at: Some(timestamp(&cycle.updated_at)?),
                execution_count: cycle.execution_ids.len() as u64,
                archived: false,
                status: cycle_status(cycle.lifecycle),
                running_count: 0,
            };
            work::cycle(connection, &mut summary)?;
            (
                "cycle",
                summary.cycle_id.clone(),
                None,
                None,
                serde_json::to_string(&summary.phase)?,
                summary.created_at,
                serde_json::to_vec(&summary)?,
            )
        }
        _ => return Ok(()),
    };
    let revision = advance(connection)?;
    for member in members {
        connection.execute("INSERT INTO query_members VALUES(?1,?2,?3)", params![id, position, member])?;
    }
    connection.execute(
        "INSERT INTO query_history VALUES(?1,?2,?3,?4,?5,?6,?7,
        EXISTS(SELECT 1 FROM query_archives WHERE (kind=?1 AND id=?2) OR (kind='cycle' AND id=?5)),?8,?9,?10)",
        params![kind, id, revision, position, cycle, template, status, time.is_none(), time_key(time), body],
    )?;
    Ok(())
}

pub(crate) fn executions(connection: &Connection, selection: &ExecutionSelection, read: &Read) -> Result<Page<ExecutionSummary>, Error> {
    let plan = execution_plan(selection, read)?;
    page(connection, plan, read, |body, archived| {
        let mut row: ExecutionSummary = serde_json::from_slice(body)?;
        row.archived = archived;
        Ok(row)
    })
}

pub(crate) fn cycles(connection: &Connection, selection: &CycleSelection, read: &Read) -> Result<Page<CycleSummary>, Error> {
    let mut plan = plan("cycle", selection.archive, read);
    if selection.phases.len() == 1 && selection.archive != Some(ArchiveScope::All) {
        select_index(&mut plan, "query_history_status");
    }
    alternatives(&mut plan, "id", &selection.cycle_ids);
    alternatives(&mut plan, "status", &selection.phases.iter().map(serde_json::to_string).collect::<Result<Vec<_>, _>>()?);
    if !selection.statuses.is_empty() {
        plan.sql
            .push_str(&format!(" AND json_quote(json_extract(h.body,'$.status')) IN ({})", vec!["?"; selection.statuses.len()].join(",")));
        plan.arguments
            .extend(selection.statuses.iter().map(serde_json::to_string).collect::<Result<Vec<_>, _>>()?.into_iter().map(Value::Text));
    }
    window(&mut plan, selection.created_at.as_ref());
    finish(&mut plan, read);
    page(connection, plan, read, |body, archived| {
        let mut row: CycleSummary = serde_json::from_slice(body)?;
        row.archived = archived;
        // Count Latest Member Revisions At The Same Catalog And Metadata Cut.
        let count_read = Read { after: None, ..read.clone() };
        let mut count = self::plan("execution", Some(ArchiveScope::All), &count_read);
        select_index(&mut count, "query_history_cycle");
        alternatives(&mut count, "cycle", std::slice::from_ref(&row.cycle_id));
        alternatives(&mut count, "status", &[serde_json::to_string(&wire::ExecutionStatus::Running)?]);
        count.sql = count.sql.replacen("SELECT h.body,h.archived,h.missing,h.time,h.id", "SELECT COUNT(*)", 1);
        let running: i64 = connection.query_row(&count.sql, rusqlite::params_from_iter(count.arguments), |row| row.get(0))?;
        row.running_count = running as u64;
        Ok(row)
    })
}

pub(super) fn execution_plan(selection: &ExecutionSelection, read: &Read) -> Result<Plan, Error> {
    let mut plan = plan("execution", selection.archive, read);
    if selection.archive != Some(ArchiveScope::All) {
        let index = if selection.template_ids.len() == 1 && selection.statuses.len() == 1 {
            "query_history_template"
        } else if selection.template_ids.len() == 1 {
            "query_history_template_only"
        } else if selection.cycle_ids.len() == 1 {
            "query_history_cycle"
        } else if selection.statuses.len() == 1 {
            "query_history_status"
        } else {
            "query_history_order"
        };
        select_index(&mut plan, index);
    }
    alternatives(&mut plan, "template", &selection.template_ids);
    alternatives(&mut plan, "cycle", &selection.cycle_ids);
    alternatives(&mut plan, "status", &selection.statuses.iter().map(serde_json::to_string).collect::<Result<Vec<_>, _>>()?);
    window(&mut plan, selection.started_at.as_ref());
    finish(&mut plan, read);
    Ok(plan)
}

/* === Private Functions === */

fn advance(connection: &Connection) -> Result<i64, Error> {
    Ok(connection.query_row(
        "UPDATE query_history_head SET revision=revision+1 WHERE revision<9223372036854775807 RETURNING revision",
        [],
        |row| row.get(0),
    )?)
}

pub(crate) fn timestamp(time: &dokime_domain::model::time::Timestamp) -> Result<Timestamp, Error> {
    let time = time.to_datetime()?;
    Ok(Timestamp {
        seconds: time.unix_timestamp(),
        nanos: time.nanosecond() as i32,
    })
}

fn time_key(time: Option<Timestamp>) -> Vec<u8> {
    time.map_or_else(
        || vec![0; 16],
        |time| ((i128::from(time.seconds) * 1_000_000_000 + i128::from(time.nanos)) ^ i128::MIN).to_be_bytes().to_vec(),
    )
}

fn execution_summary(execution: &result::TestExecution) -> Result<ExecutionSummary, Error> {
    let status = execution_status(&execution.status);
    let result = execution.result.as_ref().map(result_summary);
    Ok(ExecutionSummary {
        execution_id: execution.test_execution_id.to_string(),
        cycle_id: Some(execution.test_cycle_id.to_string()),
        case_id: execution.test_case_id.to_string(),
        template_id: Some(execution.snapshot.case().test_case_template_id.to_string()),
        label: Some(execution.snapshot.case().metadata.label.clone()),
        status,
        started_at: execution.started_at.as_ref().map(timestamp).transpose()?,
        finished_at: execution.finished_at.as_ref().map(timestamp).transpose()?,
        result,
        checkpoint_available: execution.saved.is_some(),
        archived: false,
        availability: availability(execution.saved.is_some(), status),
    })
}

fn execution_status(status: &result::ExecutionStatus) -> wire::ExecutionStatus {
    match status {
        result::ExecutionStatus::Invalidated => wire::ExecutionStatus::Invalidated,
        result::ExecutionStatus::Pending => wire::ExecutionStatus::Pending,
        result::ExecutionStatus::Running => wire::ExecutionStatus::Running,
        result::ExecutionStatus::Passed => wire::ExecutionStatus::Passed,
        result::ExecutionStatus::Failed => wire::ExecutionStatus::Failed,
        result::ExecutionStatus::Blocked => wire::ExecutionStatus::Blocked,
        result::ExecutionStatus::Cancelled => wire::ExecutionStatus::Cancelled,
        result::ExecutionStatus::Interrupted => wire::ExecutionStatus::Interrupted,
        result::ExecutionStatus::Skipped => wire::ExecutionStatus::Skipped,
    }
}

fn result_summary(result: &result::ResultSummary) -> wire::ResultSummary {
    wire::ResultSummary {
        status: match result.status {
            result::ResultStatus::Passed => wire::ResultStatus::Passed,
            result::ResultStatus::Failed => wire::ResultStatus::Failed,
            result::ResultStatus::Blocked => wire::ResultStatus::Blocked,
            result::ResultStatus::Pending => wire::ResultStatus::Pending,
            result::ResultStatus::Cancelled => wire::ResultStatus::Cancelled,
        },
        passed: result.passed,
        failed: result.failed,
        blocked: result.blocked,
        skipped: result.skipped,
    }
}

fn cycle_status(status: result::CycleStatus) -> wire::CycleStatus {
    match status {
        result::CycleStatus::Ready => wire::CycleStatus::Ready,
        result::CycleStatus::Running => wire::CycleStatus::Running,
        result::CycleStatus::Paused => wire::CycleStatus::Paused,
        result::CycleStatus::Stopping => wire::CycleStatus::Stopping,
        result::CycleStatus::Completed => wire::CycleStatus::Completed,
        result::CycleStatus::Failed => wire::CycleStatus::Failed,
        result::CycleStatus::Blocked => wire::CycleStatus::Blocked,
        result::CycleStatus::Stopped => wire::CycleStatus::Stopped,
    }
}

fn availability(saved: bool, status: wire::ExecutionStatus) -> ExecutionAvailability {
    if saved {
        ExecutionAvailability::Stored
    } else if status == wire::ExecutionStatus::Running {
        ExecutionAvailability::Live
    } else if status == wire::ExecutionStatus::Interrupted {
        ExecutionAvailability::Interrupted
    } else {
        ExecutionAvailability::Uncheckpointed
    }
}

fn execution_work(row: &mut ExecutionSummary, work: &dokime_domain::state::records::work::Execution) -> Result<(), Error> {
    row.status = execution_status(&work.status);
    row.started_at = work.started_at.as_ref().map(timestamp).transpose()?;
    row.finished_at = work.finished_at.as_ref().map(timestamp).transpose()?;
    row.result = work.result.as_ref().map(result_summary);
    row.availability = availability(row.checkpoint_available, row.status);
    Ok(())
}

fn plan(kind: &str, archive: Option<ArchiveScope>, read: &Read) -> Plan {
    let index = if archive == Some(ArchiveScope::All) { "query_history_all" } else { "query_history_order" };
    let mut plan = Plan { sql: "SELECT h.body,h.archived,h.missing,h.time,h.id FROM query_history h WHERE h.kind=? AND h.revision<=? AND NOT EXISTS(SELECT 1 FROM query_history n WHERE n.kind=h.kind AND n.id=h.id AND n.revision>h.revision AND n.revision<=?)".into(), arguments: vec![kind.to_owned().into(),read.revision.into(),read.revision.into()] };
    if let Some(cut) = &read.cut {
        plan.sql = plan.sql.replace("n.revision<=?)", "n.revision<=? AND n.position<=?)");
        plan.arguments.push(super::data::position(cut.metadata).into());
        plan.sql.push_str(" AND h.position<=?");
        plan.arguments.push(super::data::position(cut.metadata).into());
    }
    select_index(&mut plan, index);
    match archive.unwrap_or(ArchiveScope::Unarchived) {
        ArchiveScope::All => {}
        archive => {
            plan.sql.push_str(" AND h.archived=?");
            plan.arguments.push(i64::from(archive == ArchiveScope::Archived).into());
        }
    }
    if let Some(key) = &read.after {
        plan.sql.push_str(" AND (h.missing,h.time,h.id)>(?,?,?)");
        plan.arguments.extend([i64::from(key.missing).into(), key.time.clone().into(), key.id.clone().into()]);
    }
    plan
}

fn select_index(plan: &mut Plan, index: &str) {
    let start = plan.sql.find(" FROM query_history h").expect("owned history plan");
    let end = plan.sql[start..].find(" WHERE ").expect("owned history predicate") + start;
    plan.sql.replace_range(start..end, &format!(" FROM query_history h INDEXED BY {index}"));
}

fn alternatives(plan: &mut Plan, column: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    plan.sql.push_str(&format!(" AND h.{column} IN ({})", vec!["?"; values.len()].join(",")));
    plan.arguments.extend(values.iter().cloned().map(Value::Text));
}

fn window(plan: &mut Plan, window: Option<&Window>) {
    if let Some(window) = window {
        for (op, time) in [(">=", window.start), ("<", window.end)] {
            if let Some(time) = time {
                plan.sql.push_str(&format!(" AND h.missing=0 AND h.time{op}?"));
                plan.arguments.push(time_key(Some(time)).into());
            }
        }
    }
}

fn finish(plan: &mut Plan, read: &Read) {
    plan.sql.push_str(" ORDER BY h.missing,h.time,h.id LIMIT ?");
    plan.arguments.push((i64::from(read.rows) + 1).into());
}

fn page<T>(connection: &Connection, plan: Plan, read: &Read, decode: impl Fn(&[u8], bool) -> Result<T, Error>) -> Result<Page<T>, Error> {
    if let Some(cut) = &read.cut
        && !super::snapshots::available(connection, cut)?
    {
        return Err(Error::QuerySnapshotUnavailable);
    }
    let mut statement = connection.prepare(&plan.sql)?;
    let mut selected = statement.query(rusqlite::params_from_iter(plan.arguments))?;
    let mut rows = Vec::new();
    let mut used = 0u64;
    let mut last = None;
    while let Some(row) = selected.next()? {
        let body = row.get_ref(0)?.as_blob().map_err(|_| Error::Corrupt("history summary is not bytes"))?;
        if rows.len() == read.rows as usize || body.len() as u64 > read.bytes.saturating_sub(used) {
            if rows.is_empty() {
                return Err(Error::Data("history row exceeds page budget"));
            }
            return Ok(Page { rows, next: last });
        }
        used += body.len() as u64;
        rows.push(decode(body, row.get(1)?)?);
        last = Some(Key {
            missing: row.get(2)?,
            time: row.get(3)?,
            id: row.get(4)?,
        });
    }
    Ok(Page { rows, next: None })
}

/* === Tests === */
