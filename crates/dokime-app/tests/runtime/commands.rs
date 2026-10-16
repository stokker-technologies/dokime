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
use std::{fs, path::PathBuf};
// Third Party Imports
use tokio::process::Command;
// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn receipts_respect_idempotency() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(provider_config(root.path())).await;
    let context = CommandContext {
        idempotency_key: Some("client-key-1".to_owned()),
        ..CommandContext::default()
    };
    let test_cycle_id = provider_cycle(&runtime, "idempotent").await;

    let first = runtime
        .run_cycle_cases(test_cycle_id.clone(), context.clone())
        .await;
    let duplicate = runtime
        .run_cycle_cases(test_cycle_id, context.clone())
        .await;
    let fresh_cycle_id = provider_cycle(&runtime, "fresh").await;
    let fresh = runtime
        .run_cycle_cases(fresh_cycle_id, CommandContext::default())
        .await;

    assert_eq!(first.status, CommandStatus::Accepted);
    assert_eq!(duplicate.command_id, first.command_id);
    assert_eq!(duplicate.accepted_at, first.accepted_at);
    assert_eq!(duplicate.job_id, first.job_id);
    assert_ne!(fresh.command_id, first.command_id);
}

pub(crate) async fn idempotency_survives_restart() {
    let root = tempdir().unwrap();
    let context = CommandContext {
        idempotency_key: Some("restart-key-1".to_owned()),
        ..CommandContext::default()
    };
    let definition = CycleDefinition {
        suite: empty_suite(),
        metadata: metadata("fixture.cycle", "Fixture Cycle"),
        cases: Vec::new(),
        continue_on_failure: false,
    };

    let cycle_id = {
        let (mut runtime, handle) = booted(root.path()).await;
        let created = handle
            .start_cycle(definition.clone(), context.clone())
            .await;
        assert!(created.is_ok());
        let cycles = handle
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .cycles;
        assert_eq!(cycles.len(), 1);
        let cycle_id = cycles[0].test_cycle_id.clone();
        drop(handle);
        runtime.shutdown().await.unwrap();
        cycle_id
    };

    let (mut runtime, handle) = booted(root.path()).await;
    let restored = handle.runtime_overview().await.unwrap().cycles;
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].test_cycle_id, cycle_id);
    assert_eq!(restored[0].lifecycle, CycleStatus::Stopped);
    let fresh = handle
        .start_cycle(definition.clone(), context.clone())
        .await
        .unwrap();
    assert_eq!(fresh, cycle_id);
    assert_eq!(
        handle.start_cycle(definition, context).await.unwrap(),
        fresh
    );
    assert_eq!(
        handle
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .cycles
            .len(),
        1
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn accepted_provider_revision() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(provider_config(root.path())).await;
    let test_cycle_id = provider_cycle(&runtime, "replay").await;
    let before = runtime.runtime_overview().await.unwrap().revision;
    let receipt = runtime
        .run_cycle_cases(test_cycle_id, CommandContext::default())
        .await;
    let revision = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds")
        .revision;

    assert_eq!(receipt.status, CommandStatus::Accepted);
    assert!(receipt.runtime_revision > before);
    // Provider progress can commit while the caller fetches the next overview.
    assert!(revision >= receipt.runtime_revision);
}

pub(crate) async fn refusals_refuses_processor() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(provider_config(root.path())).await;
    let revision = || async {
        runtime
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .revision
    };
    let before = revision().await;

    let rejected = runtime
        .run_cycle_cases(TestCycleId::new(), CommandContext::default())
        .await;
    let after = revision().await;

    assert_eq!(rejected.status, CommandStatus::Rejected);
    assert_eq!(before, after, "refused commands must not be journaled");

    let accepted_cycle_id = provider_cycle(&runtime, "after-rejection").await;
    let accepted = runtime
        .run_cycle_cases(accepted_cycle_id, CommandContext::default())
        .await;

    assert_eq!(accepted.status, CommandStatus::Accepted);
    assert!(accepted.runtime_revision > rejected.runtime_revision);
    // Provider progress can commit while the caller fetches the next overview.
    assert!(revision().await >= accepted.runtime_revision);
}

#[tokio::test]
async fn kill_recovers_cycle() {
    if std::env::var_os("DOKIME_RUNTIME_KILL_CHILD").is_some() {
        run_kill_child().await;
        return;
    }

    let root = tempdir().unwrap();
    let marker = root.path().join("journal-entry-ready");
    let mut child = Command::new(std::env::current_exe().expect("current test executable"))
        .arg("--exact")
        .arg("commands::kill_recovers_cycle")
        .arg("--nocapture")
        .env("DOKIME_RUNTIME_KILL_CHILD", "1")
        .env("DOKIME_RUNTIME_KILL_ROOT", root.path())
        .kill_on_drop(true)
        .spawn()
        .expect("spawn child runtime writer");

    // Readiness includes creating a fresh storage backend. Wait for that handshake,
    // not a one-second boot benchmark, and always reap the child before restart.
    let ready = tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            if marker.is_file() {
                break;
            }

            if let Some(status) = child.try_wait().expect("inspect child runtime") {
                panic!("child exited before writing its readiness marker: {status}");
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    child.kill().await.expect("kill and reap child");
    ready.expect("child did not write its readiness marker");

    // The Saved Cycle Retains Admission Even After It Leaves The Live Registry.
    let (mut runtime_owner, runtime) = booted(root.path()).await;
    let mut query = runtime
        .storage()
        .query(harmos::storage::ObjectQuery::<
            dokime::storage::objects::cycle::Cycle,
        >::new())
        .await
        .unwrap();
    let cycle = query
        .next_page_projected(
            std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            |entries, _| {
                assert_eq!(entries.len(), 1);
                Ok(entries[0]
                    .value
                    .restore()
                    .expect("saved cycle is canonical"))
            },
        )
        .await
        .unwrap()
        .expect("admitted cycle remains saved");
    assert_eq!(cycle.metadata.identifier.as_str(), "fixture.cycle");
    assert!(cycle.execution_ids.is_empty());
    assert_eq!(cycle.lifecycle, CycleStatus::Ready);
    let id = cycle.test_cycle_id.clone();
    drop(cycle);
    drop(query);
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();

    let (mut runtime_owner, runtime) = booted(root.path()).await;
    let view = runtime
        .storage()
        .view_with_limits(crate::support::budgets::story_settings().ceiling)
        .await
        .unwrap();
    let cycle = dokime::storage::objects::cycle::read_in(
        runtime.storage(),
        &view,
        &id,
        harmos::storage::ObjectVersion::Latest,
        std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
    )
    .await
    .unwrap()
    .expect("saved cycle survives another boot");
    // Restart recovery persists the interrupted cycle header.
    assert_eq!(cycle.value.lifecycle, CycleStatus::Stopped);
    assert!(cycle.value.execution_ids.is_empty());
    drop(cycle);
    drop(view);
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();
}

async fn run_kill_child() {
    let root =
        PathBuf::from(std::env::var_os("DOKIME_RUNTIME_KILL_ROOT").expect("kill child root env"));
    let (_owner, runtime) = booted(&root).await;
    let created = runtime
        .start_cycle(
            CycleDefinition {
                suite: empty_suite(),
                metadata: metadata("fixture.cycle", "Fixture Cycle"),
                cases: Vec::new(),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await;

    assert!(created.is_ok());
    fs::write(root.join("journal-entry-ready"), "ready").expect("write marker");

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/* === Tests === */
