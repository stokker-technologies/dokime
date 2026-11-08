/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SQLite Reader Isolation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::storage::sqlite::testing::*;
use rusqlite::Connection;
use std::num::NonZeroUsize;

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
async fn metadata_reads_the_committed_cut_while_the_writer_is_busy() {
    let root = tempfile::tempdir().unwrap();
    let state = State::open_with(root.path(), dokime_runtime::config::local_storage_defaults()).await.unwrap();
    let initial = state.data_open(None).await.unwrap();
    state.initialize_settings().await.unwrap();
    let settings = state.read_application_settings().await.unwrap();
    let sources = state.sources().await.unwrap();
    #[cfg(feature = "native")]
    let metadata = state.read_application_metadata().await.unwrap();
    let mut writer = state.writer().await;
    let transaction = writer.transaction().unwrap();
    transaction.execute("UPDATE storage_data_head SET saved=?1 WHERE singleton=1", [position(initial.sequence + 1)]).unwrap();
    transaction.execute("UPDATE application_settings SET revision=revision+1", []).unwrap();
    transaction
        .execute("INSERT INTO storage_sources SELECT 'pending','fixture',COALESCE(MAX(priority),-1)+1,1 FROM storage_sources", [])
        .unwrap();
    #[cfg(feature = "native")]
    transaction.execute("DELETE FROM application_metadata WHERE id=1", []).unwrap();

    let saved = state.data_saved().await.unwrap();
    assert_eq!(saved.history, initial.history);
    assert_eq!(saved.sequence, initial.sequence);
    assert_eq!(saved.retention_generation, initial.retention_generation);
    assert_eq!(state.read_application_settings().await.unwrap(), settings);
    assert_eq!(state.sources().await.unwrap(), sources);
    #[cfg(feature = "native")]
    assert_eq!(state.read_application_metadata().await.unwrap(), metadata);

    transaction.rollback().unwrap();
    drop(writer);
    state.close().await.unwrap();
}

// Every Reader Holds An Open Snapshot While The Writer Commits 100 Rows: The Commit
// Never Waits For Them, And Each Held Snapshot Still Reads Its Original Cut.
#[tokio::test]
async fn readers_never_hold_the_writer() {
    let root = tempfile::tempdir().unwrap();
    let readers = NonZeroUsize::new(4).unwrap();
    let settings = Settings {
        previews: dokime_runtime::config::local_storage_defaults().previews,
        readers,
        ..dokime_runtime::config::local_storage_defaults()
    };
    let state = State::open_with(root.path(), settings).await.unwrap();
    state.call(|connection| connection.execute_batch("CREATE TABLE samples(value INTEGER NOT NULL)")).await.unwrap();

    let (entered, mut entries) = tokio::sync::mpsc::unbounded_channel();
    let mut releases = Vec::new();
    let mut held = Vec::new();
    for _ in 0..readers.get() {
        let (release, released) = std::sync::mpsc::channel::<()>();
        releases.push(release);
        let entered = entered.clone();
        let state = state.clone();
        held.push(tokio::spawn(async move {
            state
                .read(move |connection| {
                    let count =
                        |connection: &Connection| connection.query_row("SELECT count(*) FROM samples", [], |row| row.get::<_, i64>(0));
                    let before = count(connection)?;
                    entered.send(()).unwrap();
                    released.recv().unwrap();
                    Ok((before, count(connection)?))
                })
                .await
                .unwrap()
        }));
    }
    for _ in 0..readers.get() {
        entries.recv().await.unwrap();
    }

    let saved = state
        .call(|connection| {
            let transaction = connection.transaction()?;
            for value in 0..100 {
                transaction.execute("INSERT INTO samples VALUES (?1)", [value])?;
            }
            transaction.commit()?;
            connection.query_row("SELECT count(*) FROM samples", [], |row| row.get::<_, i64>(0))
        })
        .await
        .unwrap();
    assert_eq!(saved, 100);

    for release in releases {
        release.send(()).unwrap();
    }
    for reader in held {
        assert_eq!(reader.await.unwrap(), (0, 0));
    }
    let fresh = state.read(|connection| connection.query_row("SELECT count(*) FROM samples", [], |row| row.get::<_, i64>(0))).await;
    assert_eq!(fresh.unwrap(), 100);
    state.close().await.unwrap();
    assert!(matches!(state.read(|_| Ok(())).await, Err(Error::Closed)));
}
