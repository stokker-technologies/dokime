/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Reports Retain Exact Bytes and Refuse Tampering After Restart

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use dokime_protocol::v1;
use dokime_storage::{Result, Store};

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
fn report_seal_retains_stored_document_bytes() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&dokime_storage::Execution {
        identifier: "report".into(),
        cycle: "report".into(),
        origin: Some(v1::Origin {
            kind: Some(v1::origin::Kind::Import(v1::Import::default())),
        }),
        ..Default::default()
    })?;
    store.close("report", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
    store.seal("report")?;
    let seal = store.cut("report")?.seal;
    let document = v1::Document {
        metadata: Some(v1::Metadata {
            attributes: (0..7)
                .map(|index| {
                    (
                        format!("field-{index}"),
                        v1::Value {
                            kind: Some(v1::value::Kind::Integer(index)),
                        },
                    )
                })
                .collect(),
            ..Default::default()
        }),
        ..Default::default()
    };
    let report = store.publish_report("report", &seal, document, b"<html></html>".to_vec())?;
    drop(store);
    let store = Store::open(root.path())?;
    assert_eq!(store.reports("report")?, vec![report.clone()]);
    let mut altered = report;
    altered.html.push(b'X');
    rusqlite::Connection::open(root.path().join("execution.sqlite3"))?
        .execute("UPDATE reports SET payload=?1 WHERE execution='report'", [altered.encode_to_vec()])?;
    assert!(store.reports("report").is_err());
    Ok(())
}
