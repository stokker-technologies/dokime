/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Offline Backend Profile And Credential Admission Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use clap::Parser;

// Local Imports
use crate::configuration::resolve as configuration;
use dokime::client::profiles::Store;
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
#[dokime_tracing::trace_test]
#[tokio::test]
async fn backend_profiles() {
    // Add, Select, Reopen And Remove An Offline Profile.
    let application = tempfile::tempdir().unwrap();
    let configuration = configuration(application.path());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("backends.json");
    let filename = path.to_str().unwrap();
    let command = |arguments: &[&str]| Cli::try_parse_from(std::iter::once("dokime").chain(arguments.iter().copied())).unwrap();
    command(&["backend", "list", "--profiles-file", filename, "--json"]).run(configuration.clone()).await.unwrap();
    assert!(!path.exists());
    let invalid = command(&[
        "backend",
        "add",
        "--identifier",
        "lab",
        "--label",
        "Lab",
        "--endpoint",
        "http://remote.example/path",
        "--profiles-file",
        filename,
    ]);
    assert_eq!(invalid.run(configuration.clone()).await.unwrap_err().exit_code(), std::process::ExitCode::from(2));
    assert!(!path.exists());
    command(&[
        "backend",
        "add",
        "--identifier",
        "lab",
        "--label",
        "Lab",
        "--endpoint",
        "http://remote.example:8765",
        "--profiles-file",
        filename,
    ])
    .run(configuration.clone())
    .await
    .unwrap();
    let selection = dokime::client::profiles::select(&Store::new(&path).load().unwrap(), Some("lab"), None, None).unwrap();
    assert!(selection.target.is_remote());
    assert_eq!(selection.target.endpoint(), "http://remote.example:8765/");
    let original = std::fs::read(&path).unwrap();
    let duplicate = command(&[
        "backend",
        "add",
        "--identifier",
        "lab",
        "--label",
        "Duplicate",
        "--endpoint",
        "http://127.0.0.1:9000",
        "--profiles-file",
        filename,
    ]);
    assert!(matches!(duplicate.run(configuration.clone()).await, Err(Error::Input(_))));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    command(&["backend", "use", "--identifier", "lab", "--profiles-file", filename]).run(configuration.clone()).await.unwrap();
    assert_eq!(Store::new(&path).load().unwrap().default.as_deref(), Some("lab"));
    command(&["backend", "show", "--identifier", "lab", "--profiles-file", filename, "--json"]).run(configuration.clone()).await.unwrap();
    let selected = std::fs::read(&path).unwrap();
    assert!(matches!(
        command(&["backend", "use", "--identifier", "unknown", "--profiles-file", filename]).run(configuration.clone()).await,
        Err(Error::Input(_))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), selected);
    command(&["backend", "remove", "--identifier", "lab", "--profiles-file", filename]).run(configuration.clone()).await.unwrap();
    let document = Store::new(&path).load().unwrap();
    assert!(document.default.is_none());
    assert!(document.profiles.is_empty());

    // Refuse Removed Credential Flags And Preserve Error Categories.
    for option in ["--ca", "--expected-backend"] {
        assert!(Cli::try_parse_from(["dokime", "status", option, "value"]).is_err());
        assert!(Cli::try_parse_from(["dokime", "status", "--endpoint", "https://host:8765", option, "value",]).is_err());
    }
    assert!(Cli::try_parse_from(["dokime", "status", "--backend", "lab", "--endpoint", "http://127.0.0.1"]).is_err());
    assert!(Cli::try_parse_from(["dokime", "status", "--backend", "lab", "--ca", "missing.pem"]).is_err());
    assert!(Cli::try_parse_from(["dokime", "status", "--backend", "lab", "--expected-backend", "machine"]).is_err());
    assert!(Cli::try_parse_from(["dokime", "status", "--endpoint", "http://127.0.0.1", "--timeout-seconds", "0"]).is_err());
    assert!(Cli::try_parse_from(["dokime", "auth", "login", "--key-file", "missing-key"]).is_err());
    assert!(Cli::try_parse_from(["dokime", "auth", "logout"]).is_err());
    assert!(Cli::try_parse_from(["dokime", "backend", "add", "--remote", "lab.dokime-remote"]).is_ok());
    assert!(Cli::try_parse_from(["dokime", "backend", "add", "--remote", "lab.dokime-remote", "--identifier", "lab"]).is_ok());
    assert!(Cli::try_parse_from(["dokime", "backend", "add", "--remote", "lab.dokime-remote", "--endpoint", "http://lab"]).is_err());
    assert!(Cli::try_parse_from(["dokime", "backend", "add", "--name", "lab", "--endpoint", "http://lab"]).is_err());

    let check = Error::Check(Box::new(Error::Client(dokime::client::Error::IncompatibleApi)));
    assert_eq!(check.exit_code(), std::process::ExitCode::from(1));
    let exhausted = dokime::client::Error::Rpc(tonic::Status::resource_exhausted("response budget"));
    assert_eq!(Error::Client(exhausted).exit_code(), std::process::ExitCode::from(4));
    let exhausted = dokime::client::Error::Rpc(tonic::Status::resource_exhausted("response budget"));
    assert_eq!(Error::Operation(exhausted.into()).exit_code(), std::process::ExitCode::from(4));
}
