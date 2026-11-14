/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

// Third Party Imports
use prost::Message;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Evidence, Execution, ExecutionSnapshot, ExecutionState, Result, Sample, streams};
use dokime_protocol::context::evidence::RecordsAppendRequest;
use dokime_protocol::v1;
use harmos_protocol::v1::Catalog;
use harmos_protocol::v1::History;
use harmos_protocol::v1::Receipt;
use prost_reflect::DescriptorPool;
use prost_reflect::DynamicMessage;
use std::fs::OpenOptions;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::sync::watch::Sender;

/* === Constants === */

const APPLICATION_ID: i32 = 0x444b4e58;

/* === Macros === */

/* === Type Aliases === */

type StatisticsCache = HashMap<(String, String, String, u64), (u64, crate::StatisticsRead)>;

/* === Structs === */

#[derive(Clone)]
pub struct Store {
    connection: Arc<Mutex<Connection>>,
    database: Arc<PathBuf>,
    pub(crate) identity: Arc<str>,
    pub(crate) cancelled: Option<Arc<AtomicBool>>,
    pub(crate) statistics: Arc<Mutex<StatisticsCache>>,
    pub(crate) verified: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    pub(crate) changed: Sender<u64>,
}

pub struct ReadCancellation {
    cancelled: Arc<AtomicBool>,
    interrupt: rusqlite::InterruptHandle,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for ReadCancellation {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        self.interrupt.interrupt();
    }
}
impl Store {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn open(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root)?;
        let database = root.join("execution.sqlite3");
        let exists = database.exists();
        let occupied = std::fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?.iter().any(|entry| entry.file_name() != "local.lock");
        if !exists && occupied {
            return Err(Error::Incompatible(root.display().to_string()));
        }
        let connection = Connection::open(&database)?;
        configure_read_functions(&connection)?;
        if exists {
            let identity: i32 = connection
                .query_row("PRAGMA application_id", [], |row| row.get(0))
                .map_err(|error| Error::Incompatible(error.to_string()))?;
            let version: i32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
            if identity != APPLICATION_ID || version != 1 {
                return Err(Error::Incompatible(root.display().to_string()));
            }
            let mut statement =
                connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")?;
            let tables = statement.query_map([], |row| row.get::<_, String>(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
            if tables
                != [
                    "administration",
                    "attempts",
                    "bindings",
                    "cases",
                    "commands",
                    "content_chunks",
                    "content_objects",
                    "cycle_history",
                    "cycles",
                    "deletions",
                    "evidence",
                    "evidence_reads",
                    "executions",
                    "file_requests",
                    "file_uploads",
                    "notes",
                    "plugins",
                    "read_positions",
                    "record_fields",
                    "registrations",
                    "reports",
                    "resource_requests",
                    "resource_values",
                    "sample_batches",
                    "sample_fields",
                    "samples",
                    "streams",
                ]
            {
                return Err(Error::Incompatible(root.display().to_string()));
            }
            let response_column: bool =
                connection
                    .query_row("SELECT EXISTS(SELECT 1 FROM pragma_table_info('deletions') WHERE name='response')", [], |row| row.get(0))?;
            if !response_column {
                return Err(Error::Incompatible(root.display().to_string()));
            }
        } else {
            connection.execute_batch(&format!("PRAGMA application_id={APPLICATION_ID}; PRAGMA user_version=1;
                CREATE TABLE administration(id TEXT PRIMARY KEY, payload BLOB NOT NULL);
                CREATE TABLE executions(id TEXT PRIMARY KEY, cycle TEXT NOT NULL, state INTEGER NOT NULL, payload BLOB NOT NULL, seal BLOB NOT NULL);
                CREATE INDEX executions_cycle ON executions(cycle,id);
                CREATE TABLE evidence(sequence INTEGER PRIMARY KEY AUTOINCREMENT, execution TEXT NOT NULL REFERENCES executions(id), position INTEGER NOT NULL, payload BLOB NOT NULL, UNIQUE(execution,position));
                CREATE TABLE evidence_reads(position INTEGER PRIMARY KEY);
                CREATE TABLE attempts(execution TEXT NOT NULL REFERENCES executions(id), step TEXT NOT NULL, operation TEXT NOT NULL, attempt INTEGER NOT NULL, position INTEGER NOT NULL, PRIMARY KEY(execution,step,operation,attempt,position));
                CREATE TABLE samples(execution TEXT NOT NULL REFERENCES executions(id), position INTEGER NOT NULL, stream TEXT NOT NULL, payload BLOB NOT NULL, source_ns INTEGER NOT NULL, sequence INTEGER NOT NULL, PRIMARY KEY(execution,position));
                CREATE TABLE streams(execution TEXT NOT NULL REFERENCES executions(id), stream TEXT NOT NULL, closed INTEGER NOT NULL, PRIMARY KEY(execution,stream));
                CREATE INDEX samples_stream_position ON samples(execution,stream,position);
                CREATE INDEX samples_stream_time ON samples(execution,stream,source_ns,position);
                CREATE TABLE sample_batches(execution TEXT NOT NULL REFERENCES executions(id), position INTEGER NOT NULL, stream TEXT NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(execution,position));
                CREATE TABLE sample_fields(execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL, position INTEGER NOT NULL, source_ns INTEGER NOT NULL, number REAL, label TEXT, PRIMARY KEY(execution,stream,field,position));
                CREATE INDEX sample_fields_time ON sample_fields(execution,stream,field,source_ns,position,number);
                CREATE TABLE content_objects(id TEXT PRIMARY KEY, execution TEXT NOT NULL, path TEXT NOT NULL, artifact INTEGER NOT NULL, metadata BLOB NOT NULL, length INTEGER NOT NULL, sha256 TEXT NOT NULL, phase INTEGER NOT NULL);
                CREATE TABLE file_uploads(id TEXT PRIMARY KEY REFERENCES content_objects(id), metadata BLOB NOT NULL, generation INTEGER NOT NULL);
                CREATE TABLE file_requests(sequence INTEGER PRIMARY KEY AUTOINCREMENT, operation TEXT NOT NULL, request TEXT NOT NULL, fingerprint BLOB NOT NULL, response BLOB NOT NULL, UNIQUE(operation,request));
                CREATE TABLE content_chunks(object TEXT NOT NULL REFERENCES content_objects(id), offset INTEGER NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(object,offset));
                CREATE INDEX content_path ON content_objects(execution,path);
                CREATE TABLE record_fields(execution TEXT NOT NULL, record TEXT NOT NULL, field TEXT NOT NULL, position INTEGER NOT NULL, source_ns INTEGER NOT NULL, number REAL, label TEXT, PRIMARY KEY(execution,record,field,position));
                CREATE INDEX record_fields_time ON record_fields(execution,record,field,source_ns,position);
                CREATE TABLE notes(id TEXT PRIMARY KEY, cycle TEXT NOT NULL, execution TEXT, receipt BLOB NOT NULL, payload BLOB NOT NULL);
                CREATE TABLE plugins(id TEXT PRIMARY KEY, payload BLOB NOT NULL);
                CREATE TABLE registrations(id INTEGER PRIMARY KEY, key TEXT UNIQUE, plan BLOB NOT NULL, payload BLOB NOT NULL);
                CREATE TABLE cases(id TEXT PRIMARY KEY, cycle TEXT NOT NULL, payload BLOB NOT NULL);
                CREATE TABLE cycles(id TEXT PRIMARY KEY, payload BLOB NOT NULL);
                CREATE TABLE bindings(execution TEXT PRIMARY KEY REFERENCES executions(id), case_id TEXT NOT NULL, label TEXT NOT NULL, archived INTEGER NOT NULL);
                CREATE TABLE commands(request TEXT NOT NULL, id TEXT NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(request,id));
                CREATE TABLE cycle_history(sequence INTEGER PRIMARY KEY AUTOINCREMENT, cycle TEXT NOT NULL, execution TEXT NOT NULL, payload BLOB NOT NULL);
                CREATE TABLE deletions(request TEXT PRIMARY KEY, execution TEXT NOT NULL, seal BLOB NOT NULL, response BLOB NOT NULL);
                CREATE TABLE reports(execution TEXT NOT NULL REFERENCES executions(id), id TEXT NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(execution,id));
                CREATE TABLE read_positions(execution TEXT NOT NULL REFERENCES executions(id), position INTEGER NOT NULL, PRIMARY KEY(execution,position));
                CREATE TABLE resource_values(execution TEXT NOT NULL REFERENCES executions(id), resource TEXT NOT NULL, position INTEGER NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(execution,resource,position));
                CREATE INDEX resource_values_position ON resource_values(execution,position);
                CREATE TABLE resource_requests(execution TEXT NOT NULL REFERENCES executions(id), resource TEXT NOT NULL, request TEXT NOT NULL, fingerprint BLOB NOT NULL, position INTEGER NOT NULL, PRIMARY KEY(execution,resource,request));"))?;
        }
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL; PRAGMA journal_mode=WAL;")?;
        let identity_path = root.join("backend-id");
        let identity = if identity_path.exists() {
            std::fs::read_to_string(&identity_path)?
        } else {
            let identity = uuid::Uuid::now_v7().to_string();
            let mut file = OpenOptions::new().write(true).create_new(true).open(identity_path)?;
            file.write_all(identity.as_bytes())?;
            file.sync_all()?;
            identity
        };
        uuid::Uuid::parse_str(&identity).map_err(|_| Error::Incompatible(root.display().to_string()))?;
        let evidence_history: Option<Vec<u8>> =
            connection.query_row("SELECT payload FROM administration WHERE id='evidence-read-history'", [], |row| row.get(0)).optional()?;
        if let Some(history) = evidence_history {
            let history = History::decode(history.as_slice())?;
            if history.store != identity || uuid::Uuid::parse_str(&history.epoch).is_err() {
                return Err(Error::Incompatible(root.display().to_string()));
            }
        } else {
            let populated: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM evidence)", [], |row| row.get(0))?;
            if populated {
                return Err(Error::Incompatible(root.display().to_string()));
            }
            let history = History {
                store: identity.clone(),
                epoch: uuid::Uuid::now_v7().to_string(),
            };
            connection.execute("INSERT INTO administration VALUES('evidence-read-history',?1)", [history.encode_to_vec()])?;
        }
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            database: Arc::new(database),
            identity: identity.into(),
            cancelled: None,
            statistics: Arc::new(Mutex::new(HashMap::new())),
            verified: Arc::new(Mutex::new(HashMap::new())),
            changed: tokio::sync::watch::channel(0).0,
        })
    }
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        if self.cancelled.as_ref().is_some_and(|flag| flag.load(Ordering::Acquire)) {
            return Err(Error::Invalid("Read Storage: The Query Was Cancelled; Start a New Read".into()));
        }
        let connection =
            self.connection.lock().map_err(|_| Error::Invalid("Restart the Host After the Evidence Store Lock Failed".into()))?;
        if self.cancelled.as_ref().is_some_and(|flag| flag.load(Ordering::Acquire)) {
            return Err(Error::Invalid("Read Storage: The Query Was Cancelled; Start a New Read".into()));
        }
        Ok(connection)
    }
    pub(crate) fn prepare_projection(&self, sql: &str) -> Result<()> {
        if self.cancelled.is_none() {
            return Err(Error::Invalid("Read Plot: Open an Isolated Read Session Before Projecting Expressions".into()));
        }
        self.lock()?.execute_batch(sql)?;
        Ok(())
    }
    pub fn read_session(&self) -> Result<(Self, ReadCancellation)> {
        let connection = Connection::open_with_flags(self.database.as_path(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        configure_read_functions(&connection)?;
        connection.execute_batch("BEGIN")?;
        let interrupt = connection.get_interrupt_handle();
        let cancelled = Arc::new(AtomicBool::new(false));
        let session = Self {
            connection: Arc::new(Mutex::new(connection)),
            database: self.database.clone(),
            identity: self.identity.clone(),
            cancelled: Some(cancelled.clone()),
            statistics: self.statistics.clone(),
            verified: self.verified.clone(),
            changed: self.changed.clone(),
        };
        Ok((session, ReadCancellation { cancelled, interrupt }))
    }
    pub fn create(&self, execution: &Execution) -> Result<()> {
        self.create_executions(std::slice::from_ref(execution))
    }
    pub fn create_executions(&self, executions: &[Execution]) -> Result<()> {
        for execution in executions {
            validate_execution(execution)?;
        }
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        for execution in executions {
            transaction.execute(
                "INSERT INTO executions VALUES (?1,?2,0,?3,X'')",
                params![execution.identifier, execution.cycle, execution.encode_to_vec()],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }
    pub fn cycle_is_sealed(&self, cycle: &str) -> Result<bool> {
        Ok(self.lock()?.query_row(
            "SELECT EXISTS(SELECT 1 FROM executions WHERE cycle=?1) AND NOT EXISTS(SELECT 1 FROM executions WHERE cycle=?1 AND state!=2)",
            [cycle],
            |row| row.get(0),
        )?)
    }
    pub fn cycle_execution_page(&self, cycle: &str, after: &str, limit: NonZeroUsize) -> Result<Vec<String>> {
        let connection = self.lock()?;
        let mut query = connection.prepare("SELECT id FROM executions WHERE cycle=?1 AND id>?2 ORDER BY id LIMIT ?3")?;
        Ok(query.query_map(params![cycle, after, limit.get() as i64], |row| row.get(0))?.collect::<std::result::Result<_, _>>()?)
    }
    pub fn identifiers(&self) -> Result<Vec<String>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id FROM executions ORDER BY rowid")?;
        Ok(statement.query_map([], |row| row.get(0))?.collect::<std::result::Result<_, _>>()?)
    }
    pub fn save_plugin(&self, identifier: &str, description: &[u8]) -> Result<()> {
        self.lock()?.execute(
            "INSERT INTO plugins VALUES (?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
            params![identifier, description],
        )?;
        Ok(())
    }
    pub fn plugin(&self, identifier: &str) -> Result<Vec<u8>> {
        Ok(self.lock()?.query_row("SELECT payload FROM plugins WHERE id=?1", [identifier], |row| row.get(0))?)
    }
    pub fn plugin_identifiers(&self) -> Result<Vec<String>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id FROM plugins ORDER BY id")?;
        Ok(statement.query_map([], |row| row.get(0))?.collect::<std::result::Result<_, _>>()?)
    }
    pub fn cycle_outcomes(&self, cycle: &str) -> Result<Vec<v1::ExecutionOutcome>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT payload FROM executions WHERE cycle=?1 ORDER BY id")?;
        let bytes = statement.query_map([cycle], |row| row.get::<_, Vec<u8>>(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
        let mut outcomes = Vec::new();
        for payload in bytes {
            let execution = Execution::decode(payload.as_slice())?;
            outcomes.push(execution.outcome.unwrap_or_default());
        }
        Ok(outcomes)
    }
    pub fn remove_plugin(&self, identifier: &str) -> Result<()> {
        self.lock()?.execute("DELETE FROM plugins WHERE id=?1", [identifier])?;
        Ok(())
    }
    pub fn export(&self, execution: &str) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.write_execution_archive(execution, &mut bytes)?;
        Ok(bytes)
    }

    pub fn require_run_open(&self, execution: &str) -> Result<()> {
        let connection = self.lock()?;
        require_open(&connection, execution)?;
        let payload: Vec<u8> = connection.query_row("SELECT payload FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
        let execution = Execution::decode(payload.as_slice())?;
        if !matches!(execution.origin.and_then(|origin| origin.kind), Some(v1::origin::Kind::PluginRun(_))) {
            return Err(Error::Invalid("Use a Live PluginRun Execution for This Host Call".into()));
        }
        Ok(())
    }
    pub fn append(&self, execution: &str, mut evidence: Evidence) -> Result<u64> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        insert_evidence(&transaction, execution, &mut evidence)?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(evidence.position)
    }
    pub fn append_requested(
        &self, execution: &str, operation: &str, request_id: &str, fingerprint: &[u8], mut evidence: Evidence,
    ) -> Result<(u64, Receipt)> {
        use harmos_protocol::v1::{History, Position, Receipt};
        if request_id.is_empty() {
            return Err(Error::Invalid(format!("{operation}: Supply a Request ID")));
        }
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let saved: Option<(Vec<u8>, i64)> = transaction
            .query_row(
                "SELECT fingerprint,position FROM resource_requests WHERE execution=?1 AND resource=?2 AND request=?3",
                params![execution, operation, request_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let position = if let Some((saved, position)) = saved {
            if saved != fingerprint {
                return Err(Error::Conflict(format!("{operation}: Use a New Request ID for Changed Content")));
            }
            position as u64
        } else {
            insert_evidence(&transaction, execution, &mut evidence)?;
            transaction.execute(
                "INSERT INTO resource_requests VALUES(?1,?2,?3,?4,?5)",
                params![execution, operation, request_id, fingerprint, evidence.position as i64],
            )?;
            evidence.position
        };
        transaction.execute("INSERT OR IGNORE INTO read_positions VALUES(?1,?2)", params![execution, position as i64])?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        let durable = Position {
            history: Some(History {
                store: self.identity.to_string(),
                epoch: execution.into(),
            }),
            sequence: position,
        };
        Ok((
            position,
            Receipt {
                request_id: request_id.into(),
                accepted_at: Some(durable.clone()),
                durable_at: Some(durable),
                completed: true,
            },
        ))
    }
    pub fn open_stream(&self, execution: &str, stream: &str) -> Result<()> {
        let connection = self.lock()?;
        require_open(&connection, execution)?;
        connection.execute("INSERT INTO streams VALUES (?1,?2,0)", params![execution, stream])?;
        Ok(())
    }
    pub fn sample(&self, execution: &str, sample: Sample) -> Result<u64> {
        Ok(self.sample_batch(execution, &[sample])?[0])
    }
    pub fn close_streams(&self, execution: &str) -> Result<()> {
        self.lock()?.execute("UPDATE streams SET closed=1 WHERE execution=?1", [execution])?;
        Ok(())
    }
    pub fn streams_closed(&self, execution: &str) -> Result<bool> {
        let connection = self.lock()?;
        let count: i64 =
            connection.query_row("SELECT COUNT(*) FROM streams WHERE execution=?1 AND closed=0", [execution], |row| row.get(0))?;
        Ok(count == 0)
    }
    pub fn close(&self, execution: &str, termination: v1::Termination, outcome: v1::ExecutionOutcome) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        require_open(&transaction, execution)?;
        let open: i64 =
            transaction.query_row("SELECT COUNT(*) FROM streams WHERE execution=?1 AND closed=0", [execution], |row| row.get(0))?;
        if open != 0 {
            return Err(Error::Invalid("Close Execution Streams Before Closing Evidence".into()));
        }
        let payload: Vec<u8> = transaction.query_row("SELECT payload FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
        let mut value = Execution::decode(payload.as_slice())?;
        let imported = matches!(value.origin.as_ref().and_then(|origin| origin.kind.as_ref()), Some(v1::origin::Kind::Import(_)));
        if imported && matches!(outcome.attribution, Some(v1::execution_outcome::Attribution::Evaluated(_))) {
            return Err(Error::Invalid("Close Import: Use an Importer-Declared or Not-Evaluated Outcome".into()));
        }
        if !imported && matches!(outcome.attribution, Some(v1::execution_outcome::Attribution::Declared(_))) {
            return Err(Error::Invalid("Close PluginRun: Use a Host-Evaluated or Not-Evaluated Outcome".into()));
        }
        value.termination = termination as i32;
        value.outcome = Some(outcome);
        transaction.execute("UPDATE executions SET state=1,payload=?2 WHERE id=?1", params![execution, value.encode_to_vec()])?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        self.statistics
            .lock()
            .map_err(|_| Error::Invalid("Close Execution: Restart After the Statistics Cache Failed".into()))?
            .retain(|(identifier, _, _, _), _| identifier != execution);
        Ok(())
    }
    pub fn seal(&self, execution: &str) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let state: i32 = transaction.query_row("SELECT state FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
        if state != 1 {
            return Err(Error::Invalid("Close the Execution Before Sealing It".into()));
        }
        let digest = digest(&transaction, execution)?;
        transaction.execute("UPDATE executions SET state=2,seal=?2 WHERE id=?1", params![execution, digest])?;
        let cycle: String = transaction.query_row("SELECT cycle FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
        crate::history::capture_cycle(&transaction, &cycle, execution)?;
        transaction.commit()?;
        self.verified
            .lock()
            .map_err(|_| Error::Invalid("Seal Execution: Restart After Seal Verification Failed".into()))?
            .insert(execution.into(), digest);
        Ok(())
    }
    pub fn read(&self, execution: &str) -> Result<ExecutionSnapshot> {
        let connection = self.lock()?;
        let (state, payload, seal): (i32, Vec<u8>, Vec<u8>) =
            connection.query_row("SELECT state,payload,seal FROM executions WHERE id=?1", [execution], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
        if (!seal.is_empty() && state != 2) || (state == 2 && digest(&connection, execution)? != seal) {
            return Err(Error::Invalid("Execution Seal Does Not Match Evidence; Restore a Valid Backup".into()));
        }
        let mut evidence_query = connection.prepare("SELECT payload FROM evidence WHERE execution=?1 ORDER BY position")?;
        let evidence_bytes =
            evidence_query.query_map([execution], |row| row.get::<_, Vec<u8>>(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
        let evidence = evidence_bytes.iter().map(|bytes| Evidence::decode(bytes.as_slice()).map_err(Error::from)).collect::<Result<_>>()?;
        let mut samples_query = connection.prepare("SELECT stream,payload FROM samples WHERE execution=?1 ORDER BY position")?;
        let sample_bytes = samples_query
            .query_map([execution], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let samples = sample_bytes.into_iter().map(|(stream, bytes)| streams::decode(stream, bytes)).collect::<Result<_>>()?;
        Ok(ExecutionSnapshot {
            execution: Execution::decode(payload.as_slice())?,
            state: match state {
                0 => ExecutionState::Open,
                1 => ExecutionState::Closed,
                2 => ExecutionState::Sealed,
                _ => return Err(Error::Incompatible("Invalid Execution State".into())),
            },
            evidence,
            samples,
            seal,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn insert_evidence(transaction: &rusqlite::Transaction<'_>, execution: &str, evidence: &mut Evidence) -> Result<()> {
    require_open(transaction, execution)?;
    evidence.position = next_position(transaction, execution)?;
    transaction.execute(
        "INSERT INTO evidence(execution,position,payload) VALUES (?1,?2,?3)",
        params![execution, evidence.position as i64, evidence.encode_to_vec()],
    )?;
    if evidence.kind == "attempt:start" {
        transaction.execute(
            "INSERT INTO attempts VALUES (?1,?2,?3,?4,?5)",
            params![execution, evidence.step, evidence.operation, evidence.attempt, evidence.position as i64],
        )?;
    }
    if let Some(kind) = evidence.kind.strip_prefix("record:") {
        let payload: Vec<u8> = transaction.query_row("SELECT payload FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
        let definition = Execution::decode(payload.as_slice())?;
        let catalog = Catalog::decode(definition.catalog.as_slice())?;
        for section in &catalog.sections {
            if section.r#type != "dokime.v1.RecordContracts" {
                continue;
            }
            let contracts = v1::RecordContracts::decode(section.payload.as_slice())?;
            let Some(contract) =
                contracts.records.iter().find(|contract| contract.metadata.as_ref().is_some_and(|metadata| metadata.identifier == kind))
            else {
                continue;
            };
            let Some(types) = catalog.types.clone() else {
                continue;
            };
            let Ok(pool) = DescriptorPool::from_file_descriptor_set(types) else {
                continue;
            };
            let Some(descriptor) = pool.get_message_by_name(&contract.message) else {
                continue;
            };
            let Ok(record) = RecordsAppendRequest::decode(evidence.payload.as_slice()) else {
                continue;
            };
            let Ok(row) = DynamicMessage::decode(descriptor, record.payload.as_slice()) else {
                continue;
            };
            for (field, number, label) in crate::stream_windows::indexed_fields(&row, "") {
                transaction.execute(
                    "INSERT INTO record_fields VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![execution, kind, field, evidence.position as i64, evidence.source_ns, number, label],
                )?;
            }
        }
    }

    Ok(())
}

fn configure_read_functions(connection: &Connection) -> Result<()> {
    connection.create_scalar_function(
        "dokime_sqrt",
        1,
        rusqlite::functions::FunctionFlags::SQLITE_DETERMINISTIC | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
        |context| Ok(context.get::<Option<f64>>(0)?.map(|value| value.max(0.0).sqrt())),
    )?;
    Ok(())
}
fn validate_execution(execution: &Execution) -> Result<()> {
    if execution.identifier.trim().is_empty() || execution.cycle.trim().is_empty() {
        return Err(Error::Invalid("Create Execution: Supply Execution and Cycle Identities".into()));
    }
    match execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        Some(v1::origin::Kind::PluginRun(run)) => run.validate().map_err(|error| Error::Invalid(format!("Create Execution: {error}")))?,
        Some(v1::origin::Kind::Import(_)) => (),
        None => return Err(Error::Invalid("Create Execution: Supply a Frozen PluginRun or Import Origin".into())),
    }
    if execution.outcome.is_some() || execution.termination != v1::Termination::Unspecified as i32 {
        return Err(Error::Invalid("Create Execution: Attribute Its Outcome When Closing Evidence".into()));
    }
    Ok(())
}

pub(crate) fn require_open(connection: &Connection, execution: &str) -> Result<()> {
    let state: Option<i32> = connection.query_row("SELECT state FROM executions WHERE id=?1", [execution], |row| row.get(0)).optional()?;
    if state != Some(0) {
        return Err(Error::Invalid("Start an Open Execution Before Writing Evidence".into()));
    }
    Ok(())
}
pub(crate) fn next_position(connection: &Connection, execution: &str) -> Result<u64> {
    Ok(connection.query_row("SELECT MAX(COALESCE((SELECT MAX(position) FROM evidence WHERE execution=?1),0),COALESCE((SELECT MAX(position) FROM samples WHERE execution=?1),0))+1", [execution], |row| row.get::<_,i64>(0))? as u64)
}
pub(crate) fn digest(connection: &Connection, execution: &str) -> Result<Vec<u8>> {
    let mut hash = Sha256::new();
    let payload: Vec<u8> = connection.query_row("SELECT payload FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
    hash.update((payload.len() as u64).to_le_bytes());
    hash.update(payload);
    let mut statement = connection.prepare("SELECT position,kind,payload FROM (SELECT position,'evidence' AS kind,payload FROM evidence WHERE execution=?1 UNION ALL SELECT position,'stream:'||stream AS kind,payload FROM samples WHERE execution=?1) ORDER BY position")?;
    for row in statement.query_map([execution], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Vec<u8>>(2)?)))? {
        let (position, kind, bytes) = row?;
        hash.update(position.to_le_bytes());
        hash.update((kind.len() as u64).to_le_bytes());
        hash.update(kind.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    let mut batches = connection.prepare("SELECT position,stream,payload FROM sample_batches WHERE execution=?1 ORDER BY position")?;
    for row in batches.query_map([execution], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Vec<u8>>(2)?)))? {
        let (position, stream, payload) = row?;
        hash.update(position.to_le_bytes());
        hash.update((stream.len() as u64).to_le_bytes());
        hash.update(stream.as_bytes());
        hash.update((payload.len() as u64).to_le_bytes());
        hash.update(payload);
    }
    Ok(hash.finalize().to_vec())
}

/* === Tests === */
