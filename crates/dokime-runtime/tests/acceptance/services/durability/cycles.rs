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
use std::path::PathBuf;
// Third Party Imports
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

#[tokio::test]
async fn kill_recovers_cycle() {
    if std::env::var_os("DOKIME_RUNTIME_KILL_CHILD").is_some() {
        run_kill_child().await;
        return;
    }

    let root = tempdir().unwrap();
    let mut child = crate::service_support::children::ChildStory::spawn(
        "services::durability::cycles::kill_recovers_cycle",
        &[("DOKIME_RUNTIME_KILL_CHILD", "1".as_ref()), ("DOKIME_RUNTIME_KILL_ROOT", root.path().as_os_str())],
    );
    // Readiness includes creating a fresh storage backend; always reap the child before restart.
    child.ready().await;
    child.kill();

    // The Saved Cycle Retains Admission Even After It Leaves The Live Registry.
    let (mut runtime_owner, runtime) = booted(root.path()).await;
    let cycles = runtime
        .storage()
        .read_objects::<dokime_runtime::storage::objects::cycle::Cycle>(harmos::storage::ObjectRead {
            at: runtime.storage().cursor_saved(),
            key: None,
            versions: harmos::storage::query::ObjectVersions::Latest,
        })
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    assert_eq!(cycles.iter().map(|batch| batch.len()).sum::<usize>(), 1);
    let cycle = cycles
        .try_map(|batches| {
            let batch = batches.into_iter().find(|batch| !batch.is_empty()).unwrap();

            batch.try_map(|mut revisions| dokime_domain::model::execution::result::TestCycle::try_from(revisions.pop().unwrap().value))
        })
        .unwrap();
    assert_eq!(cycle.metadata.identifier.as_str(), "fixture.cycle");
    assert!(cycle.execution_ids.is_empty());
    assert_eq!(cycle.lifecycle, CycleStatus::Stopped);
    assert_eq!(cycle.phase, dokime_domain::model::execution::result::CyclePhase::Sealed);
    assert!(cycle.saved.is_some(), "recovery seals the interrupted container before admission");
    let id = cycle.test_cycle_id.clone();
    drop(cycle);
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();

    let (mut runtime_owner, runtime) = booted(root.path()).await;
    let cycle = dokime_runtime::storage::objects::cycle::read(runtime.storage(), &id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .expect("saved cycle survives another boot");
    // Restart recovery persists the interrupted cycle header.
    assert_eq!(cycle.value.lifecycle, CycleStatus::Stopped);
    assert!(cycle.value.execution_ids.is_empty());
    drop(cycle);
    drop(runtime);
    runtime_owner.shutdown().await.unwrap();
}

async fn run_kill_child() {
    let root = PathBuf::from(std::env::var_os("DOKIME_RUNTIME_KILL_ROOT").expect("kill child root env"));
    let (_owner, runtime) = booted(&root).await;
    let created = runtime
        .create_cycle(
            CycleDefinition {
                metadata: metadata("fixture.cycle", "Fixture Cycle"),
                continue_automatically: false,
            },
            CommandContext::default(),
        )
        .await;

    assert!(created.is_ok());
    // The Crash Starts After Both Native Data And Managed State Are Durable.
    runtime.storage().flush_recovery().await.unwrap();
    crate::service_support::children::announce_ready();
    std::future::pending::<()>().await;
}

/* === Tests === */
