/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History Revision, Recovery, Archive And Query Plan Proofs

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
    let connection = Connection::open(&database).unwrap();
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
    let sealed = CycleSelection {
        phases: vec![result::CyclePhase::Sealed],
        ..selection
    };
    assert!(cycles(&connection, &sealed, &read(4)).unwrap().rows.is_empty());
}
