/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod directory;

use std::sync::Arc;

use dokime_runtime::artifacts::Artifacts;
use dokime_runtime::transport::MAX_ARTIFACT_CHUNK_BYTES;
use dokime_runtime::transport::recording::RecordingSink;

fn grant(token: &str) -> (Artifacts, Arc<RecordingSink>) {
    let sink = Arc::new(RecordingSink::default());

    (Artifacts::new(token, sink.clone()), sink)
}

fn io_kind(error: dokime_runtime::Error) -> std::io::ErrorKind {
    match error {
        dokime_runtime::Error::Artifact(source) => source.kind(),
        other => panic!("expected an artifact error, got {other}"),
    }
}

#[test]
fn importing_file_whole() {
    let (artifacts, sink) = grant("import-test");
    let content = (0..MAX_ARTIFACT_CHUNK_BYTES + 4096).map(|index| (index % 253) as u8).collect::<Vec<_>>();
    let directory = tempfile::tempdir().expect("a temporary directory is available");
    let source = directory.path().join("recorded.csv");
    std::fs::write(&source, &content).unwrap();

    artifacts.import(&source, "evidence/imported.csv").expect("an existing file streams in");

    let chunks = sink.chunks();
    assert_eq!(chunks.len(), 2);
    assert!(chunks.iter().all(|chunk| chunk.mime == "text/csv"));
    assert!(chunks[1].last);
    assert_eq!(sink.assembled("evidence/imported.csv"), content);
}

#[test]
fn importing_missing_nothing() {
    let (artifacts, sink) = grant("import-missing");
    let directory = tempfile::tempdir().expect("a temporary directory is available");

    let error = artifacts
        .import(directory.path().join("absent.bin"), "evidence/absent.bin")
        .expect_err("a source that is not there cannot be imported");

    assert_eq!(io_kind(error), std::io::ErrorKind::NotFound);
    assert!(sink.chunks().is_empty(), "a failed import never opens a transfer the host would have to discard");
}

#[cfg(unix)]
#[test]
fn import_fails_opened() {
    let (artifacts, sink) = grant("import-unreadable");
    let directory = tempfile::tempdir().expect("a temporary directory is available");

    artifacts.import(directory.path(), "evidence/unreadable.bin").expect_err("a source that cannot be read is not evidence");

    let chunks = sink.chunks();
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].last && chunks[0].aborted, "the host is told to drop the transfer rather than left holding it");
    assert!(chunks[0].bytes.is_empty());
}
