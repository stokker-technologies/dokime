/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Versioned Local Catalog Projections at Durable Cuts

/* === Modules === */

pub(crate) mod work;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::{MemorySize, Message, storage::ObjectEntry};
use rusqlite::{Connection, OptionalExtension, params, types::Value};
use serde::{Deserialize, Serialize};

// Local Imports
use super::{Error, State};
use dokime_domain::model::{
    execution::{
        result,
        wire::{self, ArchiveScope},
    },
    history::wire::ExecutionAvailability,
    identity::{SuiteId, wire::Timestamp},
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
CREATE TABLE IF NOT EXISTS query_execution_outputs (
 execution TEXT NOT NULL, position TEXT NOT NULL REFERENCES storage_data_entries(position) ON DELETE CASCADE,
 path TEXT NOT NULL, body BLOB NOT NULL, PRIMARY KEY(execution,position,path)
) WITHOUT ROWID;
CREATE INDEX IF NOT EXISTS query_execution_outputs_position ON query_execution_outputs(position);
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

#[derive(Serialize, Deserialize)]
struct ExecutionProjection {
    #[serde(flatten)]
    summary: ExecutionSummary,

    suite_id: Option<SuiteId>,
    case: Option<crate::runtime::query::cases::CaseHeader>,
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
    pub(crate) async fn execution_output(
        &self, at: harmos::storage::StorageCursor, execution: String, path: String,
    ) -> Result<Option<dokime_domain::model::execution::saved::ExecutionOutput>, Error> {
        self.read(move |connection| {
            Ok((|| {
                let saved = super::data::saved(connection)?;
                saved.history.check(&at.history)?;
                let row: Option<(String, Vec<u8>)> = connection
                    .query_row(
                        "SELECT o.position,o.body FROM query_execution_outputs o WHERE o.execution=?1 AND o.path=?2
                    AND o.position=(SELECT e.position FROM storage_data_entries e WHERE e.kind='object'
                        AND e.schema_id='dokime.object.execution' AND e.item=?1 AND e.position<=?3
                        AND (e.retired_at IS NULL OR e.retired_at>?4) ORDER BY e.position DESC LIMIT 1)",
                        params![execution, path, super::data::position(at.sequence), super::data::position(saved.retention_generation)],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()?;
                let Some((position, body)) = row else { return Ok(None) };
                let (dependencies, output): (harmos::storage::StorageCursor, dokime_domain::model::execution::saved::ExecutionOutput) =
                    serde_json::from_slice(&body)?;
                if dependencies.history != at.history || super::data::sequence(&position)? <= dependencies.sequence {
                    return Err(Error::Invalid("saved output seal admission or evidence cut differs"));
                }
                if output.relative_path != path {
                    return Err(Error::Corrupt("saved output path differs"));
                }
                Ok(Some(output))
            })())
        })
        .await?
    }

    pub(crate) async fn cycle_case_headers(
        &self, cycle: String, members: Vec<(String, u64)>, maximum: u64,
    ) -> Result<Vec<crate::runtime::query::cases::CaseHeader>, Error> {
        let members = members.into_iter().map(|(id, sequence)| (id, super::data::position(sequence))).collect::<Vec<_>>();
        let members = serde_json::to_string(&members)?;
        self.read(move |connection| {
            Ok((|| {
                let mut statement = connection.prepare(
                    "SELECT h.body FROM json_each(?1) m CROSS JOIN query_history h
                WHERE h.kind='execution' AND h.id=json_extract(m.value,'$[0]') AND h.cycle=?2
                AND h.revision=(SELECT n.revision FROM query_history n
                    WHERE n.kind='execution' AND n.id=json_extract(m.value,'$[0]') AND n.position=json_extract(m.value,'$[1]')
                    ORDER BY n.revision DESC LIMIT 1)
                ORDER BY CAST(m.key AS INTEGER)",
                )?;
                let mut rows = statement.query(params![members, cycle])?;
                let mut headers = Vec::new();
                let mut bytes = 0u64;
                let mut retained = std::mem::size_of_val(&headers) as u64;
                while let Some(row) = rows.next()? {
                    let body = super::data::blob(row, 0)?;
                    bytes = bytes.saturating_add(body.len() as u64);
                    if bytes > maximum {
                        return Err(Error::Budget {
                            budget: "cycle case headers",
                            limit: maximum,
                            reached: bytes,
                        });
                    }
                    let projection: ExecutionProjection = serde_json::from_slice(body)?;
                    let case = projection.case.ok_or_else(|| Error::Incompatible("local catalog lacks sealed case headers".into()))?;
                    retained = retained.saturating_add(case.memory_size() as u64);
                    if retained > maximum {
                        return Err(Error::Budget {
                            budget: "cycle case headers",
                            limit: maximum,
                            reached: retained,
                        });
                    }
                    headers.push(case);
                }
                Ok(headers)
            })())
        })
        .await?
    }

    pub(crate) async fn history_cycle_case_count(&self, cycle: String) -> Result<u64, Error> {
        self.read(move |connection| {
            connection.query_row(
                "SELECT COUNT(DISTINCT json_extract(h.body,'$.case_id')) FROM query_members m CROSS JOIN query_history h
             WHERE m.cycle=?1 AND m.position=(SELECT c.position FROM query_history c
                 WHERE c.kind='cycle' AND c.id=?1 ORDER BY c.revision DESC LIMIT 1)
                 AND h.kind='execution' AND h.id=m.execution AND h.cycle=?1
                 AND h.revision=(SELECT n.revision FROM query_history n
                     WHERE n.kind='execution' AND n.id=m.execution ORDER BY n.revision DESC LIMIT 1)",
                [cycle],
                |row| row.get::<_, i64>(0),
            )
        })
        .await
        .map(|count| count as u64)
    }

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
        "SELECT execution,position,path,body FROM query_execution_outputs LIMIT 0",
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
            if let Some(seal) = &execution.saved {
                for output in &seal.source.outputs {
                    if output.file.is_none() {
                        continue;
                    }
                    connection.execute(
                        "INSERT INTO query_execution_outputs VALUES(?1,?2,?3,?4)",
                        params![
                            execution.test_execution_id.as_str(),
                            position,
                            output.relative_path,
                            serde_json::to_vec(&(&seal.dependencies_saved, output))?
                        ],
                    )?;
                }
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
                serde_json::to_vec(&ExecutionProjection {
                    case: execution.saved.as_ref().map(|_| {
                        crate::runtime::query::cases::CaseHeader::new(
                            execution.snapshot.case(),
                            0,
                            crate::runtime::query::cases::CaseProjection {
                                status: Some(crate::runtime::query::cases::status(Some(&execution))),
                                latest_execution_id: Some(execution.test_execution_id.clone()),
                                latest_execution_status: Some(execution.status.clone()),
                                attempt_count: 1,
                                actions: Default::default(),
                            },
                        )
                    }),
                    summary,
                    suite_id: execution.scope.suite_id,
                })?,
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

pub(crate) fn timestamp(time: &dokime_domain::model::time::Timestamp) -> Result<Timestamp, Error> {
    let time = time.to_datetime()?;
    Ok(Timestamp {
        seconds: time.unix_timestamp(),
        nanos: time.nanosecond() as i32,
    })
}

/* === Private Functions === */

fn advance(connection: &Connection) -> Result<i64, Error> {
    Ok(connection.query_row(
        "UPDATE query_history_head SET revision=revision+1 WHERE revision<9223372036854775807 RETURNING revision",
        [],
        |row| row.get(0),
    )?)
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

#[cfg(test)]
mod tests {
    /* === Modules === */

    /* === Re-Exports === */

    /* === Imports === */

    // Standard Library Imports

    // Third Party Imports

    // Local Imports
    use super::*;

    /* === Constants === */

    /* === Macros === */

    /* === Type Aliases === */

    /* === Structs === */

    /* === Enums === */

    /* === Traits === */

    /* === Implementations === */

    /* === Public Functions === */

    /* === Private Functions === */

    fn selection() -> ExecutionSelection {
        ExecutionSelection {
            template_ids: vec![],
            cycle_ids: vec![],
            statuses: vec![],
            started_at: None,
            archive: None,
        }
    }

    fn read(revision: i64) -> Read {
        Read {
            revision,
            cut: None,
            after: None,
            rows: 2,
            bytes: 8192,
        }
    }

    fn insert(connection: &Connection, id: &str, revision: i64, time: Option<i64>, status: wire::ExecutionStatus) {
        let row = ExecutionSummary {
            execution_id: id.into(),
            cycle_id: Some("cycle".into()),
            case_id: "case".into(),
            template_id: Some("example.measurement".into()),
            label: Some(id.into()),
            status,
            started_at: time.map(|seconds| Timestamp { seconds, nanos: 0 }),
            finished_at: None,
            result: None,
            availability: ExecutionAvailability::Uncheckpointed,
            checkpoint_available: false,
            archived: false,
        };
        connection
            .execute(
                "INSERT INTO query_history VALUES('execution',?1,?2,?3,'cycle','example.measurement',?4,0,?5,?6,?7)",
                params![
                    id,
                    revision,
                    super::super::data::position(revision as u64),
                    serde_json::to_string(&status).unwrap(),
                    time.is_none(),
                    time_key(row.started_at),
                    serde_json::to_vec(&row).unwrap()
                ],
            )
            .unwrap();
        connection.execute("UPDATE query_history_head SET revision=MAX(revision,?1)", [revision]).unwrap();
    }

    #[tokio::test]
    async fn archive_changes_are_atomic_versioned_and_recoverable() {
        let root = tempfile::tempdir().unwrap();
        let state = State::open(root.path()).await.unwrap();
        state
            .call(|connection| {
                connection.execute_batch(SCHEMA)?;
                insert(connection, "a", 1, Some(10), wire::ExecutionStatus::Failed);
                Ok(())
            })
            .await
            .unwrap();
        state.history_archives(vec!["cycle".into()], vec![]).await.unwrap();
        let old = state.history_executions(selection(), read(1)).await.unwrap();
        assert_eq!(old.rows.len(), 1);
        let current = state.history_revision().await.unwrap();
        assert!(state.history_executions(selection(), read(current)).await.unwrap().rows.is_empty());
        let mut archived = selection();
        archived.archive = Some(ArchiveScope::Archived);
        assert!(state.history_executions(archived, read(current)).await.unwrap().rows[0].archived);
        state.history_archives(vec![], vec![]).await.unwrap();
        assert_eq!(state.history_executions(selection(), read(state.history_revision().await.unwrap())).await.unwrap().rows.len(), 1);
        state.close().await.unwrap();
    }

    /* === Tests === */

    #[test]
    fn historical_pages_keep_values_membership_and_missing_times_last() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        insert(&connection, "b", 1, Some(10), wire::ExecutionStatus::Running);
        insert(&connection, "a", 2, Some(10), wire::ExecutionStatus::Failed);
        insert(&connection, "c", 3, None, wire::ExecutionStatus::Pending);
        let page = executions(&connection, &selection(), &read(3)).unwrap();
        assert_eq!(page.rows.iter().map(|row| row.execution_id.as_str()).collect::<Vec<_>>(), ["a", "b"]);
        insert(&connection, "b", 4, Some(20), wire::ExecutionStatus::Passed);
        insert(&connection, "d", 5, Some(30), wire::ExecutionStatus::Failed);
        let next = Read { after: page.next, ..read(3) };
        let page = executions(&connection, &selection(), &next).unwrap();
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].execution_id, "c");
        assert!(page.next.is_none());
        let original = executions(&connection, &selection(), &read(3)).unwrap();
        assert!(original.rows[1].status == wire::ExecutionStatus::Running);
        let fresh = executions(&connection, &selection(), &Read { rows: 10, ..read(5) }).unwrap();
        assert_eq!(fresh.rows.len(), 4);
        assert!(fresh.rows[1].status == wire::ExecutionStatus::Passed);
    }

    #[test]
    fn half_open_windows_and_exact_filters_use_covering_seeks() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        for i in 1..=1000 {
            insert(&connection, &format!("e{i:04}"), i, Some(i), wire::ExecutionStatus::Failed);
        }
        let mut selection = selection();
        selection.template_ids = vec!["example.measurement".into()];
        selection.statuses = vec![wire::ExecutionStatus::Failed];
        selection.started_at = Some(Window {
            start: Some(Timestamp { seconds: 998, nanos: 0 }),
            end: Some(Timestamp { seconds: 1000, nanos: 0 }),
        });
        let page = executions(&connection, &selection, &read(1000)).unwrap();
        assert_eq!(page.rows.iter().map(|row| row.execution_id.as_str()).collect::<Vec<_>>(), ["e0998", "e0999"]);
        let plan = execution_plan(&selection, &read(1000)).unwrap();
        let mut explain = connection.prepare(&format!("EXPLAIN QUERY PLAN {}", plan.sql)).unwrap();
        let details = explain
            .query_map(rusqlite::params_from_iter(plan.arguments), |row| row.get::<_, String>(3))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(details.iter().any(|line| line.contains("SEARCH h USING INDEX query_history_template")), "{details:?}");
        selection.template_ids = vec!["example.measurement' OR 1=1 --".into()];
        assert!(executions(&connection, &selection, &read(1000)).unwrap().rows.is_empty());
    }

    #[test]
    fn rollback_and_restart_expose_only_committed_revisions() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("history.sqlite");
        {
            let mut connection = Connection::open(&path).unwrap();
            connection.execute_batch(SCHEMA).unwrap();
            insert(&connection, "a", 1, Some(10), wire::ExecutionStatus::Running);
            let transaction = connection.transaction().unwrap();
            insert(&transaction, "a", 2, Some(10), wire::ExecutionStatus::Passed);
            drop(transaction);
        }
        let connection = Connection::open(&path).unwrap();
        check_format(&connection).unwrap();
        let page = executions(&connection, &selection(), &read(2)).unwrap();
        assert!(page.rows[0].status == wire::ExecutionStatus::Running);
        connection.execute_batch("DROP TABLE query_history").unwrap();
        assert!(matches!(check_format(&connection), Err(Error::Incompatible(_))));
    }

    #[test]
    fn native_cycle_projection_and_membership_share_the_object_transaction() {
        use dokime_domain::model::{
            identity::{Identifier, Metadata, TestCycleId, TestExecutionId},
            time::Timestamp as NativeTime,
        };
        use harmos::storage::{ObjectKey, ObjectValue, ObjectVersionToken, SchemaKey, StorageCursor, StoreHistory};
        let root = tempfile::tempdir().unwrap();
        let database = root.path().join("catalog.sqlite");
        let id = TestCycleId::from_seed("query.cycle");
        let member = TestExecutionId::from_seed("query.execution");
        let mut cycle = result::TestCycle {
            saved: None,
            test_cycle_id: id.clone(),
            session_id: None,
            metadata: Metadata::new(Identifier::new("query.cycle").unwrap(), "Cycle", None),
            phase: result::CyclePhase::Active,
            lifecycle: result::CycleStatus::Ready,
            created_at: NativeTime::new("2026-10-01T00:00:00Z"),
            started_at: None,
            updated_at: NativeTime::new("2026-10-01T00:00:00Z"),
            execution_ids: vec![member.clone()],
            records: Default::default(),
            continue_automatically: false,
            result: None,
            seal_failure: None,
        };
        let native = crate::storage::objects::cycle::Cycle::try_from(&cycle).unwrap();
        let position = StorageCursor {
            history: StoreHistory {
                store: Identifier::new("query.store").unwrap(),
                epoch: Identifier::new("query.epoch").unwrap(),
            },
            sequence: 1,
        };
        let mut object = ObjectEntry {
            version: ObjectVersionToken::new(position.clone()),
            position,
            value: ObjectValue {
                schema: SchemaKey {
                    id: Identifier::new("dokime.object.cycle").unwrap(),
                    version: 1.try_into().unwrap(),
                    payload_encoding: Default::default(),
                },
                item: ObjectKey::new(id.as_str(), 4096.try_into().unwrap()).unwrap(),
                payload: native.encode(),
            },
        };
        let mut connection = Connection::open(&database).unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        {
            let transaction = connection.transaction().unwrap();
            index(&transaction, &object).unwrap();
        }
        assert_eq!(connection.query_row("SELECT count(*) FROM query_history", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        assert_eq!(connection.query_row("SELECT count(*) FROM query_members", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        {
            let transaction = connection.transaction().unwrap();
            index(&transaction, &object).unwrap();
            transaction.commit().unwrap();
        }
        drop(connection);
        let mut connection = Connection::open(&database).unwrap();
        let selected = cycles(
            &connection,
            &CycleSelection {
                cycle_ids: vec![],
                phases: vec![],
                statuses: vec![],
                created_at: None,
                archive: None,
            },
            &read(1),
        )
        .unwrap();
        assert_eq!(selected.rows[0].execution_count, 1);
        assert_eq!(selected.rows[0].status, wire::CycleStatus::Ready);
        assert_eq!(selected.rows[0].running_count, 0);
        assert_eq!(
            connection.query_row("SELECT execution FROM query_members", [], |row| row.get::<_, String>(0)).unwrap(),
            member.to_string()
        );

        insert(&connection, member.as_str(), 2, Some(10), wire::ExecutionStatus::Running);
        connection.execute("UPDATE query_history SET cycle=?1 WHERE kind='execution'", [id.as_str()]).unwrap();
        cycle.lifecycle = result::CycleStatus::Running;
        object.position.sequence = 3;
        object.version = ObjectVersionToken::new(object.position.clone());
        object.value.payload = crate::storage::objects::cycle::Cycle::try_from(&cycle).unwrap().encode();
        index(&connection, &object).unwrap();
        let selection = CycleSelection {
            cycle_ids: vec![id.to_string()],
            phases: vec![result::CyclePhase::Active],
            statuses: vec![wire::CycleStatus::Running, wire::CycleStatus::Paused],
            created_at: None,
            archive: None,
        };
        let captured = cycles(&connection, &selection, &read(3)).unwrap();
        assert_eq!(captured.rows[0].status, wire::CycleStatus::Running);
        assert_eq!(captured.rows[0].running_count, 1);
        insert(&connection, member.as_str(), 4, Some(10), wire::ExecutionStatus::Passed);
        connection.execute("UPDATE query_history SET cycle=?1 WHERE kind='execution'", [id.as_str()]).unwrap();
        assert_eq!(cycles(&connection, &selection, &read(3)).unwrap().rows, captured.rows);
        assert_eq!(cycles(&connection, &selection, &read(4)).unwrap().rows[0].running_count, 0);
        let work = dokime_domain::state::records::work::Work {
            executions: vec![dokime_domain::state::records::work::Execution {
                id: member.clone(),
                status: result::ExecutionStatus::Running,
                started_at: Some(NativeTime::new("2026-10-01T00:00:01Z")),
                finished_at: None,
                result: None,
            }],
            cycles: vec![dokime_domain::state::records::work::Cycle {
                id: id.clone(),
                status: result::CycleStatus::Paused,
            }],
        };
        let row = crate::storage::sqlite::history::work::Row { body: serde_json::to_vec(&work).unwrap() };
        let record = harmos::storage::StoredRecord {
            position: StorageCursor { sequence: 5, ..object.position.clone() },
            schema: crate::storage::sqlite::history::work::schema(),
            scope: None,
            item: Identifier::new("work.fixture").unwrap(),
            source: None,
            source_time: None,
            payload: row.encode(),
        };
        {
            let transaction = connection.transaction().unwrap();
            super::work::index(&transaction, &record).unwrap();
        }
        assert_eq!(cycles(&connection, &selection, &read(5)).unwrap().rows[0].running_count, 0);
        {
            let transaction = connection.transaction().unwrap();
            super::work::index(&transaction, &record).unwrap();
            transaction.commit().unwrap();
        }
        let paused = cycles(&connection, &selection, &read(6)).unwrap();
        assert_eq!(paused.rows[0].status, wire::CycleStatus::Paused);
        assert_eq!(paused.rows[0].running_count, 1);
        assert_eq!(cycles(&connection, &selection, &read(3)).unwrap().rows, captured.rows);
        // A Later Native Header Must Not Restore Its Older Running Status.
        object.position.sequence = 6;
        object.version = ObjectVersionToken::new(object.position.clone());
        index(&connection, &object).unwrap();
        assert_eq!(cycles(&connection, &selection, &read(7)).unwrap().rows[0].status, wire::CycleStatus::Paused);
        let members: i64 = connection
            .query_row("SELECT count(*) FROM query_members WHERE cycle=?1 AND position='0000000000000005'", [id.as_str()], |row| row.get(0))
            .unwrap();
        assert_eq!(members, 1);
        let sealed = CycleSelection {
            phases: vec![result::CyclePhase::Sealed],
            ..selection
        };
        assert!(cycles(&connection, &sealed, &read(4)).unwrap().rows.is_empty());
    }
}
