/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Format Refusal Contracts

/* === Modules === */

#[path = "../../build/source.rs"]
mod source;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    error::Error as StdError,
    path::{Path, PathBuf},
    time::SystemTime,
};

// Third Party Imports

// Local Imports
use dokime_runtime::{
    config::Config,
    storage::{self, format, sqlite::State},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Files = BTreeMap<PathBuf, (Option<Vec<u8>>, SystemTime)>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn files(root: &Path) -> Files {
    fn collect(root: &Path, directory: &Path, values: &mut Files) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            let metadata = path.metadata().unwrap();
            let bytes = metadata.is_file().then(|| std::fs::read(&path).unwrap());
            values.insert(path.strip_prefix(root).unwrap().to_owned(), (bytes, metadata.modified().unwrap()));
            if metadata.is_dir() {
                collect(root, &path, values);
            }
        }
    }
    let mut values = Files::new();
    collect(root, root, &mut values);
    values
}

fn has_mismatch(error: &(dyn StdError + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(error) = current {
        if matches!(error.downcast_ref::<storage::Error>(), Some(storage::Error::StorageFormatMismatch { .. })) {
            return true;
        }
        current = error.source();
    }
    false
}

/* === Tests === */

#[test]
fn storage_format_normalizes_codec_trivia_and_preserves_encoding_changes() {
    let source = include_str!("../../src/storage/sqlite/data/entry/object.rs");
    let with_comments = format!("//! Documentation\n/* outer /* nested */ comment */\n{source}\n// End\n");
    let with_whitespace = source.replace("pub(super) fn", "pub ( super )\nfn\t");
    assert_eq!(source::digest(source), source::digest(&with_comments));
    assert_eq!(source::digest(source), source::digest(&with_whitespace));
    assert_ne!(source::digest(source), source::digest(&source.replace("to_le_bytes", "to_be_bytes")));
    assert_ne!(source::digest("const NAME: &str = \"a b\";"), source::digest("const NAME: &str = \"ab\";"));
}

#[tokio::test]
async fn storage_format_fresh_root_records_the_writer_before_evidence() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("new-root");
    assert!(!format::check(&root).unwrap());
    assert!(!root.exists());

    let state = State::open(&root).await.unwrap();
    let record: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join(format::RECORD)).unwrap()).unwrap();
    assert_eq!(record["format"], format::IDENTITY);
    assert_eq!(record["writer_build"], format::BUILD);
    state.close().await.unwrap();
}

#[tokio::test]
async fn storage_format_initializes_before_use_and_keeps_the_original_writer() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("local.lock"), b"retained lease").unwrap();
    let state = State::open(root.path()).await.unwrap();
    let path = root.path().join(format::RECORD);
    let mut record: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(record["format"], format::IDENTITY);
    assert_eq!(record["writer_build"], format::BUILD);
    state.close().await.unwrap();
    assert_eq!(std::fs::read(root.path().join("local.lock")).unwrap(), b"retained lease");

    record["writer_build"] = "another-compatible-build".into();
    let previous = serde_json::to_vec(&record).unwrap();
    std::fs::write(&path, &previous).unwrap();
    let state = State::open(root.path()).await.unwrap();
    state.close().await.unwrap();
    assert_eq!(std::fs::read(path).unwrap(), previous);
}

#[tokio::test]
async fn storage_format_refusal_preserves_every_file() {
    for record in
        [Some(br#"{"format":"different-layout","writer_build":"previous-build"}"#.as_slice()), Some(b"invalid record".as_slice()), None]
    {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("evidence")).unwrap();
        std::fs::write(root.path().join("evidence/retained"), [0, 1, 2, 255]).unwrap();
        let database = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
        database.execute_batch("CREATE TABLE evidence(body BLOB NOT NULL); INSERT INTO evidence VALUES (X'000102ff');").unwrap();
        drop(database);
        if let Some(record) = record {
            std::fs::write(root.path().join(format::RECORD), record).unwrap();
        }
        let before = files(root.path());
        let error = State::open(root.path()).await.err().unwrap();
        assert!(matches!(&error, storage::Error::StorageFormatMismatch { root: path, running_build, .. }
            if path == root.path() && *running_build == format::BUILD));
        if record.is_some_and(|bytes| bytes.starts_with(b"{\"format\"")) {
            assert!(matches!(&error, storage::Error::StorageFormatMismatch { writer_build, writer_format, running_format, .. }
                if writer_build.as_ref() == "previous-build" && writer_format.as_deref() == Some("different-layout") && *running_format == format::IDENTITY));
        }
        let error = storage::Backend::primary(root.path().to_owned()).await.err().unwrap();
        assert!(has_mismatch(&error));
        assert_eq!(files(root.path()), before);
    }
}

#[tokio::test]
async fn storage_format_refuses_application_startup_without_writes() {
    for marked in [false, true] {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("evidence"), [0, 1, 2, 255]).unwrap();
        if marked {
            let record = serde_json::json!({"format": "different-layout", "writer_build": "previous-build"});
            std::fs::write(root.path().join(format::RECORD), serde_json::to_vec(&record).unwrap()).unwrap();
        }
        let before = files(root.path());
        let error = dokime_runtime::app::launch(Config::local(root.path().to_owned())).await.err().unwrap();
        assert!(has_mismatch(&error));
        assert_eq!(files(root.path()), before);
    }
}

#[tokio::test]
async fn storage_format_keeps_exclusive_root_ownership() {
    let root = tempfile::tempdir().unwrap();
    let first = State::open(root.path()).await.unwrap();
    let marker = root.path().join(format::RECORD);
    let before = std::fs::read(&marker).unwrap();
    assert!(matches!(State::open(root.path()).await, Err(storage::Error::Owned)));
    assert_eq!(std::fs::read(marker).unwrap(), before);
    first.close().await.unwrap();
    let next = State::open(root.path()).await.unwrap();
    next.close().await.unwrap();
}
