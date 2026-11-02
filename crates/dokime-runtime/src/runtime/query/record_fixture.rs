/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Record Query Integration Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::storage::{StorageCursor, StoredRecord};
use rusqlite::params;
use sha2::{Digest, Sha256};

// Local Imports
use super::*;
use dokime_domain::model::{
    execution::RecordScope as NativeScope,
    identity::wire::Timestamp,
    identity::{Identifier, RecordId, SessionId},
    query::{page::PageRequest, snapshot::Snapshot},
    records::{
        RecordSeverity,
        evidence::{Record, RecordEntry},
    },
    values::Value,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Fixture {
    _root: tempfile::TempDir,
    state: State,
    pub config: QueryConfig,
    pub execution: TestExecutionId,
    pub second: TestExecutionId,
    pub cycle: TestCycleId,
    pub epoch: SessionId,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub async fn close(&self) {
        self.state.close().await.unwrap();
    }

    pub async fn reopen(&mut self) {
        self.state = State::open(self._root.path()).await.unwrap();
        self.state.data_open(None).await.unwrap();
    }

    pub async fn append_member(&self, execution: TestExecutionId) {
        let cycle = self.cycle.to_string();
        self.state.call(move |connection| {
            connection.execute("INSERT INTO query_members VALUES(?1,'000000000000000e',?2)",params![cycle,execution.as_str()])?;
            connection.execute("INSERT INTO query_history SELECT kind,id,4,'000000000000000e',cycle,template,status,archived,missing,time,body FROM query_history WHERE kind='cycle'",[])?;
            connection.execute("UPDATE query_history_head SET revision=4",[])
        }).await.unwrap();
    }

    pub async fn corrupt_record(&self, position: u64) {
        self.state
            .call(move |connection| {
                connection.execute("UPDATE storage_data_entries SET digest=zeroblob(32) WHERE position=?1", [format!("{position:016x}")])
            })
            .await
            .unwrap();
    }

    pub async fn invalidate(&self) {
        self.state.call(|connection| connection.execute("UPDATE query_token_key SET generation='0000000000000001'", [])).await.unwrap();
    }

    pub async fn new(rows: u32) -> Self {
        let root = tempfile::tempdir().unwrap();
        let state = State::open(root.path()).await.unwrap();
        state.data_open(None).await.unwrap();
        let mut settings = crate::config::query_defaults();
        settings.max_page_rows = rows;
        settings.page_rows = rows;
        let config = settings.compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        let fixture = Self {
            _root: root,
            state,
            config,
            execution: TestExecutionId::from_seed("query-record-owner"),
            second: TestExecutionId::from_seed("query-record-second"),
            cycle: TestCycleId::from_seed("query-record-cycle"),
            epoch: SessionId::from_seed("query-record-epoch"),
        };
        let cycle = fixture.cycle.to_string();
        let members = [fixture.execution.to_string(), fixture.second.to_string()];
        fixture.state.call(move |connection| {
            for (offset, (kind, id)) in [("execution", members[0].as_str()), ("execution", members[1].as_str()), ("cycle", cycle.as_str())].into_iter().enumerate() {
                let position = format!("{:016x}", offset + 1);
                connection.execute("INSERT INTO storage_data_entries(position,kind,identity,schema_id,item,body,digest) VALUES(?1,'object',?1,?2,?3,x'00',zeroblob(32))", params![position, format!("dokime.object.{kind}"), id])?;
                connection.execute("INSERT INTO query_history VALUES(?1,?2,?3,?4,?5,NULL,'',0,1,zeroblob(16),x'00')",params![kind,id,offset as i64+1,position,cycle])?;
            }
            for member in members {
                connection.execute("INSERT INTO query_members VALUES(?1,'0000000000000003',?2)",params![cycle,member])?;
            }
            connection.execute("UPDATE storage_data_head SET saved='0000000000000003'", [])?;
            connection.execute("UPDATE query_history_head SET revision=3", [])?;
            Ok(())
        }).await.unwrap();
        fixture
    }

    pub fn request(&self) -> ListRecords {
        ListRecords {
            scope: RecordScope::Execution(self.execution.to_string()),
            record_type: "example.measurement".into(),
            versions: vec![],
            r#where: vec![],
            between: None,
            page: PageRequest { rows: None, bytes: None, after: None },
            at: None,
        }
    }

    pub fn native_scope(&self, execution: &TestExecutionId) -> NativeScope {
        NativeScope {
            test_cycle_id: Some(self.cycle.clone()),
            test_execution_id: Some(execution.clone()),
            ..Default::default()
        }
    }

    pub fn entry(&self, position: u64) -> RecordEntry {
        let record = Record::new(
            RecordId::from_seed(format!("query-row-{position}")),
            dokime_domain::model::time::Timestamp::from_datetime(
                time::OffsetDateTime::UNIX_EPOCH + time::Duration::nanoseconds(position as i64),
            ),
            Identifier::new("example.measurement").unwrap(),
            1,
            "observer.fixture",
            RecordSeverity::Warning,
        );
        let mut entry = RecordEntry::from_record(position, record);
        entry.ordinal = position;
        entry.scope = Some(self.native_scope(&self.execution));
        entry.details.insert("counter.value".into(), Value::Unsigned(position));
        entry
    }

    pub async fn insert(&self, position: u64, epoch: &SessionId, entry: RecordEntry) {
        let history = self.state.data_history().await.unwrap();
        let epoch = epoch.clone();
        self.state
            .call(move |connection| {
                let stored = StoredRecord {
                    position: StorageCursor { history, sequence: position },
                    schema: crate::storage::records::schema(),
                    scope: Some(Identifier::new(epoch.as_str()).unwrap()),
                    item: Identifier::new(entry.record_id.as_str()).unwrap(),
                    source: None,
                    source_time: None,
                    payload: crate::storage::records::encode(&entry, usize::MAX).unwrap(),
                };
                let body = serde_json::to_vec(&stored).unwrap();
                let digest = Sha256::digest(&body);
                let position = format!("{position:016x}");
                connection.execute(
                    "INSERT INTO storage_data_entries(position,kind,identity,item,scope,body,digest) VALUES(?1,'record',?2,?2,?3,?4,?5)",
                    params![position, entry.record_id.as_str(), epoch.as_str(), body, digest.as_slice()],
                )?;
                let scope = entry.scope.as_ref();
                connection.execute(
                    "INSERT INTO storage_record_visibility VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        position,
                        entry.record_id.as_str(),
                        format!("{:016x}", entry.ordinal),
                        scope.and_then(|s| s.test_cycle_id.as_ref()).map(|id| id.as_str()),
                        scope.and_then(|s| s.test_execution_id.as_ref()).map(|id| id.as_str()),
                        format!("{:016x}", entry.sequence),
                        epoch.as_str()
                    ],
                )?;
                for scope in &entry.captured_scopes {
                    connection.execute(
                        "INSERT INTO storage_record_captures VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                        params![
                            scope.test_execution_id.as_ref().unwrap().as_str(),
                            position,
                            entry.record_id.as_str(),
                            format!("{:016x}", entry.ordinal),
                            serde_json::to_string(scope).unwrap(),
                            scope.test_cycle_id.as_ref().map(|id| id.as_str()),
                            format!("{:016x}", entry.sequence),
                            epoch.as_str()
                        ],
                    )?;
                }
                connection.execute("UPDATE storage_data_head SET saved=MAX(saved,?1)", [position])?;
                Ok(())
            })
            .await
            .unwrap();
    }

    pub async fn snapshot(&self, scope: Scope, through: u64, journal: Option<u64>, records: Vec<RecordContract>) -> QuerySnapshot {
        let history = self.state.data_history().await.unwrap();
        let contracts =
            serde_json::to_vec(&serde_json::json!({"records": records, "telemetry": [], "record_epoch": self.epoch.as_str()})).unwrap();
        let cut = self
            .state
            .snapshot_capture(
                scope.clone(),
                Some(StorageCursor { history, sequence: through }),
                Some(3),
                Timestamp { seconds: 4_000_000_000, nanos: 0 },
                contracts,
                journal,
            )
            .await
            .unwrap()
            .unwrap();
        let tokens = Tokens {
            key: self.state.snapshot_key().await.unwrap(),
            maximum: self.config.limits().token_bytes,
        };
        let token = tokens.encode(&Token::Snapshot(cut.clone()), ListRecords::ID).unwrap();
        QuerySnapshot {
            snapshot: Snapshot {
                token,
                identity: tokens::identity(&cut),
                expires_at: Some(cut.expires),
            },
            cut,
            scope,
            after: None,
        }
    }

    pub async fn read(&self, request: &ListRecords, snapshot: &QuerySnapshot, after: Option<&str>) -> Result<RecordPage, Box<Failure>> {
        let binding = tokens::binding(request, self.config.limits().working_bytes as usize, ListRecords::ID)?;
        let mut snapshot = QuerySnapshot {
            snapshot: snapshot.snapshot.clone(),
            cut: snapshot.cut.clone(),
            scope: snapshot.scope.clone(),
            after: None,
        };
        if let Some(after) = after {
            let tokens = Tokens {
                key: self.state.snapshot_key().await.unwrap(),
                maximum: self.config.limits().token_bytes,
            };
            let Token::Continuation { cut, selection, last } = tokens.decode(after, ListRecords::ID)? else {
                panic!("expected continuation")
            };
            assert!(cut == snapshot.cut);
            if selection != binding {
                return Err(Failure::new(Reason::InvalidContinuation, ListRecords::ID, "selection changed"));
            }
            snapshot.after = Some(last);
        }
        let limits = self.config.page_limits(&request.page, usize::MAX, ListRecords::ID)?;
        page(&self.state, &self.config, request, snapshot, limits, binding).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
