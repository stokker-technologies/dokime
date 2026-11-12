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
use dokime_protocol::v1;

/* === Constants === */

const APPLICATION_ID: i32 = 0x444b4e58;

/* === Macros === */

/* === Type Aliases === */

type StatisticsCache = HashMap<(String, String, String, u64), (u64, crate::StatisticsRead)>;

/* === Structs === */

#[derive(Clone)]
pub struct Store {
    connection: Arc<Mutex<Connection>>,
    pub(crate) statistics: Arc<Mutex<StatisticsCache>>,
    pub(crate) verified: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    pub(crate) changed: tokio::sync::watch::Sender<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root)?;
        let database = root.join("execution.sqlite3");
        let exists = database.exists();
        if !exists && std::fs::read_dir(root)?.next().is_some() {
            return Err(Error::Incompatible(root.display().to_string()));
        }
        let connection = Connection::open(&database)?;
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
            if tables != ["artifact_calls", "artifact_transfers", "content_chunks", "content_objects", "evidence", "executions", "notes", "plugins", "reports", "sample_batches", "sample_fields", "samples", "streams"] {
                return Err(Error::Incompatible(root.display().to_string()));
            }
        } else {
            connection.execute_batch(&format!("PRAGMA application_id={APPLICATION_ID}; PRAGMA user_version=1;
                CREATE TABLE executions(id TEXT PRIMARY KEY, cycle TEXT NOT NULL, state INTEGER NOT NULL, payload BLOB NOT NULL, seal BLOB NOT NULL);
                CREATE TABLE evidence(execution TEXT NOT NULL REFERENCES executions(id), position INTEGER NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(execution,position));
                CREATE TABLE samples(execution TEXT NOT NULL REFERENCES executions(id), position INTEGER NOT NULL, stream TEXT NOT NULL, payload BLOB NOT NULL, source_ns INTEGER NOT NULL, sequence INTEGER NOT NULL, PRIMARY KEY(execution,position));
                CREATE TABLE streams(execution TEXT NOT NULL REFERENCES executions(id), stream TEXT NOT NULL, closed INTEGER NOT NULL, PRIMARY KEY(execution,stream));
                CREATE INDEX samples_stream_position ON samples(execution,stream,position);
                CREATE INDEX samples_stream_time ON samples(execution,stream,source_ns,position);
                CREATE TABLE sample_batches(execution TEXT NOT NULL REFERENCES executions(id), position INTEGER NOT NULL, stream TEXT NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(execution,position));
                CREATE TABLE sample_fields(execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL, position INTEGER NOT NULL, source_ns INTEGER NOT NULL, number REAL, label TEXT, PRIMARY KEY(execution,stream,field,position));
                CREATE INDEX sample_fields_time ON sample_fields(execution,stream,field,source_ns,position,number);
                CREATE TABLE artifact_transfers(execution TEXT NOT NULL REFERENCES executions(id), transfer TEXT NOT NULL, object TEXT NOT NULL, sequence INTEGER NOT NULL, terminal INTEGER NOT NULL, PRIMARY KEY(execution,transfer));
                CREATE TABLE artifact_calls(execution TEXT NOT NULL, transfer TEXT NOT NULL, sequence INTEGER NOT NULL, fingerprint BLOB NOT NULL, receipt BLOB NOT NULL, PRIMARY KEY(execution,transfer,sequence));
                CREATE TABLE content_objects(id TEXT PRIMARY KEY, execution TEXT NOT NULL REFERENCES executions(id), path TEXT NOT NULL, artifact INTEGER NOT NULL, metadata BLOB NOT NULL, length INTEGER NOT NULL, sha256 TEXT NOT NULL, phase INTEGER NOT NULL);
                CREATE TABLE content_chunks(object TEXT NOT NULL REFERENCES content_objects(id), offset INTEGER NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(object,offset));
                CREATE INDEX content_path ON content_objects(execution,path);
                CREATE TABLE notes(id TEXT PRIMARY KEY, cycle TEXT NOT NULL, execution TEXT, receipt BLOB NOT NULL, payload BLOB NOT NULL);
                CREATE TABLE plugins(id TEXT PRIMARY KEY, payload BLOB NOT NULL);
                CREATE TABLE reports(execution TEXT NOT NULL REFERENCES executions(id), id TEXT NOT NULL, payload BLOB NOT NULL, PRIMARY KEY(execution,id));"))?;
        }
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL; PRAGMA journal_mode=WAL;")?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            statistics: Arc::new(Mutex::new(HashMap::new())),
            verified: Arc::new(Mutex::new(HashMap::new())),
            changed: tokio::sync::watch::channel(0).0,
        })
    }
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|_| Error::Invalid("Restart the Host After the Evidence Store Lock Failed".into()))
    }
    pub fn create(&self, execution: &Execution) -> Result<()> {
        if execution.identifier.trim().is_empty() || execution.cycle.trim().is_empty() {
            return Err(Error::Invalid("Create Execution: Supply Execution and Cycle Identities".into()));
        }
        match execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
            Some(v1::origin::Kind::PluginRun(run)) => {
                run.validate().map_err(|error| Error::Invalid(format!("Create Execution: {error}")))?
            }
            Some(v1::origin::Kind::Import(_)) => (),
            None => return Err(Error::Invalid("Create Execution: Supply a Frozen PluginRun or Import Origin".into())),
        }
        if execution.outcome.is_some() || execution.termination != v1::Termination::Unspecified as i32 {
            return Err(Error::Invalid("Create Execution: Attribute Its Outcome When Closing Evidence".into()));
        }
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO executions VALUES (?1,?2,0,?3,X'')",
            params![execution.identifier, execution.cycle, execution.encode_to_vec()],
        )?;
        Ok(())
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
        let mut bytes=Vec::new();
        self.write_execution_archive(execution,&mut bytes)?;
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
        require_open(&transaction, execution)?;
        evidence.position = next_position(&transaction, execution)?;
        transaction
            .execute("INSERT INTO evidence VALUES (?1,?2,?3)", params![execution, evidence.position as i64, evidence.encode_to_vec()])?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(evidence.position)
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

fn require_open(connection: &Connection, execution: &str) -> Result<()> {
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
