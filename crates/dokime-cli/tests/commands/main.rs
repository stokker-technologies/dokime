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
use dokime_cli::{Cli, Error};
use dokime_client::profiles::Store;

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
    assert!(Cli::try_parse_from(["dokime", "serve"]).is_err());
    for arguments in [
        vec![
            "plan",
            "preview",
            "--template",
            "checks.probe",
            "--case-id",
            "case",
        ],
        vec![
            "plan",
            "preview",
            "--input-file",
            "request.json",
            "--case-id",
            "case",
        ],
        vec![
            "plan",
            "preview",
            "--case-id",
            "case",
            "--selection-json",
            "{}",
        ],
        vec![
            "plan",
            "register",
            "--input-file",
            "request.json",
            "--request-key",
            "plan-1",
        ],
        vec![
            "cycle",
            "create",
            "--input-file",
            "request.json",
            "--label",
            "Override",
        ],
        // A cycle needs a label; waiting needs a run request.
        vec!["cycle", "create"],
        vec!["cycle", "create", "--label", "Test", "--wait-seconds", "5"],
        // Appending names an existing cycle, or reads it from the request file.
        vec!["cycle", "append", "--template", "checks.probe"],
        vec![
            "cycle",
            "append",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "--input-file",
            "request.json",
        ],
        vec![
            "cycle",
            "append",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "--input-file",
            "request.json",
            "--template",
            "checks.probe",
        ],
    ] {
        assert!(Cli::try_parse_from(std::iter::once("dokime").chain(arguments)).is_err());
    }
    // Dynamic Calls Accept Providers Absent From The Compiled Product Catalog.
    assert!(
        Cli::try_parse_from([
            "dokime",
            "operation",
            "call",
            "external.plugin.exchange",
            "--owner",
            "external-plugin",
            "--input-stream-file",
            "frames.jsonl",
            "--json",
            "--endpoint",
            "http://127.0.0.1:8765",
        ])
        .is_ok()
    );
    assert!(
        Cli::try_parse_from([
            "dokime",
            "operation",
            "show",
            "external.plugin.exchange",
            "--owner",
            "external-plugin",
            "--json",
        ])
        .is_ok()
    );

    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(b"{broken-json").unwrap();
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
    assert!(matches!(args.run().await, Err(Error::Input(_))));

    let args = Cli::try_parse_from([
        "dokime",
        "operation",
        "call",
        "missing.operation",
        "--endpoint",
        "invalid-endpoint",
    ])
    .unwrap();
    assert!(matches!(args.run().await, Err(Error::Client(_))));

    let args = Cli::try_parse_from([
        "dokime",
        "operation",
        "call",
        "dokime.evidence.watch_records",
        "--endpoint",
        "invalid-endpoint",
    ])
    .unwrap();
    let refused = args.run().await.unwrap_err();
    assert_eq!(refused.exit_code(), std::process::ExitCode::from(2));
    assert!(matches!(refused, Error::Client(_)));

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
        vec!["cycle", "stop", "invalid"],
        vec!["cycle", "rerun", "invalid", "case", "--key", "rerun-1"],
        vec![
            "cycle",
            "rerun",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "invalid",
            "--key",
            "rerun-1",
        ],
        vec![
            "cycle",
            "rerun",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "a13ae695-1f02-4464-9f7b-0b14ecffcb41",
            "--key",
            " ",
        ],
        vec!["execution", "stop", "invalid"],
        vec!["plan", "preview", "--case-id", "invalid"],
        vec![
            "plan",
            "preview",
            "--selection-json",
            "{\"unexpected\":true}",
        ],
        vec![
            "plan",
            "register",
            "--case-id",
            "invalid",
            "--request-key",
            "plan-1",
        ],
        vec![
            "plan",
            "register",
            "--template",
            "checks.probe",
            "--request-key",
            " ",
        ],
        // A template is named by its readable ID, refused before connecting
        // when it does not name a suite and one lowercase segment.
        vec!["plan", "preview", "--template", "probe"],
        vec!["plan", "preview", "--template", "checks.Probe"],
        vec![
            "plan",
            "preview",
            "--selection-json",
            "{\"suite_id\":\"foreign\",\"template_id\":\"checks.probe\",\"parameters\":[],\"files\":[]}",
        ],
        vec![
            "plan",
            "register",
            "--template",
            "0197f000-0000-7000-8000-000000000002",
            "--request-key",
            "plan-1",
        ],
        vec![
            "plan",
            "preview",
            "--input-file",
            file.path().to_str().unwrap(),
        ],
        vec![
            "plan",
            "register",
            "--input-file",
            file.path().to_str().unwrap(),
        ],
        vec![
            "cycle",
            "create",
            "--input-file",
            file.path().to_str().unwrap(),
        ],
        vec![
            "cycle",
            "create",
            "--case-id",
            "invalid",
            "--identifier",
            "test",
            "--label",
            "Test",
        ],
        vec![
            "cycle",
            "create",
            "--label",
            "Test",
            "--identifier",
            "Not An Identifier",
        ],
        vec!["cycle", "create", "--label", "Test", "--request-key", " "],
        vec!["cycle", "append", "invalid", "--template", "checks.probe"],
        vec![
            "cycle",
            "append",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "--case-id",
            "invalid",
        ],
        vec![
            "cycle",
            "append",
            "--input-file",
            file.path().to_str().unwrap(),
        ],
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

    for arguments in [
        vec!["cycle", "create", "--label", "Test"],
        vec![
            "cycle",
            "append",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "--template",
            "checks.probe",
        ],
        vec!["plan", "preview", "--template", "checks.probe"],
        vec![
            "plan",
            "preview",
            "--selection-json",
            "{\"suite_id\":\"bench.checks\",\"template_id\":\"bench.checks.probe_2\",\"parameters\":[],\"files\":[]}",
        ],
    ] {
        let args = Cli::try_parse_from(
            std::iter::once("dokime")
                .chain(arguments)
                .chain(["--endpoint", "invalid-endpoint"]),
        )
        .unwrap();
        assert!(matches!(args.run().await, Err(Error::Client(_))));
    }
}

pub(crate) async fn offline_backend_profiles() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("backends.json");
    let filename = path.to_str().unwrap();
    let command = |arguments: &[&str]| {
        Cli::try_parse_from(std::iter::once("dokime").chain(arguments.iter().copied())).unwrap()
    };
    command(&["backend", "list", "--profiles-file", filename, "--json"])
        .run()
        .await
        .unwrap();
    assert!(!path.exists());
    let invalid = command(&[
        "backend",
        "add",
        "lab",
        "--endpoint",
        "http://remote.example",
        "--profiles-file",
        filename,
    ]);
    assert_eq!(
        invalid.run().await.unwrap_err().exit_code(),
        std::process::ExitCode::from(2)
    );
    assert!(!path.exists());
    command(&[
        "backend",
        "add",
        "lab",
        "--endpoint",
        "http://127.0.0.1:8765",
        "--profiles-file",
        filename,
    ])
    .run()
    .await
    .unwrap();
    let original = std::fs::read(&path).unwrap();
    let duplicate = command(&[
        "backend",
        "add",
        "lab",
        "--endpoint",
        "http://127.0.0.1:9000",
        "--profiles-file",
        filename,
    ]);
    assert!(matches!(duplicate.run().await, Err(Error::Input(_))));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    command(&["backend", "use", "lab", "--profiles-file", filename])
        .run()
        .await
        .unwrap();
    assert_eq!(
        Store::new(&path).load().unwrap().default.as_deref(),
        Some("lab")
    );
    command(&[
        "backend",
        "show",
        "lab",
        "--profiles-file",
        filename,
        "--json",
    ])
    .run()
    .await
    .unwrap();
    let selected = std::fs::read(&path).unwrap();
    assert!(matches!(
        command(&["backend", "use", "unknown", "--profiles-file", filename])
            .run()
            .await,
        Err(Error::Input(_))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), selected);
    command(&["backend", "remove", "lab", "--profiles-file", filename])
        .run()
        .await
        .unwrap();
    let document = Store::new(&path).load().unwrap();
    assert!(document.default.is_none());
    assert!(document.profiles.is_empty());
}

pub(crate) async fn authentication_inputs_precede_external_operations() {
    for option in ["--ca", "--expected-backend"] {
        assert!(Cli::try_parse_from(["dokime", "status", option, "value"]).is_err());
        assert!(
            Cli::try_parse_from([
                "dokime",
                "status",
                "--endpoint",
                "https://host:8765",
                option,
                "value",
            ])
            .is_ok()
        );
    }
    assert!(
        Cli::try_parse_from([
            "dokime",
            "status",
            "--backend",
            "lab",
            "--endpoint",
            "http://127.0.0.1"
        ])
        .is_err()
    );
    assert!(
        Cli::try_parse_from([
            "dokime",
            "status",
            "--backend",
            "lab",
            "--ca",
            "missing.pem"
        ])
        .is_err()
    );
    assert!(
        Cli::try_parse_from([
            "dokime",
            "status",
            "--backend",
            "lab",
            "--expected-backend",
            "machine"
        ])
        .is_err()
    );
    assert!(
        Cli::try_parse_from([
            "dokime",
            "status",
            "--endpoint",
            "http://127.0.0.1",
            "--timeout-seconds",
            "0"
        ])
        .is_err()
    );
    let login =
        Cli::try_parse_from(["dokime", "auth", "login", "--endpoint", "http://127.0.0.1"]).unwrap();
    assert!(matches!(login.run().await, Err(Error::Input(_))));
    let logout = Cli::try_parse_from([
        "dokime",
        "auth",
        "logout",
        "--endpoint",
        "http://127.0.0.1",
        "--token-file",
        "missing-token",
    ])
    .unwrap();
    assert!(matches!(logout.run().await, Err(Error::Input(_))));
    let check = Error::Check(Box::new(Error::Client(
        dokime_client::Error::IncompatibleApi,
    )));
    assert_eq!(check.exit_code(), std::process::ExitCode::from(1));
    let exhausted = dokime_client::Error::Rpc(tonic::Status::resource_exhausted("response budget"));
    assert_eq!(
        Error::Client(exhausted).exit_code(),
        std::process::ExitCode::from(4)
    );
    let exhausted = dokime_client::Error::Rpc(tonic::Status::resource_exhausted("response budget"));
    assert_eq!(
        Error::Operation(exhausted.into()).exit_code(),
        std::process::ExitCode::from(4)
    );
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
        eprintln!("Scenario: offline_backend_profiles");
        run_scenario(offline_backend_profiles, false, None, false);
        eprintln!("Scenario: authentication_inputs_precede_external_operations");
        run_scenario(
            authentication_inputs_precede_external_operations,
            false,
            None,
            false,
        );
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
