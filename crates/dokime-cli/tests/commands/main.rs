/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Write;

// Third Party Imports
use clap::Parser;

// Local Imports
use dokime::client::operations::Error as OperationError;
use dokime_cli::{Cli, Error};

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

pub(crate) async fn json_files_connection() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(b"{\"unexpected\":true}").unwrap();
    let args = Cli::try_parse_from([
        "dokime",
        "operation",
        "call",
        "dokime.runtime.health",
        "--endpoint",
        "invalid-endpoint",
        "--input-file",
        file.path().to_str().unwrap(),
    ])
    .unwrap();
    assert!(matches!(
        args.run().await,
        Err(Error::Operation(OperationError::Input(_)))
    ));

    let args = Cli::try_parse_from([
        "dokime",
        "operation",
        "call",
        "missing.operation",
        "--endpoint",
        "invalid-endpoint",
    ])
    .unwrap();
    assert!(matches!(
        args.run().await,
        Err(Error::Operation(OperationError::Unknown(_)))
    ));

    for arguments in [
        vec!["cycle", "run", "invalid", "--request-key", "run-1"],
        vec![
            "cycle",
            "run",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "--request-key",
            " ",
        ],
        vec!["cycle", "request", "invalid"],
        vec![
            "cycle",
            "request",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "--cycle-id",
            "invalid",
        ],
        vec![
            "cycle",
            "run",
            "--input-file",
            file.path().to_str().unwrap(),
        ],
    ] {
        let args = Cli::try_parse_from(
            std::iter::once("dokime")
                .chain(arguments)
                .chain(["--endpoint", "invalid-endpoint"]),
        )
        .unwrap();
        assert!(matches!(args.run().await, Err(Error::Input(_))));
    }
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
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: json_files_connection");
        run_scenario(json_files_connection, false, None, false);
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
