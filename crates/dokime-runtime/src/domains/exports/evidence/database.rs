/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Write Captured Evidence To Typed SQLite Tables

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

// Third Party Imports
use rusqlite::{Connection, params, types::Value as SqlValue};
use serde::Serialize;
use tokio_util::sync::CancellationToken;

// Local Imports
use super::{
    InventoryBudget,
    error::{Error, Result},
    snapshot::Snapshot,
};
use crate::runtime::storage::execution::ExecutionView;
use dokime_domain::model::{definition::MaterializedParameter, exports::EVIDENCE_EXPORT_SCHEMA_VERSION, values::Value};

/* === Constants === */

const SCHEMA: &str = r#"
CREATE TABLE cycles(cycle_id TEXT PRIMARY KEY, phase TEXT NOT NULL, body_json TEXT NOT NULL) STRICT;
CREATE TABLE executions(
    execution_id TEXT PRIMARY KEY, cycle_id TEXT NOT NULL, status TEXT NOT NULL,
    captured_at TEXT NOT NULL, revision_json TEXT NOT NULL, body_json TEXT NOT NULL
) STRICT;
CREATE TABLE definitions(
    execution_id TEXT NOT NULL, kind TEXT NOT NULL, identifier TEXT NOT NULL, body_json TEXT NOT NULL,
    PRIMARY KEY(execution_id, kind, identifier)
) STRICT;
CREATE TABLE inputs(
    execution_id TEXT NOT NULL, direction TEXT NOT NULL, identifier TEXT NOT NULL,
    source_kind TEXT NOT NULL, value ANY, value_json TEXT NOT NULL, unit_json TEXT, body_json TEXT NOT NULL,
    PRIMARY KEY(execution_id, direction, identifier)
) STRICT;
CREATE TABLE steps(
    execution_id TEXT NOT NULL, step_id TEXT NOT NULL, label TEXT NOT NULL, status TEXT NOT NULL,
    started_at TEXT, finished_at TEXT, body_json TEXT NOT NULL, PRIMARY KEY(execution_id, step_id)
) STRICT;
CREATE TABLE criteria(
    execution_id TEXT NOT NULL, step_id TEXT NOT NULL, criterion_id TEXT NOT NULL,
    status TEXT NOT NULL, observed ANY, body_json TEXT NOT NULL,
    PRIMARY KEY(execution_id, step_id, criterion_id)
) STRICT;
CREATE TABLE measurements(
    execution_id TEXT NOT NULL, step_id TEXT NOT NULL, ordinal INTEGER NOT NULL,
    identifier TEXT NOT NULL, value ANY, unit_json TEXT, body_json TEXT NOT NULL,
    PRIMARY KEY(execution_id, step_id, ordinal)
) STRICT;
CREATE TABLE step_attempts(
    execution_id TEXT NOT NULL, attempt_id TEXT NOT NULL, step_id TEXT NOT NULL, body_json TEXT NOT NULL,
    PRIMARY KEY(execution_id, attempt_id)
) STRICT;
CREATE TABLE annotations(
    execution_id TEXT NOT NULL, kind TEXT NOT NULL, ordinal INTEGER NOT NULL, body_json TEXT NOT NULL,
    PRIMARY KEY(execution_id, kind, ordinal)
) STRICT;
CREATE TABLE traceability(
    execution_id TEXT NOT NULL, ordinal INTEGER NOT NULL, body_json TEXT NOT NULL,
    PRIMARY KEY(execution_id, ordinal)
) STRICT;
CREATE TABLE records(
    execution_id TEXT NOT NULL, ordinal INTEGER NOT NULL, epoch TEXT NOT NULL, record_id TEXT NOT NULL,
    sequence ANY NOT NULL, record_type TEXT NOT NULL, schema_version INTEGER NOT NULL,
    timestamp TEXT NOT NULL, received_at TEXT NOT NULL, source TEXT NOT NULL, severity TEXT NOT NULL,
    message TEXT, body_json TEXT NOT NULL, PRIMARY KEY(execution_id, ordinal)
) STRICT;
CREATE TABLE files(id TEXT PRIMARY KEY,kind TEXT NOT NULL CHECK(kind IN ('artifact','telemetry','report')),
    path TEXT NOT NULL,size_bytes INTEGER NOT NULL,sha256 TEXT NOT NULL,media_type TEXT NOT NULL) STRICT;
CREATE TABLE artifacts(file_id TEXT PRIMARY KEY REFERENCES files(id),execution_id TEXT NOT NULL REFERENCES executions(execution_id),
    relative_path TEXT NOT NULL) STRICT;
CREATE TABLE streams(file_id TEXT PRIMARY KEY REFERENCES files(id),execution_id TEXT NOT NULL REFERENCES executions(execution_id),
    stream_id TEXT NOT NULL,rows INTEGER NOT NULL,first_at INTEGER,last_at INTEGER,batches_json TEXT NOT NULL,contract_json TEXT NOT NULL) STRICT;
CREATE TABLE stream_summaries(file_id TEXT NOT NULL REFERENCES streams(file_id),signal TEXT NOT NULL,count INTEGER NOT NULL,
    minimum REAL NOT NULL,maximum REAL NOT NULL,sum REAL NOT NULL,PRIMARY KEY(file_id,signal)) STRICT;
CREATE TABLE outputs(
    execution_id TEXT NOT NULL, ordinal INTEGER NOT NULL, relative_path TEXT NOT NULL,
    archive_path TEXT NOT NULL, size_bytes ANY NOT NULL, sha256 TEXT NOT NULL, body_json TEXT NOT NULL,
    PRIMARY KEY(execution_id, ordinal)
) STRICT;
"#;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Writer {
    connection: Connection,
    streams: Vec<StreamInventory>,
}

#[derive(Clone, Serialize)]
pub(super) struct Counts {
    pub execution_id: String,
    pub records: u64,
    pub samples: u64,
}

#[derive(Serialize)]
pub(super) struct StreamInventory {
    pub execution_id: String,
    pub stream_id: String,
    pub schema_version: u32,
    pub file_id: String,
    pub path: String,
    pub samples: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Writer {
    pub fn open(path: &Path, snapshot: &Snapshot) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(&format!(
            "PRAGMA page_size=4096; PRAGMA max_page_count={}; PRAGMA journal_mode=DELETE; \
             PRAGMA synchronous=FULL; PRAGMA trusted_schema=OFF; \
             PRAGMA user_version={}; BEGIN IMMEDIATE; {SCHEMA}",
            super::MAX_EXPORT_BYTES / 4096,
            EVIDENCE_EXPORT_SCHEMA_VERSION,
        ))?;
        let phase =
            snapshot.cycle.get("phase").and_then(serde_json::Value::as_str).ok_or(Error::Invalid("captured cycle phase is missing"))?;
        connection.execute("INSERT INTO cycles VALUES(?1,?2,?3)", params![snapshot.cycle_id.as_str(), phase, json(&snapshot.cycle)?])?;
        Ok(Self { connection, streams: Vec::new() })
    }

    pub async fn execution(
        &mut self, view: &ExecutionView, cancellation: &CancellationToken, _budget: &mut InventoryBudget,
    ) -> Result<Counts> {
        cancelled(cancellation)?;
        self.header(view)?;
        self.definitions(view)?;
        self.outcomes(view)?;
        self.annotations(view)?;
        let id = view.execution().test_execution_id.to_string();
        let mut counts = Counts {
            execution_id: id.clone(),
            records: 0,
            samples: 0,
        };
        let mut reader = view.context_records().await?;
        while let Some(records) = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(Error::Cancelled),
            result = reader.next() => result?,
        } {
            cancelled(cancellation)?;
            for entry in records.iter() {
                let record = &entry.record;
                self.connection.execute(
                    "INSERT INTO records VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
                    params![
                        &id,
                        ordinal(counts.records)?,
                        entry.epoch.as_str(),
                        record.record_id.as_str(),
                        unsigned(record.sequence),
                        record.record_type.as_str(),
                        record.schema_version,
                        record.timestamp.as_str(),
                        record.received_at.as_str(),
                        &record.source,
                        label(&record.severity)?,
                        &record.message,
                        json(&entry)?,
                    ],
                )?;
                counts.records = counts.records.checked_add(1).ok_or(Error::TooLarge)?;
            }
        }
        Ok(counts)
    }

    fn header(&self, view: &ExecutionView) -> Result<()> {
        let execution = view.execution();
        let revision = view.checkpoint()?;
        self.connection.execute(
            "INSERT INTO executions VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                execution.test_execution_id.as_str(),
                execution.test_cycle_id.as_str(),
                label(&execution.status)?,
                view.captured_at()?.as_str(),
                json(&revision)?,
                json(execution)?,
            ],
        )?;
        for (name, parameter) in execution.snapshot.parameters() {
            self.input(execution.test_execution_id.as_str(), "input", name, parameter)?;
        }
        for (name, parameter) in &execution.output_parameters {
            self.input(execution.test_execution_id.as_str(), "output", name, parameter)?;
        }
        Ok(())
    }

    fn input(&self, execution: &str, direction: &str, name: &str, parameter: &MaterializedParameter) -> Result<()> {
        self.connection.execute(
            "INSERT INTO inputs VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                execution,
                direction,
                name,
                label(&parameter.source)?,
                cell(&parameter.value)?,
                json(&parameter.value)?,
                optional_json(parameter.unit.as_ref())?,
                json(parameter)?,
            ],
        )?;
        Ok(())
    }

    fn definition(&self, execution: &str, kind: &str, id: &str, value: &impl Serialize) -> Result<()> {
        self.connection.execute("INSERT INTO definitions VALUES(?1,?2,?3,?4)", params![execution, kind, id, json(value)?])?;
        Ok(())
    }

    fn definitions(&self, view: &ExecutionView) -> Result<()> {
        let execution = view.execution();
        let id = execution.test_execution_id.as_str();
        let snapshot = &execution.snapshot;
        self.definition(id, "case", execution.test_case_id.as_str(), snapshot.case())?;
        if let Some(template) = snapshot.template() {
            self.definition(id, "template", snapshot.case().test_case_template_id.as_str(), template)?;
        }
        if let Some(suite) = snapshot.suite() {
            self.definition(id, "suite", suite.identifier.as_str(), suite)?;
        }
        for parameter in snapshot.parameter_definitions() {
            self.definition(id, "parameter", parameter.metadata.identifier.as_str(), parameter)?;
        }
        for (document_id, document) in &view.context()?.execution_documents {
            self.definition(id, "document", document_id.as_str(), document)?;
        }
        for (record_type, versions) in &view.source_snapshot()?.record_contracts {
            for (version, contract) in versions {
                self.definition(id, "record_schema", &format!("{record_type}@{version}"), contract)?;
            }
        }
        Ok(())
    }

    fn outcomes(&self, view: &ExecutionView) -> Result<()> {
        let execution = view.execution();
        let id = execution.test_execution_id.as_str();
        for step in &execution.steps {
            self.connection.execute(
                "INSERT INTO steps VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    id,
                    step.step_id.as_str(),
                    &step.metadata.label,
                    label(&step.status)?,
                    step.started_at.as_ref().map(|time| time.as_str()),
                    step.finished_at.as_ref().map(|time| time.as_str()),
                    json(step)?,
                ],
            )?;
            for criterion in &step.criteria {
                let observed = criterion.observed.as_ref().map(cell).transpose()?;
                self.connection.execute(
                    "INSERT INTO criteria VALUES(?1,?2,?3,?4,?5,?6)",
                    params![
                        id,
                        step.step_id.as_str(),
                        criterion.criterion_id.as_str(),
                        label(&criterion.status)?,
                        observed,
                        json(criterion)?,
                    ],
                )?;
            }
            for (index, measurement) in step.measurements.iter().enumerate() {
                self.connection.execute(
                    "INSERT INTO measurements VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        id,
                        step.step_id.as_str(),
                        ordinal(index as u64)?,
                        measurement.metadata.identifier.as_str(),
                        cell(&measurement.value)?,
                        optional_json(measurement.unit.as_ref())?,
                        json(measurement)?,
                    ],
                )?;
            }
        }
        for attempt in &execution.step_attempts {
            self.connection.execute(
                "INSERT INTO step_attempts VALUES(?1,?2,?3,?4)",
                params![id, attempt.attempt_id.as_str(), attempt.step_id.as_str(), json(attempt)?,],
            )?;
        }
        Ok(())
    }

    fn annotation(&self, id: &str, kind: &str, index: usize, value: &impl Serialize) -> Result<()> {
        self.connection.execute("INSERT INTO annotations VALUES(?1,?2,?3,?4)", params![id, kind, ordinal(index as u64)?, json(value)?])?;
        Ok(())
    }

    fn annotations(&self, view: &ExecutionView) -> Result<()> {
        let context = view.context()?;
        let id = view.execution().test_execution_id.as_str();
        for (index, value) in context.notes.iter().enumerate() {
            self.annotation(id, "note", index, value)?;
        }
        for (index, value) in context.prompts.iter().enumerate() {
            self.annotation(id, "prompt", index, value)?;
        }
        for (index, value) in context.health.iter().enumerate() {
            self.annotation(id, "health", index, value)?;
        }
        for (index, value) in context.traceability.iter().enumerate() {
            self.connection.execute("INSERT INTO traceability VALUES(?1,?2,?3)", params![id, ordinal(index as u64)?, json(value)?])?;
        }
        Ok(())
    }

    pub fn telemetry(
        &mut self, execution: &str, file: &crate::storage::sqlite::StreamFile, path: &str, budget: &mut InventoryBudget,
    ) -> Result<()> {
        let digest = super::archive::hex(&file.sha256);
        self.connection.execute(
            "INSERT INTO files VALUES (?1,'telemetry',?2,?3,?4,'application/vnd.apache.arrow.file')",
            params![file.id, path, ordinal(file.size_bytes)?, digest],
        )?;
        self.connection.execute(
            "INSERT INTO streams VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                file.id,
                execution,
                file.stream,
                ordinal(file.rows)?,
                file.first_at,
                file.last_at,
                json(&file.batches)?,
                json(&file.contract)?
            ],
        )?;
        for summary in &file.summaries {
            self.connection.execute(
                "INSERT INTO stream_summaries VALUES (?1,?2,?3,?4,?5,?6)",
                params![file.id, summary.signal, ordinal(summary.count)?, summary.minimum, summary.maximum, summary.sum],
            )?;
        }
        let inventory = StreamInventory {
            execution_id: execution.to_owned(),
            stream_id: file.stream.clone(),
            schema_version: file.contract.schema_version,
            file_id: file.id.clone(),
            path: path.to_owned(),
            samples: file.rows,
        };
        budget.reserve(&inventory)?;
        self.streams.push(inventory);
        Ok(())
    }

    pub fn file(&self, reference: &harmos::storage::FileReference, path: &str, kind: &str) -> Result<()> {
        self.connection.execute(
            "INSERT OR IGNORE INTO files VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                reference.metadata.id.as_str(),
                kind,
                path,
                ordinal(reference.length)?,
                super::archive::hex(&reference.sha256),
                reference.metadata.media_type
            ],
        )?;
        Ok(())
    }

    pub fn output(
        &self, execution: &str, index: usize, archive: &str, value: &dokime_domain::model::execution::saved::ExecutionOutput,
    ) -> Result<()> {
        self.file(&value.file, archive, "artifact")?;
        self.connection
            .execute("INSERT INTO artifacts VALUES (?1,?2,?3)", params![value.file.metadata.id.as_str(), execution, value.relative_path])?;
        let sha256 = super::archive::hex(&value.file.sha256);
        self.connection.execute(
            "INSERT INTO outputs VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![execution, ordinal(index as u64)?, &value.relative_path, archive, unsigned(value.file.length), sha256, json(value)?,],
        )?;
        Ok(())
    }

    pub fn finish(self) -> Result<Vec<StreamInventory>> {
        self.connection.execute_batch("COMMIT;")?;
        self.connection.close().map_err(|(_, error)| Error::Sqlite(error))?;
        Ok(self.streams)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn cancelled(token: &CancellationToken) -> Result<()> {
    if token.is_cancelled() { Err(Error::Cancelled) } else { Ok(()) }
}

fn ordinal(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| Error::TooLarge)
}

fn unsigned(value: u64) -> SqlValue {
    i64::try_from(value).map_or_else(|_| SqlValue::Text(value.to_string()), SqlValue::Integer)
}

fn json(value: &impl Serialize) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

fn optional_json(value: Option<&impl Serialize>) -> Result<Option<String>> {
    value.map(json).transpose()
}

fn label(value: &impl Serialize) -> Result<String> {
    serde_json::to_value(value)?.as_str().map(str::to_owned).ok_or(Error::Invalid("captured enum has no textual identity"))
}

fn cell(value: &Value) -> Result<SqlValue> {
    Ok(match value {
        Value::Null => SqlValue::Null,
        Value::Bool(value) => SqlValue::Integer(i64::from(*value)),
        Value::Integer(value) => SqlValue::Integer(*value),
        Value::Unsigned(value) => unsigned(*value),
        Value::Number(value) if value.is_finite() => SqlValue::Real(*value),
        Value::Number(_) => return Err(Error::Invalid("captured non-finite value cannot be exported")),
        Value::Text(value) => SqlValue::Text(value.clone()),
        Value::Datetime(_) => {
            let text = value.to_plain_json().map_err(|_| Error::Invalid("captured datetime cannot be exported"))?;
            SqlValue::Text(text.as_str().ok_or(Error::Invalid("captured datetime is not text"))?.to_owned())
        }
        Value::Array(_) | Value::Object(_) => {
            let value = value.to_plain_json().map_err(|_| Error::Invalid("captured structured value cannot be exported"))?;
            SqlValue::Text(serde_json::to_string(&value)?)
        }
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_cells() {
        assert_eq!(cell(&Value::Integer(i64::MIN)).unwrap(), SqlValue::Integer(i64::MIN));
        assert_eq!(cell(&Value::Unsigned(i64::MAX as u64)).unwrap(), SqlValue::Integer(i64::MAX));
        assert_eq!(cell(&Value::Unsigned(u64::MAX)).unwrap(), SqlValue::Text("18446744073709551615".into()));
        let nested = Value::Array(vec![Value::Unsigned(u64::MAX), Value::Integer(-1), Value::Number(1.25)]);
        let SqlValue::Text(encoded) = cell(&nested).unwrap() else {
            panic!("structured SQL cell must be JSON text");
        };
        let plain: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(plain[0].as_u64(), Some(u64::MAX));
        assert_eq!(plain[1].as_i64(), Some(-1));
        assert_eq!(plain[2].as_f64(), Some(1.25));
        assert!(cell(&Value::Number(f64::NAN)).is_err());
    }
}
