/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SQLite History Recovery

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::model::{
    execution::{
        result,
        wire::{self},
    },
    history::wire::ExecutionAvailability,
    identity::wire::Timestamp,
    query::{
        page::ExecutionSummary,
        selection::{CycleSelection, ExecutionSelection},
    },
};
use dokime_runtime::storage::sqlite::rusqlite::{Connection, params};
use dokime_runtime::storage::sqlite::testing::*;
use harmos::{Message, storage::ObjectEntry};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

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
                position(revision as u64),
                serde_json::to_string(&status).unwrap(),
                time.is_none(),
                time_key(row.started_at),
                serde_json::to_vec(&row).unwrap()
            ],
        )
        .unwrap();
    connection.execute("UPDATE query_history_head SET revision=MAX(revision,?1)", [revision]).unwrap();
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
    let native = dokime_runtime::storage::objects::cycle::Cycle::try_from(&cycle).unwrap();
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
                version: <dokime_runtime::storage::objects::cycle::Cycle as harmos::Identified>::definition().1.try_into().unwrap(),
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
    assert_eq!(connection.query_row("SELECT execution FROM query_members", [], |row| row.get::<_, String>(0)).unwrap(), member.to_string());

    insert(&connection, member.as_str(), 2, Some(10), wire::ExecutionStatus::Running);
    connection.execute("UPDATE query_history SET cycle=?1 WHERE kind='execution'", [id.as_str()]).unwrap();
    cycle.lifecycle = result::CycleStatus::Running;
    object.position.sequence = 3;
    object.version = ObjectVersionToken::new(object.position.clone());
    object.value.payload = dokime_runtime::storage::objects::cycle::Cycle::try_from(&cycle).unwrap().encode();
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
    let row = WorkRow { body: serde_json::to_vec(&work).unwrap() };
    let record = harmos::storage::StoredRecord {
        position: StorageCursor { sequence: 5, ..object.position.clone() },
        schema: work_schema(),
        scope: None,
        item: Identifier::new("work.fixture").unwrap(),
        source: None,
        source_time: None,
        payload: row.encode(),
    };
    {
        let transaction = connection.transaction().unwrap();
        index_work(&transaction, &record).unwrap();
    }
    assert_eq!(cycles(&connection, &selection, &read(5)).unwrap().rows[0].running_count, 0);
    {
        let transaction = connection.transaction().unwrap();
        index_work(&transaction, &record).unwrap();
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
