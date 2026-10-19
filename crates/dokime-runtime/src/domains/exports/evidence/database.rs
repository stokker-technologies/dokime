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
use std::{num::NonZeroUsize, path::Path};

// Third Party Imports
use rusqlite::{Connection, params, params_from_iter, types::Value as SqlValue};
use serde::Serialize;
use tokio_util::sync::CancellationToken;

// Local Imports
use super::{
    InventoryBudget, MAX_INVENTORY_ENTRIES,
    error::{Error, Result},
    snapshot::Snapshot,
};
use crate::runtime::storage::{execution::ExecutionView, telemetry::TelemetryReader};
use dokime_domain::model::{
    definition::MaterializedParameter,
    exports::{EVIDENCE_EXPORT_MAX_BYTES, EVIDENCE_EXPORT_SCHEMA_VERSION},
    telemetry::TelemetryContract,
    values::{Value, ValueType},
};

/* === Constants === */

const PARTITION_COLUMNS: usize = 512;
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
CREATE TABLE streams(
    execution_id TEXT NOT NULL, stream_id TEXT NOT NULL, schema_version INTEGER NOT NULL,
    table_name TEXT PRIMARY KEY, sample_count INTEGER NOT NULL, schema_json TEXT NOT NULL
) STRICT;
CREATE TABLE stream_columns(
    base_table TEXT NOT NULL, table_name TEXT NOT NULL, field_id TEXT NOT NULL,
    column_name TEXT NOT NULL, value_type TEXT NOT NULL, unit_json TEXT,
    body_json TEXT NOT NULL, PRIMARY KEY(table_name, column_name)
) STRICT;
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
    next_table: u64,
}

struct Stream {
    table: String,
    partitions: Vec<Partition>,
    samples: u64,
}

struct Partition {
    table: String,
    fields: Vec<String>,
    insert: String,
    raw: bool,
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
    pub table_name: String,
    pub partitions: Vec<String>,
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
             PRAGMA synchronous=FULL; PRAGMA cache_size=-4096; PRAGMA trusted_schema=OFF; \
             PRAGMA user_version={}; BEGIN IMMEDIATE; {SCHEMA}",
            EVIDENCE_EXPORT_MAX_BYTES / 4096,
            EVIDENCE_EXPORT_SCHEMA_VERSION,
        ))?;
        let phase =
            snapshot.cycle.get("phase").and_then(serde_json::Value::as_str).ok_or(Error::Invalid("captured cycle phase is missing"))?;
        connection.execute("INSERT INTO cycles VALUES(?1,?2,?3)", params![snapshot.cycle_id.as_str(), phase, json(&snapshot.cycle)?])?;
        Ok(Self { connection, streams: Vec::new(), next_table: 0 })
    }

    pub async fn execution(
        &mut self, view: &ExecutionView, cancellation: &CancellationToken, budget: &mut InventoryBudget,
    ) -> Result<Counts> {
        cancelled(cancellation)?;
        self.header(view)?;
        self.definitions(view)?;
        self.outcomes(view)?;
        self.annotations(view)?;
        let id = view.execution().test_execution_id.to_string();
        let mut counts = Counts { execution_id: id.clone(), records: 0, samples: 0 };
        let mut reader = view.context_records().await?;
        while let Some(records) = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(Error::Cancelled),
            result = reader.next() => result?,
        } {
            cancelled(cancellation)?;
            for entry in records {
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
        for versions in view.source_snapshot()?.telemetry_contracts.values() {
            for contract in versions.values() {
                if self.streams.len() >= MAX_INVENTORY_ENTRIES {
                    return Err(Error::TooLarge);
                }
                let mut stream = self.stream(&id, contract)?;
                let mut reader = TelemetryReader::for_contract(
                    view,
                    &contract.stream,
                    contract.schema_version,
                    NonZeroUsize::new(256).expect("nonzero batch size"),
                )?;
                while let Some(samples) = tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err(Error::Cancelled),
                    result = reader.next() => result?,
                } {
                    for entry in samples {
                        self.sample(&id, &mut stream, &entry)?;
                        counts.samples = counts.samples.checked_add(1).ok_or(Error::TooLarge)?;
                    }
                }
                self.connection
                    .execute("UPDATE streams SET sample_count=?1 WHERE table_name=?2", params![ordinal(stream.samples)?, &stream.table])?;
                let inventory = StreamInventory {
                    execution_id: id.clone(),
                    stream_id: contract.stream.to_string(),
                    schema_version: contract.schema_version,
                    table_name: stream.table,
                    partitions: stream.partitions.into_iter().map(|partition| partition.table).collect(),
                    samples: stream.samples,
                };
                budget.reserve(&inventory)?;
                self.streams.push(inventory);
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

    fn stream(&mut self, execution: &str, contract: &TelemetryContract) -> Result<Stream> {
        let count = contract.columns.len().max(1).div_ceil(PARTITION_COLUMNS);
        if count > MAX_INVENTORY_ENTRIES {
            return Err(Error::TooLarge);
        }
        let mut partitions = Vec::with_capacity(count);
        let mut base = String::new();
        for partition in 0..count {
            let table = format!("stream_{:06}", self.next_table);
            self.next_table = self.next_table.checked_add(1).ok_or(Error::TooLarge)?;
            if partition == 0 {
                base = table.clone();
            }
            let start = partition * PARTITION_COLUMNS;
            let end = (start + PARTITION_COLUMNS).min(contract.columns.len());
            let fields = &contract.columns[start..end];
            let mut columns = String::from(
                "m_row INTEGER PRIMARY KEY, m_execution_id TEXT NOT NULL, m_ordinal ANY NOT NULL, m_sequence ANY NOT NULL, m_timestamp TEXT NOT NULL, m_received_at TEXT NOT NULL, m_source TEXT NOT NULL",
            );
            if partition == 0 {
                columns.push_str(", sample_json TEXT NOT NULL");
            }
            for (offset, field) in fields.iter().enumerate() {
                let column = format!("c{:04}", start + offset);
                let sql_type = match field.value_type {
                    ValueType::Boolean => "INTEGER",
                    ValueType::Integer | ValueType::Number => "ANY",
                    _ => "TEXT",
                };
                columns.push_str(&format!(", {column} {sql_type}"));
                self.connection.execute(
                    "INSERT INTO stream_columns VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        &base,
                        &table,
                        field.metadata.identifier.as_str(),
                        &column,
                        label(&field.value_type)?,
                        optional_json(field.unit.as_ref())?,
                        json(field)?,
                    ],
                )?;
            }
            self.connection.execute_batch(&format!("CREATE TABLE {table}({columns}) STRICT;"))?;
            let parameters = (0..(7 + usize::from(partition == 0) + fields.len())).map(|_| "?").collect::<Vec<_>>().join(",");
            partitions.push(Partition {
                insert: format!("INSERT INTO {table} VALUES({parameters})"),
                table,
                fields: fields.iter().map(|field| field.metadata.identifier.to_string()).collect(),
                raw: partition == 0,
            });
        }
        self.connection.execute(
            "INSERT INTO streams VALUES(?1,?2,?3,?4,0,?5)",
            params![execution, contract.stream.as_str(), contract.schema_version, &base, json(contract)?,],
        )?;
        Ok(Stream { table: base, partitions, samples: 0 })
    }

    fn sample(&self, execution: &str, stream: &mut Stream, entry: &crate::storage::telemetry::TelemetryEntry) -> Result<()> {
        let sample = &entry.sample;
        for partition in &stream.partitions {
            let mut values = vec![
                SqlValue::Integer(ordinal(stream.samples)?),
                SqlValue::Text(execution.to_owned()),
                unsigned(entry.ordinal),
                unsigned(sample.sequence),
                SqlValue::Text(sample.timestamp.to_string()),
                SqlValue::Text(sample.received_at.to_string()),
                SqlValue::Text(sample.source.clone()),
            ];
            if partition.raw {
                values.push(SqlValue::Text(json(entry)?));
            }
            for field in &partition.fields {
                values.push(sample.values.get(field).map(cell).transpose()?.unwrap_or(SqlValue::Null));
            }
            self.connection.prepare_cached(&partition.insert)?.execute(params_from_iter(values))?;
        }
        stream.samples = stream.samples.checked_add(1).ok_or(Error::TooLarge)?;
        Ok(())
    }

    pub fn output(
        &self, execution: &str, index: usize, archive: &str, value: &dokime_domain::model::execution::saved::ExecutionOutput,
    ) -> Result<()> {
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
