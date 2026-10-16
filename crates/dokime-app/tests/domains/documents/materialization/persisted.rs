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
use harmos::storage::FileMetadata;
use sha2::{Digest, Sha256};
// Local Imports
use crate::support::prelude::*;
use dokime::execution::saved::{
    MATERIALIZED_REPORT_INPUT_MEDIA_TYPE, MATERIALIZED_REPORT_INPUT_SUFFIX,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn persisted_input_refusals() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Completed).await;
    let execution = &cycle.test_executions[0];
    let original = execution.saved.as_ref().unwrap().clone();
    let bytes =
        crate::support::reports::bytes(runtime.storage(), &original.materialized_input).await;
    let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let reads = crate::support::budgets::story_settings();

    // Verify The Genuine Persisted Artifact Before Introducing Faults
    let view = runtime
        .storage()
        .view_with_limits(reads.finalization())
        .await
        .unwrap();
    let loaded = dokime_app::report::Renderer
        .load_execution(runtime.storage(), &view, &original)
        .await
        .unwrap();
    assert_eq!(loaded.execution_id, original.source.execution.execution_id);
    assert_eq!(loaded.evidence_through, original.source.evidence_through);
    drop(loaded);
    drop(view);

    let mut version = envelope.clone();
    version["version"] = serde_json::json!(99);
    let mut identity = envelope.clone();
    identity["captured"]["execution_id"] = serde_json::json!(TestExecutionId::new().as_str());
    let mut depth = serde_json::Value::Null;
    for _ in 0..140 {
        depth = serde_json::Value::Array(vec![depth]);
    }
    let mut deep = envelope.clone();
    deep["captured"]["input"]["appearance"] = depth;
    let mut truncated = bytes.clone();
    truncated.truncate(truncated.len() / 2);
    let cases = vec![
        ("malformed", b"{not-json".to_vec(), false, ""),
        ("truncated", truncated, false, ""),
        (
            "version",
            serde_json::to_vec(&version).unwrap(),
            false,
            "version",
        ),
        (
            "identity",
            serde_json::to_vec(&identity).unwrap(),
            false,
            "execution",
        ),
        ("deep", serde_json::to_vec(&deep).unwrap(), false, "nesting"),
        ("scope", bytes.clone(), true, "metadata"),
        (
            "oversized",
            vec![b' '; reads.decoded_bytes.get() + 1],
            false,
            "materialized input file buffer",
        ),
    ];
    for (name, contents, wrong_scope, expected) in cases {
        // Every Malicious File Has A Real Immutable Reference And Matching Hash
        let id = Identifier::new(format!(
            "{}.{}{}",
            execution.test_execution_id,
            uuid::Uuid::now_v7(),
            MATERIALIZED_REPORT_INPUT_SUFFIX
        ))
        .unwrap();
        let file = runtime
            .storage()
            .write_file(
                FileMetadata {
                    id,
                    scope: Some(if wrong_scope {
                        Identifier::new(TestExecutionId::new().as_str()).unwrap()
                    } else {
                        Identifier::new(execution.test_execution_id.as_str()).unwrap()
                    }),
                    media_type: MATERIALIZED_REPORT_INPUT_MEDIA_TYPE.into(),
                    expected_length: Some(contents.len() as u64),
                    expected_sha256: Some(Sha256::digest(&contents).into()),
                },
                &contents,
            )
            .await
            .unwrap();
        let durable = runtime.storage().flush_until(&file.position).await.unwrap();
        let mut suspect = original.clone();
        suspect.materialized_input = file;
        suspect.dependencies_saved = durable;
        let before = runtime.storage().status().accepted;
        let view = runtime
            .storage()
            .view_with_limits(reads.finalization())
            .await
            .unwrap();
        let error = match dokime_app::report::Renderer
            .load_execution(runtime.storage(), &view, &suspect)
            .await
        {
            Ok(_) => panic!("{name} materialized input was accepted"),
            Err(error) => error,
        };
        let mut message = error.to_string();
        let mut source = std::error::Error::source(&error);
        while let Some(error) = source {
            message.push_str(&format!("; {error}"));
            source = error.source();
        }
        assert!(
            expected.is_empty() || message.contains(expected),
            "{name}: {message}"
        );
        assert_eq!(
            runtime.storage().status().accepted,
            before,
            "refusal must not write, seal, or prune"
        );
        assert_eq!(
            cycle_snapshot(&runtime, &cycle_id).await.test_executions[0]
                .saved
                .as_ref(),
            Some(&original)
        );
    }
    assert_eq!(
        crate::support::reports::bytes(runtime.storage(), &original.materialized_input).await,
        bytes
    );
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

/* === Tests === */
