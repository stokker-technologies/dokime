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

// Third Party Imports

// Local Imports
use dokime_runtime::artifacts::ArtifactUploadInput;

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
async fn execution_log_keeps_plain_name_before_and_after_step_uploads() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../src/report/fixtures/execution-report-input.json")).unwrap();
    let mut cycle: dokime_domain::model::execution::result::CycleView = serde_json::from_value(fixture["input"]["cycle"].clone()).unwrap();
    cycle.test_executions[0].saved = None;
    let execution = &cycle.test_executions[0];
    let root = tempfile::tempdir().unwrap();
    let mut owner = dokime_runtime::Runtime::start(dokime_runtime::RuntimeConfig::new(root.path(), cycle.metadata.clone())).await.unwrap();
    let runtime = owner.handle();
    let storage = dokime_runtime::testing::ExecutionOutputs::new(&runtime);
    storage.acquire(&cycle, execution).await.unwrap();
    let id = &execution.test_execution_id;
    for expected in ["artifacts/execution.2.log", "artifacts/execution.3.log"] {
        let upload = storage
            .clone()
            .begin_artifact(
                id.clone(),
                ArtifactUploadInput {
                    scope: execution.scope.clone(),
                    filename: "execution.log".into(),
                    content_type: Some("text/plain".into()),
                    size_bytes: 1,
                    sha256: None,
                },
                dokime_runtime::model::time::now_utc(),
                runtime.operation_progress().begin("Artifact Collision"),
            )
            .await
            .unwrap();
        let entries = storage.native_entries(id).await.unwrap();
        assert!(entries.iter().any(|entry| entry.artifact_id == upload.artifact_id().as_str() && entry.relative_path == expected));
        storage.reserve_output(id, "artifacts/execution.log").await.unwrap();
        drop(upload);
    }
    drop(storage);
    drop(runtime);
    owner.shutdown().await.unwrap();
}
