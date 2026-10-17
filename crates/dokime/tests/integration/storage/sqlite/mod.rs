/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg(feature = "storage")]

/* === Modules === */
pub(crate) mod catalog;
pub(crate) mod contract;
pub(crate) mod production;
pub(crate) use crate::support::local_storage as storage;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::storage::sqlite::*;
use rusqlite::Connection;

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

#[tokio::test]
async fn refuses_previous_data() {
    for version in [2, 3, 4, 5] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("local.sqlite");
        let previous = Connection::open(&path).unwrap();
        previous
            .pragma_update(None, "user_version", version)
            .unwrap();
        previous
            .execute_batch(
                "CREATE TABLE evidence(body BLOB NOT NULL);
             INSERT INTO evidence VALUES (X'000102ff');",
            )
            .unwrap();
        drop(previous);

        assert!(matches!(
            State::open(root.path()).await,
            Err(Error::Incompatible(reason)) if reason.contains(&format!("version {version}"))
        ));
        let retained = Connection::open(&path).unwrap();
        assert_eq!(
            retained
                .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
                .unwrap(),
            version
        );
        let bytes: Vec<u8> = retained
            .query_row("SELECT body FROM evidence", [], |row| row.get(0))
            .unwrap();
        assert_eq!(bytes, [0, 1, 2, 255]);
    }
}

#[tokio::test]
async fn refuses_unversioned_schema() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("local.sqlite");
    let existing = Connection::open(&path).unwrap();
    existing
        .execute_batch(
            "CREATE TABLE file_cleanup(storage_key TEXT PRIMARY KEY);
             INSERT INTO file_cleanup VALUES ('retained');",
        )
        .unwrap();
    drop(existing);

    assert!(matches!(
        State::open(root.path()).await,
        Err(Error::Incompatible(_))
    ));

    let retained = Connection::open(&path).unwrap();
    let version: i64 = retained
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 0);
    let created: i64 = retained
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE name IN ('storage_sources', 'storage_sources',
                 'storage_deletions', 'storage_operations')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(created, 0);
    let evidence: String = retained
        .query_row("SELECT storage_key FROM file_cleanup", [], |row| row.get(0))
        .unwrap();
    assert_eq!(evidence, "retained");
    retained.execute_batch("DROP TABLE file_cleanup").unwrap();
    drop(retained);

    let owner = State::open(root.path()).await.unwrap();
    owner.close().await.unwrap();
    let installed = Connection::open(&path).unwrap();
    let version: i64 = installed
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 1);
}

#[tokio::test]
async fn refuses_retired_intent() {
    let root = tempfile::tempdir().unwrap();
    let owner = State::open(root.path()).await.unwrap();
    owner.close().await.unwrap();
    let path = root.path().join("local.sqlite");
    let existing = Connection::open(&path).unwrap();
    existing
        .execute_batch(
            "CREATE TABLE storage_operations(identifier TEXT, state TEXT, attempt INTEGER);
        INSERT INTO storage_operations VALUES ('original', 'running', 7);",
        )
        .unwrap();
    drop(existing);
    assert!(matches!(
        State::open(root.path()).await,
        Err(Error::Incompatible(_))
    ));
    let retained = Connection::open(&path).unwrap();
    let claim: (String, i64) = retained
        .query_row(
            "SELECT state,attempt FROM storage_operations WHERE identifier='original'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(claim, ("running".into(), 7));
}

#[tokio::test]
async fn refuses_attachment_receipts() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("local.sqlite");
    let owner = State::open(root.path()).await.unwrap();
    owner
        .call(|connection| connection.execute_batch("DROP TABLE storage_sources"))
        .await
        .unwrap();
    owner.close().await.unwrap();
    let previous = Connection::open(&path).unwrap();
    previous.execute_batch("PRAGMA user_version=3; CREATE TABLE evidence(value TEXT); INSERT INTO evidence VALUES ('retained');").unwrap();
    drop(previous);

    assert!(matches!(
        State::open(root.path()).await,
        Err(Error::Incompatible(reason)) if reason.contains("version 3")
    ));

    let previous = Connection::open(&path).unwrap();
    assert_eq!(
        previous
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        previous
            .query_row("SELECT value FROM evidence", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "retained"
    );
    assert_eq!(
        previous
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'storage_sources'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn refuses_second_drop() {
    let root = tempfile::tempdir().unwrap();
    let state = State::open(root.path()).await.unwrap();
    assert!(matches!(State::open(root.path()).await, Err(Error::Owned)));
    let shared = state.clone();
    drop(state);
    assert!(matches!(State::open(root.path()).await, Err(Error::Owned)));
    drop(shared);
    State::open(root.path()).await.unwrap();
}

#[tokio::test]
async fn sqlite_rolls_reopen() {
    let root = tempfile::tempdir().unwrap();
    let state = State::open(root.path()).await.unwrap();
    state.call(|connection| connection.execute_batch(
        "CREATE TABLE operations(identifier TEXT PRIMARY KEY); INSERT INTO operations VALUES ('committed');"
    )).await.unwrap();
    let result = state
        .call(|connection| {
            let transaction = connection.transaction()?;
            transaction.execute("INSERT INTO operations VALUES ('rolled-back')", [])?;
            transaction.execute("INSERT INTO operations VALUES ('committed')", [])?;
            transaction.commit()
        })
        .await;
    assert!(matches!(result, Err(Error::Sqlite(_))));
    drop(state);

    let reopened = State::open(root.path()).await.unwrap();
    let identifiers = reopened
        .call(|connection| {
            connection
                .prepare("SELECT identifier FROM operations")?
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .await
        .unwrap();
    assert_eq!(identifiers, ["committed"]);
}

#[tokio::test]
async fn cancelled_caller_progress() {
    let root = tempfile::tempdir().unwrap();
    let state = State::open(root.path()).await.unwrap();
    state
        .call(|connection| {
            connection.execute_batch("CREATE TABLE writes(sequence INTEGER PRIMARY KEY)")
        })
        .await
        .unwrap();
    let (started, running) = tokio::sync::oneshot::channel();
    let (release, blocked) = std::sync::mpsc::channel();
    let first = state.clone();
    let task = tokio::spawn(async move {
        first
            .call(move |connection| {
                started.send(()).unwrap();
                blocked.recv().unwrap();
                connection.execute("INSERT INTO writes VALUES (1)", [])
            })
            .await
    });
    running.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());

    let mut second = Box::pin(state.call(|connection| {
        // The prior write must complete before this callback can run.
        assert_eq!(
            connection.query_row("SELECT COUNT(*) FROM writes", [], |row| row
                .get::<_, i64>(0))?,
            1
        );
        connection.execute("INSERT INTO writes VALUES (2)", [])
    }));
    std::future::poll_fn(|context| {
        assert!(std::future::Future::poll(second.as_mut(), context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    release.send(()).unwrap();
    second.await.unwrap();
}

#[tokio::test]
async fn close_drains_clones() {
    use std::future::Future;

    let root = tempfile::tempdir().unwrap();
    let state = State::open(root.path()).await.unwrap();
    let shared = state.clone();
    let entered = std::sync::Arc::new(tokio::sync::Notify::new());
    let (release, gate) = std::sync::mpsc::channel();
    let signal = entered.clone();
    let writer = state.clone();
    let accepted = tokio::spawn(async move {
        writer
            .call(move |connection| {
                signal.notify_one();
                gate.recv().unwrap();
                connection.execute_batch("CREATE TABLE closed_probe(value TEXT);")
            })
            .await
    });
    // The accepted local call starts.
    entered.notified().await;

    // Poll close until it waits behind the admitted SQLite work, then cancel
    // that wait. Both the operation and the live lease remain owned.
    {
        let closing = state.close();
        tokio::pin!(closing);
        std::future::poll_fn(|context| {
            assert!(closing.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
    }
    assert!(matches!(shared.call(|_| Ok(())).await, Err(Error::Closed)));
    assert!(matches!(State::open(root.path()).await, Err(Error::Owned)));

    release.send(()).unwrap();
    accepted.await.unwrap().unwrap();
    state.close().await.unwrap();
    shared.close().await.unwrap();
    assert!(matches!(shared.call(|_| Ok(())).await, Err(Error::Closed)));

    let reopened = State::open(root.path()).await.unwrap();
    reopened
        .call(|connection| connection.execute_batch("DROP TABLE closed_probe;"))
        .await
        .unwrap();
    reopened.close().await.unwrap();
}
