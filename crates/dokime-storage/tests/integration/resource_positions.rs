/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Revisioned Resources and Storage Admission

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_protocol::model::Section;
use prost::Message;
use rusqlite::{Connection, params};

// Local Imports
use dokime_protocol::host::ExecutionOutcome;
use dokime_protocol::host::ExecutionsDeleteResponse;
use dokime_protocol::model::Import;
use dokime_protocol::model::Origin;
use dokime_protocol::model::Termination;
use dokime_protocol::model::origin;
use dokime_storage::{Error, Evidence, Execution, Result, Store};

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

#[test]
fn superseded_journals_refuse_admission_without_deleting_their_values() -> Result<()> {
    for key in ["view:resource", "view-at:resource", "prepared-report/resource", "artifact/old", "execution-file/old/log.txt"] {
        // Retain an Old Journal Entry
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let value = Section {
            r#type: "resource".into(),
            payload: vec![0, 128, 255],
        };
        store.save_administration(key, &value)?;
        drop(store);

        // Refuse the Root and Preserve Its Stored Value
        assert!(matches!(Store::open(root.path()), Err(Error::Incompatible(_))));
        let connection = Connection::open(root.path().join("execution.sqlite3"))?;
        let payload: Vec<u8> = connection.query_row("SELECT payload FROM administration WHERE id=?1", [key], |row| row.get(0))?;
        assert_eq!(payload, value.encode_to_vec());
    }
    Ok(())
}

#[test]
fn retired_journals_refuse_occupied_state_and_preserve_empty_roots() -> Result<()> {
    for table in ["commands", "notes"] {
        // Empty Retired Tables Carry No Runtime State and Need No Migration.
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        drop(store);
        let connection = Connection::open(root.path().join("execution.sqlite3"))?;
        connection.execute_batch(&format!("CREATE TABLE {table}(payload BLOB NOT NULL)"))?;
        drop(connection);
        let store = Store::open(root.path())?;
        drop(store);

        // Undecodable Retired State Is Refused Without Altering Its Bytes.
        let payload = vec![0_u8, 128, 255];
        let connection = Connection::open(root.path().join("execution.sqlite3"))?;
        connection.execute(&format!("INSERT INTO {table} VALUES (?1)"), params![payload])?;
        drop(connection);
        assert!(matches!(Store::open(root.path()), Err(Error::Incompatible(_))));
        let connection = Connection::open(root.path().join("execution.sqlite3"))?;
        let saved: Vec<u8> = connection.query_row(&format!("SELECT payload FROM {table}"), [], |row| row.get(0))?;
        assert_eq!(saved, payload);
    }
    Ok(())
}

#[test]
fn issued_positions_and_original_append_receipts_survive_restart() -> Result<()> {
    // Create Two Histories and Append One Idempotent Record.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    for id in ["first", "second"] {
        store.create(&Execution {
            identifier: id.into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(origin::Kind::Import(Import::default())),
            }),
            ..Default::default()
        })?;
    }
    let evidence = Evidence {
        kind: "log".into(),
        payload: vec![7],
        ..Default::default()
    };
    let original = store.append_requested("first", "append", "request", &[1], evidence.clone())?;
    let cut = store.evidence_position(None)?;
    assert_eq!(store.append_requested("first", "append", "request", &[1], evidence.clone())?, original);
    assert!(store.append_requested("first", "append", "request", &[2], evidence).is_err());
    let mut future = cut.clone();
    future.sequence += 1;
    assert!(store.evidence_position(Some(&future)).is_err());
    drop(store);

    // Reopen and Follow a Different Execution Without Reinterpreting the Cut.
    let store = Store::open(root.path())?;
    assert_eq!(store.evidence_position(Some(&cut))?, cut);
    store.append(
        "second",
        Evidence {
            kind: "log".into(),
            payload: vec![9],
            ..Default::default()
        },
    )?;
    let current = store.evidence_position(None)?;
    assert!(current.sequence > cut.sequence);
    assert_eq!(
        store.evidence_entries(dokime_storage::EvidenceOwner::Store, 0, cut.sequence, NonZeroUsize::new(10).unwrap(), false)?.len(),
        1
    );

    // Deleting Sealed Evidence Explicitly Invalidates a Returned Global Cut.
    store.close("first", Termination::Completed, ExecutionOutcome::default())?;
    store.seal("first", Default::default())?;
    let seal = store.cut("first")?.seal;
    store.delete_sealed_execution("delete", "first", &seal, &ExecutionsDeleteResponse::default())?;
    assert!(store.evidence_position(Some(&cut)).is_err());
    let replacement = store.evidence_position(None)?;
    assert_ne!(replacement.history, cut.history);
    Ok(())
}
