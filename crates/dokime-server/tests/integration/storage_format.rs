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
use std::{collections::BTreeMap, process::ExitCode};

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
    for marked in [false, true] {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("evidence"), [0, 1, 2, 255]).unwrap();
        if marked {
            let record = serde_json::json!({"format": "different-layout", "writer_build": "previous-build"});
            std::fs::write(root.path().join(dokime::storage::format::RECORD), serde_json::to_vec(&record).unwrap()).unwrap();
        }
        let files = || {
            std::fs::read_dir(root.path())
                .unwrap()
                .map(|entry| {
                    let entry = entry.unwrap();
                    let metadata = entry.metadata().unwrap();
                    (entry.file_name(), (std::fs::read(entry.path()).unwrap(), metadata.modified().unwrap()))
                })
                .collect::<BTreeMap<_, _>>()
        };
        let before = files();
        let config = dokime::config::Config::local(root.path());
        let error = dokime::app::launch(config).await.err().unwrap();
        let error = dokime_server::host::Error::launch(error, root.path().to_owned());
        assert!(matches!(&error, dokime_server::host::Error::IncompatibleStorage { root: path, .. } if path == root.path()));
        assert_eq!(error.exit_code(), ExitCode::from(4));
        assert_eq!(files(), before);
    }
}
