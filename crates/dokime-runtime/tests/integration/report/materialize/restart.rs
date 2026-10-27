/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Persisted Report Restart Story

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::StorageBackend;

// Local Imports
use crate::service_support::prelude::*;
use dokime_domain::model;
use {dokime_runtime::operations::reports::MaterializeReport, model::reports::wire::ReportSubject};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[tokio::test]
async fn persisted_cycle_restart() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    let mut pack = dokime_runtime::config::organization::resolve_with_base(None, None).unwrap();
    pack.manifest["app"]["label"] = toml::Value::String("Stokker Technologies".into());
    let mut brand = pack.brand().unwrap();
    brand.materialize_logos(&root.path().join("appearance")).unwrap();
    runtime.set_brand(Some(brand));
    let suite = fixture_suite(&["passed", "passed"]);
    register_custom_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite).await;
    runtime.update_cycle_policy(cycle_id.clone(), false, CommandContext::default()).await.unwrap();
    let source = cycle_snapshot(&runtime, &cycle_id).await;
    let first = source.test_executions[0].test_execution_id.clone();
    let request =
        runtime.start_execution(first.clone(), dokime_domain::model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    let checkpoint = crate::service_support::native::checkpoint(&runtime, &first).await;
    crate::service_support::waits::wait_for_run_settled(&runtime, &request.command_id).await;
    let before = cycle_snapshot(&runtime, &cycle_id).await;
    assert!(before.saved.is_none(), "the parent still has a pending execution");
    // A never-started member is a draft: skipped, without a finish.
    assert_eq!(before.test_executions[1].status, ExecutionStatus::Skipped);
    assert!(before.test_executions[1].finished_at.is_none());
    assert!(before.test_executions[0].saved.is_some(), "member sealing precedes the open container's seal");
    let seal = crate::service_support::native::open(&runtime, &checkpoint).await.execution().saved.clone().unwrap();
    let sealed_html = crate::service_support::reports::bytes(runtime.storage(), &seal.report).await;
    let captured = crate::service_support::reports::parse(std::str::from_utf8(&sealed_html).unwrap());
    assert_eq!(captured["appearance"]["app_name"], "Stokker Technologies");
    assert!(captured["appearance"]["logos"][0]["url"].as_str().unwrap().starts_with("data:image/svg+xml;base64,"));
    assert!(captured["appearance"]["themes"][0]["stylesheet"].as_str().unwrap().contains("data:font/woff2;base64,"));
    let input_bytes = crate::service_support::reports::bytes(runtime.storage(), &seal.source.context).await;
    let boundary = runtime.storage().cursor_saved();
    let report = runtime
        .operations()
        .call(MaterializeReport { subject: ReportSubject::Cycle(cycle_id.to_string()), at: boundary.clone() })
        .await
        .unwrap();
    let html = crate::service_support::reports::bytes(runtime.storage(), &report.report).await;
    assert!(!html.is_empty());
    let reread = crate::service_support::native::checkpoint(&runtime, &first).await;
    assert_eq!(
        crate::service_support::native::open(&runtime, &reread).await.execution().saved.as_ref(),
        Some(&seal),
        "cycle composition must not reseal its member"
    );
    drop(reread);
    runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    let settled = cycle_snapshot(&runtime, &cycle_id).await;
    let second = &settled.test_executions[1];
    let second_seal = second.saved.as_ref().unwrap().clone();
    assert_eq!(second.status, ExecutionStatus::Cancelled);
    drop((checkpoint, runtime));
    owner.shutdown().await.unwrap();
    std::fs::remove_dir_all(root.path().join("appearance")).unwrap();

    // Sealed Reports Retain Their Exact Anchors Through Reclamation.
    let backend = dokime_runtime::storage::Backend::primary(root.path().to_owned()).await.unwrap();
    let at = backend.open().await.unwrap();
    let released = backend
        .local_state()
        .call(|connection| {
            let mut statement = connection.prepare("SELECT item, through FROM storage_object_releases")?;
            let items =
                statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?.collect::<Result<Vec<_>, _>>()?;
            Ok(items)
        })
        .await
        .unwrap();
    assert!(released.iter().any(|(item, _)| item == cycle_id.as_str()), "sealed parent releases prior history: {released:?}");
    // A sealed member releases exactly the revisions before the anchor its
    // report captured, including the cancelled never-started member.
    let anchor = format!("{:016x}", seal.source.execution.position.sequence - 1);
    let second_anchor = format!("{:016x}", second_seal.source.execution.position.sequence - 1);
    assert_eq!(
        released
            .iter()
            .filter(|(item, _)| item == second.test_execution_id.as_str())
            .map(|(_, through)| through.as_str())
            .collect::<Vec<_>>(),
        [second_anchor.as_str()],
        "cancelled member retains its exact report anchor: {released:?}"
    );
    assert_eq!(
        released.iter().filter(|(item, _)| item == first.as_str()).map(|(_, through)| through.as_str()).collect::<Vec<_>>(),
        [anchor.as_str()],
        "a sealed member releases through its captured anchor: {released:?}"
    );
    while !backend
        .prune_objects(&at, NonZeroUsize::new(128).unwrap(), |schema| {
            harmos::domain::model::storage::catalog::definition::<dokime_runtime::storage::objects::Objects>(schema)
                .map(|definition| definition.retention())
        })
        .await
        .unwrap()
    {}
    backend.close().await.unwrap();
    drop(backend);

    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    runtime.set_brand(dokime_runtime::config::organization::brand());
    let recovered = dokime_runtime::storage::objects::cycle::read(runtime.storage(), &cycle_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(recovered.value.saved, settled.saved);
    drop(recovered);
    assert!(runtime.cycles().await.is_empty(), "sealed containers stay in native history");
    let recovered_checkpoint = crate::service_support::native::checkpoint(&runtime, &first).await;
    assert_eq!(crate::service_support::native::open(&runtime, &recovered_checkpoint).await.execution().saved.as_ref(), Some(&seal));
    drop(recovered_checkpoint);
    assert_eq!(crate::service_support::reports::bytes(runtime.storage(), &seal.source.context).await, input_bytes);
    let report = runtime.open_cycle_report(&cycle_id).await.unwrap();
    assert!(!report.html.is_empty());
    assert_eq!(
        crate::service_support::reports::bytes(runtime.storage(), &seal.report).await,
        sealed_html,
        "cycle composition must not replace the sealed execution report"
    );
    let second_checkpoint = crate::service_support::native::checkpoint(&runtime, &second.test_execution_id).await;
    let second_view = crate::service_support::native::open(&runtime, &second_checkpoint).await;
    assert_eq!(second_view.execution().status, ExecutionStatus::Cancelled);
    assert_eq!(second_view.execution().saved.as_ref(), Some(&second_seal));
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

/* === Tests === */
