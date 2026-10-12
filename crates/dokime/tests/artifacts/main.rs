/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod directory;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime::artifacts::Artifacts;
use dokime::transport::MAX_ARTIFACT_CHUNK_BYTES;
use dokime::transport::recording::RecordingSink;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn grant(token: &str) -> (Artifacts, Arc<RecordingSink>) {
    let sink = Arc::new(RecordingSink::default());

    (Artifacts::new(token, sink.clone()), sink)
}

fn io_kind(error: dokime::Error) -> std::io::ErrorKind {
    match error {
        dokime::Error::Artifact(source) => source.kind(),
        other => panic!("expected an artifact error, got {other}"),
    }
}

/* === Tests === */

pub(crate) fn importing_file_whole() {
    let (artifacts, sink) = grant("import-test");
    let content = (0..MAX_ARTIFACT_CHUNK_BYTES + 4096)
        .map(|index| (index % 253) as u8)
        .collect::<Vec<_>>();
    let directory = tempfile::tempdir().expect("a temporary directory is available");
    let source = directory.path().join("recorded.csv");
    std::fs::write(&source, &content).unwrap();

    artifacts
        .import(&source, "evidence/imported.csv")
        .expect("an existing file streams in");

    let chunks = sink.chunks();
    assert_eq!(chunks.len(), 2);
    assert!(chunks.iter().all(|chunk| chunk.mime == "text/csv"));
    assert!(chunks[1].last);
    assert_eq!(sink.assembled("evidence/imported.csv"), content);
}

pub(crate) fn importing_missing_nothing() {
    let (artifacts, sink) = grant("import-missing");
    let directory = tempfile::tempdir().expect("a temporary directory is available");

    let error = artifacts
        .import(directory.path().join("absent.bin"), "evidence/absent.bin")
        .expect_err("a source that is not there cannot be imported");

    assert_eq!(io_kind(error), std::io::ErrorKind::NotFound);
    assert!(
        sink.chunks().is_empty(),
        "a failed import never opens a transfer the host would have to discard"
    );
}

#[cfg(unix)]
pub(crate) fn import_fails_opened() {
    let (artifacts, sink) = grant("import-unreadable");
    let directory = tempfile::tempdir().expect("a temporary directory is available");

    artifacts
        .import(directory.path(), "evidence/unreadable.bin")
        .expect_err("a source that cannot be read is not evidence");

    let chunks = sink.chunks();
    assert_eq!(chunks.len(), 1);
    assert!(
        chunks[0].last && chunks[0].aborted,
        "the host is told to drop the transfer rather than left holding it"
    );
    assert!(chunks[0].bytes.is_empty());
}

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let _ = multithread;
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    builder.start_paused(paused);
    builder.build().expect("scenario runtime")
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: directory::file_sent_piece");
        run_scenario(directory::file_sent_piece, false, None, true);
    }
    {
        eprintln!("Scenario: directory::file_shrank_had");
        run_scenario(directory::file_shrank_had, false, None, true);
    }
    {
        eprintln!("Scenario: directory::rewrite_same_time");
        run_scenario(directory::rewrite_same_time, false, None, true);
    }
    {
        eprintln!("Scenario: directory::file_past_skipped");
        run_scenario(directory::file_past_skipped, false, None, true);
    }
    {
        eprintln!("Scenario: directory::file_outgrows_opened");
        run_scenario(directory::file_outgrows_opened, false, None, true);
    }
    {
        eprintln!("Scenario: directory::name_refuses_directory");
        run_scenario(directory::name_refuses_directory, false, None, true);
    }
    {
        eprintln!("Scenario: directory::sweep_finalizes_directory");
        run_scenario(directory::sweep_finalizes_directory, false, None, true);
    }
    {
        eprintln!("Scenario: directory::one_name_refused");
        run_scenario(directory::one_name_refused, false, None, true);
    }
    {
        eprintln!("Scenario: directory::one_directory_cleanup");
        run_scenario(directory::one_directory_cleanup, false, None, true);
    }
    {
        eprintln!("Scenario: directory::closed_backlog_tick");
        run_scenario(directory::closed_backlog_tick, false, None, true);
    }
    {
        eprintln!("Scenario: importing_file_whole");
        importing_file_whole();
    }
    {
        eprintln!("Scenario: importing_missing_nothing");
        importing_missing_nothing();
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: import_fails_opened");
        import_fails_opened();
    }
}

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
