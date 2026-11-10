/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Server Storage Format Refusal

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::ExitCode;

// Third Party Imports

// Local Imports

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
async fn storage_format_refusal_keeps_the_server_storage_classification_and_files() {
    let root = tempfile::tempdir().unwrap();
    let legacy = root.path().join("harmos");
    std::fs::create_dir(&legacy).unwrap();
    let journal = legacy.join("journal.jsonl");
    let bytes = b"unreadable built-in history\n";
    std::fs::write(&journal, bytes).unwrap();
    let config = dokime::config::Config::local(root.path());
    let error = dokime::app::launch(config).await.err().unwrap();
    let error = dokime_server::host::Error::launch(error, root.path().to_owned(), "Test Console".into());
    assert!(matches!(&error, dokime_server::host::Error::IncompatibleStorage { root: path, .. } if path == root.path()));
    assert_eq!(error.exit_code(), ExitCode::from(4));
    assert_eq!(std::fs::read(journal).unwrap(), bytes);
    assert!(!root.path().join("local.sqlite").exists());
}
