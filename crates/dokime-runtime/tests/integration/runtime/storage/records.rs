/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::service_support::prelude::*;
use dokime_runtime::domains::evidence::RecordSelection;

#[tokio::test]
async fn managed_records_spool() {
    let root = tempdir().unwrap();
    let settings = config(root.path());
    let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings.clone(), std::iter::empty()).await.unwrap();
    runtime.shutdown().await.unwrap();
    let persistent = root_files(root.path());

    let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings.clone(), std::iter::empty()).await.unwrap();
    let handle = runtime.handle();
    handle
        .add_operator_note(OperatorNoteRequest {
            scope: Some(RecordScope::default()),
            author: Some("operator".into()),
            message: "A durable observation".into(),
        })
        .await
        .unwrap();
    let selection = RecordSelection {
        record_types: vec![Identifier::new("operator.note_added").unwrap()],
        ..RecordSelection::default()
    };
    let records = handle.selected_records(selection.clone()).await.unwrap();
    assert_eq!(records.len(), 1);
    drop(handle);
    runtime.shutdown().await.unwrap();
    assert_eq!(root_files(root.path()), persistent, "record capture creates no separate process spool files");

    let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings, std::iter::empty()).await.unwrap();
    let recovered = runtime.handle().selected_records(selection).await.unwrap();
    assert_eq!(recovered, records, "global record visibility survives managed recovery");
    runtime.shutdown().await.unwrap();
    assert_eq!(root_files(root.path()), persistent);
}

fn root_files(root: &Path) -> Vec<std::ffi::OsString> {
    let mut files = std::fs::read_dir(root).unwrap().map(|entry| entry.unwrap().file_name()).collect::<Vec<_>>();
    files.sort();

    files
}
