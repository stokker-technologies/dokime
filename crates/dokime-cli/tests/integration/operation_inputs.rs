/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operation Output Publication And Input Refusal Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use clap::Parser;

// Local Imports
use crate::configuration::resolve as configuration;
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
#[tokio::test]
async fn operation_inputs() {
    let application = tempfile::tempdir().unwrap();
    let configuration = configuration(application.path());
    for arguments in
        [vec!["--json", "{}", "--input", "request.json"], vec!["--input-file", "request.json"], vec!["--json"], vec!["--overwrite"]]
    {
        let command = ["dokime", "operation", "call", "device.enable"];
        let parsed = Cli::try_parse_from(command.into_iter().chain(arguments));
        assert!(parsed.is_err());
    }

    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("response.json");
    std::fs::write(&destination, b"previous response").unwrap();
    let path = destination.to_str().unwrap();
    let command = ["dokime", "operation", "call", "device.enable", "--endpoint", "http://", "--json", "{}", "--output", path];
    let args = Cli::try_parse_from(command).unwrap();
    let error = args.run(configuration.clone()).await.unwrap_err();
    assert!(matches!(error, Error::Input(_)), "existing output must be refused before connection");
    assert_eq!(std::fs::read(&destination).unwrap(), b"previous response");

    let args = Cli::try_parse_from(command.into_iter().chain(["--overwrite"])).unwrap();
    let error = args.run(configuration.clone()).await.unwrap_err();
    assert!(matches!(error, Error::Client(_)), "explicit overwrite may proceed to connection");
    assert_eq!(std::fs::read(&destination).unwrap(), b"previous response");
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1, "failed calls remove their staged output");

    let args = Cli::try_parse_from([
        "dokime",
        "operation",
        "call",
        "device.enable",
        "--endpoint",
        "http://",
        "--json",
        "{broken",
        "--output",
        path,
        "--overwrite",
    ])
    .unwrap();
    assert!(matches!(args.run(configuration.clone()).await.unwrap_err(), Error::Input(_)));
    assert_eq!(std::fs::read(&destination).unwrap(), b"previous response");

    let directory_output = directory.path().join("directory.json");
    std::fs::create_dir(&directory_output).unwrap();
    let args = Cli::try_parse_from([
        "dokime",
        "operation",
        "call",
        "device.enable",
        "--endpoint",
        "http://",
        "--output",
        directory_output.to_str().unwrap(),
        "--overwrite",
    ])
    .unwrap();
    assert!(matches!(args.run(configuration.clone()).await.unwrap_err(), Error::Input(_)));
    assert!(directory_output.is_dir());
}
