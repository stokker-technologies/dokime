/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Input And Interrupt-Recovery Admission Story

use std::io::Write;

use clap::Parser;

use crate::configuration::resolve as configuration;
use dokime_cli::{Cli, Error};

#[tokio::test]
async fn execution_inputs() {
    // Parse Execution And Cycle Requests.
    for command in ["run", "stop", "request", "rerun"] {
        assert!(Cli::try_parse_from(["dokime", "cycle", command]).is_err());
    }
    let execution = "00000000-0000-0000-0000-000000000001";
    let run = ["dokime", "execution", "run", execution];
    assert!(Cli::try_parse_from(run).is_ok());
    let listening = ["--records", "--samples", "--record-type", "obc.*", "--record-type", "dokime.*", "--stream", "obc.?"];
    assert!(Cli::try_parse_from(run.into_iter().chain(listening)).is_ok());
    assert!(Cli::try_parse_from(run.into_iter().chain(["--request-key", "obsolete"])).is_err());
    assert!(Cli::try_parse_from(["dokime", "plan", "materialize", "--template", "obc.check", "--id-only"]).is_ok());
    assert!(Cli::try_parse_from(["dokime", "cycle", "append", execution, "--case-id", execution, "--id-only"]).is_ok());
    let append = ["dokime", "cycle", "append", execution, "--case-id", execution, "--id-file", "ids.txt"];
    assert!(Cli::try_parse_from(append).is_ok());
    let next = ["dokime", "execution", "prompt", "next", execution, "--id-only", "--id-file", "ids.txt"];
    assert!(Cli::try_parse_from(next).is_ok());
    assert!(Cli::try_parse_from(["dokime", "cycle", "create", "--label", "Test", "--id-file", "ids.txt"]).is_err());

    // Admit Complete Interrupt-Recovery Choices Before Connection.
    let application = tempfile::tempdir().unwrap();
    let configuration = configuration(application.path());
    let execution = "00000000-0000-0000-0000-000000000001";
    let cycle = "00000000-0000-0000-0000-000000000002";
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().to_str().unwrap();
    let export_output = directory.path().join("requested-export.zip");
    for command in [
        vec!["dokime", "execution", "start", execution],
        vec!["dokime", "execution", "run", execution],
        vec!["dokime", "execution", "wait", execution],
        vec!["dokime", "execution", "prompt", "next", execution],
        vec!["dokime", "execution", "prompt", "show", execution, execution],
        vec!["dokime", "execution", "prompt", "answer", execution, execution, "--action", "yes"],
        vec!["dokime", "execution", "export", execution, "--output", export_output.to_str().unwrap()],
    ] {
        for missing in [
            vec!["--cancel-on-interrupt"],
            vec!["--owned-cycle", cycle],
            vec!["--recovery-directory", path],
            vec!["--cancel-on-interrupt", "--owned-cycle", cycle],
        ] {
            let arguments = command.iter().copied().chain(missing);
            assert!(Cli::try_parse_from(arguments).is_err(), "partial recovery choices must be refused");
        }
        let recovery = ["--cancel-on-interrupt", "--owned-cycle", cycle, "--recovery-directory", path, "--endpoint", "http://"];
        let arguments = command.iter().copied().chain(recovery);
        let parsed = Cli::try_parse_from(arguments).unwrap();
        assert!(
            matches!(parsed.run(configuration.clone()).await, Err(Error::Client(_))),
            "complete choices pass offline validation before endpoint parsing"
        );
        let destination = directory.path().join(format!("execution-{execution}.zip"));
        std::fs::write(&destination, b"previous evidence").unwrap();
        let arguments = command.iter().copied().chain(recovery);
        let parsed = Cli::try_parse_from(arguments).unwrap();
        assert!(
            matches!(parsed.run(configuration.clone()).await, Err(Error::Download(_))),
            "output conflicts must be refused before endpoint parsing"
        );
        assert_eq!(std::fs::read(&destination).unwrap(), b"previous evidence");
        std::fs::remove_file(destination).unwrap();
    }

    let append = ["dokime", "cycle", "append", cycle, "--case-id", execution];
    for incomplete in [vec!["--cancel-on-interrupt"], vec!["--owned-cycle", cycle], vec!["--recovery-directory", path]] {
        assert!(Cli::try_parse_from(append.into_iter().chain(incomplete)).is_err());
    }
    let flags = ["--cancel-on-interrupt", "--owned-cycle", cycle, "--recovery-directory", path, "--endpoint", "http://"];
    let parsed = Cli::try_parse_from(append.into_iter().chain(flags)).unwrap();
    assert!(matches!(parsed.run(configuration.clone()).await, Err(Error::Client(_))));
    let flags = ["--cancel-on-interrupt", "--owned-cycle", execution, "--recovery-directory", path, "--endpoint", "http://"];
    let parsed = Cli::try_parse_from(append.into_iter().chain(flags)).unwrap();
    assert!(matches!(parsed.run(configuration.clone()).await, Err(Error::Input(_))), "append cannot authorize recovery of another cycle");

    // Validate Request Files And Inputs Before Connection.

    assert!(Cli::try_parse_from(["dokime", "server"]).is_ok());
    for arguments in [
        vec!["plan", "preview", "--template", "checks.probe", "--case-id", "case"],
        vec!["plan", "preview", "--input-file", "request.json", "--case-id", "case"],
        vec!["plan", "preview", "--case-id", "case", "--selection-json", "{}"],
        vec!["plan", "register", "--input-file", "request.json", "--request-key", "plan-1"],
        vec!["cycle", "create", "--input-file", "request.json", "--label", "Override"],
        // A cycle needs a label and has no start or wait shortcut.
        vec!["cycle", "create"],
        vec!["cycle", "create", "--label", "Test", "--wait-seconds", "5"],
        vec!["cycle", "create", "--label", "Test", "--id-only", "--json"],
        vec!["cycle", "create", "--label", "Test", "--id-only", "--request-key", "run-1"],
        vec!["plan", "preview", "--parameters-json", "{}"],
        // Appending names an existing cycle, or reads it from the request file.
        vec!["cycle", "append", "--template", "checks.probe"],
        vec!["cycle", "append", "98184567-8924-4fb0-b4e2-667347e097a1", "--input-file", "request.json"],
        vec!["cycle", "append", "98184567-8924-4fb0-b4e2-667347e097a1", "--input-file", "request.json", "--template", "checks.probe"],
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
            "{}",
            "--endpoint",
            "http://127.0.0.1:8765",
        ])
        .is_ok()
    );
    assert!(
        Cli::try_parse_from(["dokime", "operation", "show", "external.plugin.exchange", "--owner", "external-plugin", "--json",]).is_ok()
    );

    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(b"{broken-json").unwrap();
    let args = Cli::try_parse_from([
        "dokime",
        "operation",
        "call",
        "dokime.runtime.health",
        "--endpoint",
        "http://",
        "--input",
        file.path().to_str().unwrap(),
    ])
    .unwrap();
    assert!(matches!(args.run(configuration.clone()).await, Err(Error::Input(_))));

    let args = Cli::try_parse_from(["dokime", "operation", "call", "missing.operation", "--endpoint", "http://"]).unwrap();
    assert!(matches!(args.run(configuration.clone()).await, Err(Error::Client(_))));

    let args = Cli::try_parse_from(["dokime", "operation", "call", "dokime.evidence.watch_records", "--endpoint", "http://"]).unwrap();
    let refused = args.run(configuration.clone()).await.unwrap_err();
    assert_eq!(refused.exit_code(), std::process::ExitCode::from(2));
    assert!(matches!(refused, Error::Client(_)));

    for arguments in [
        vec!["execution", "stop", "invalid"],
        vec!["execution", "run", "invalid"],
        vec!["execution", "start", "invalid"],
        vec!["execution", "wait", "invalid"],
        vec!["execution", "run", "98184567-8924-4fb0-b4e2-667347e097a1", "--stream", "obc.[ab]"],
        vec!["execution", "run", "98184567-8924-4fb0-b4e2-667347e097a1", "--record-type", "^obc.*$"],
        vec!["plan", "materialize", "--case-id", "98184567-8924-4fb0-b4e2-667347e097a1"],
        vec!["plan", "materialize", "--template", "checks.probe", "--max-cases", "1"],
        vec!["execution", "prompt", "next", "invalid"],
        vec!["execution", "prompt", "show", "98184567-8924-4fb0-b4e2-667347e097a1", "invalid"],
        vec![
            "execution",
            "prompt",
            "answer",
            "98184567-8924-4fb0-b4e2-667347e097a1",
            "a13ae695-1f02-4464-9f7b-0b14ecffcb41",
            "--action",
            "yes",
            "--values-json",
            "[]",
        ],
        vec!["plan", "preview", "--template", "checks.probe", "--parameters-json", "[]"],
        vec!["plan", "preview", "--template", "checks.probe", "--parameters-json", "{broken"],
        vec!["plan", "preview", "--case-id", "invalid"],
        vec!["plan", "preview", "--selection-json", "{\"unexpected\":true}"],
        vec!["plan", "register", "--case-id", "invalid", "--request-key", "plan-1"],
        vec!["plan", "register", "--template", "checks.probe", "--request-key", " "],
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
        vec!["plan", "register", "--template", "0197f000-0000-7000-8000-000000000002", "--request-key", "plan-1"],
        vec!["plan", "preview", "--input-file", file.path().to_str().unwrap()],
        vec!["plan", "register", "--input-file", file.path().to_str().unwrap()],
        vec!["cycle", "create", "--input-file", file.path().to_str().unwrap()],
        vec!["cycle", "create", "--label", "Test", "--identifier", "Not An Identifier"],
        vec!["cycle", "append", "invalid", "--case-id", "98184567-8924-4fb0-b4e2-667347e097a1"],
        vec!["cycle", "append", "98184567-8924-4fb0-b4e2-667347e097a1", "--case-id", "invalid"],
        vec!["cycle", "append", "--input-file", file.path().to_str().unwrap()],
    ] {
        let args = Cli::try_parse_from(std::iter::once("dokime").chain(arguments).chain(["--endpoint", "http://"])).unwrap();
        assert!(matches!(args.run(configuration.clone()).await, Err(Error::Input(_))));
    }

    for arguments in [
        vec!["cycle", "create", "--label", "Test"],
        vec!["cycle", "append", "98184567-8924-4fb0-b4e2-667347e097a1", "--case-id", "98184567-8924-4fb0-b4e2-667347e097a1"],
        vec!["plan", "preview", "--template", "checks.probe"],
        vec![
            "plan",
            "preview",
            "--selection-json",
            "{\"suite_id\":\"bench.checks\",\"template_id\":\"bench.checks.probe_2\",\"parameters\":[],\"files\":[]}",
        ],
    ] {
        let args = Cli::try_parse_from(std::iter::once("dokime").chain(arguments).chain(["--endpoint", "http://"])).unwrap();
        assert!(matches!(args.run(configuration.clone()).await, Err(Error::Client(_))));
    }
}
