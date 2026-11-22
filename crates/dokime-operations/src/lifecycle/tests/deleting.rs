/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Execution Deletion and Independent Reader Proof

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{
    CreateFileProduction, CreateFileProductionRequest, CreateSnapshot, CreateSnapshotRequest, DeleteFileProduction,
    DeleteFileProductionRequest, DeleteSnapshot, DeleteSnapshotRequest,
};
use harmos_runtime::Identifier;
use harmos_runtime::storage::FileMetadata;

// Local Imports
use super::{Fixture, Proof};
use crate::lifecycle::{
    self, cycle,
    deletion::{self, Guard},
    io, seal,
};
use dokime_storage::persisted::{
    Scope,
    shapes::{Cycle, Execution, State},
};

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
async fn deletion_guards_owners_and_productions_and_releases_only_its_cut() -> Proof {
    // Seal Two Independently Owned Executions in the Same Cycle.
    let fixture = Fixture::open().await?;
    let operations = &fixture.runtime.operations;
    let rows = NonZeroUsize::new(1).unwrap();
    lifecycle::register(operations, &fixture.mutation("schemas").await?).await?;
    fixture.native_cycle("cycle").await?;
    let (_, first_closed) = fixture.native_closed("first", "cycle", Vec::new()).await?;
    let first = lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.ok_or("Read First Execution")?;
    let first_plan =
        seal::prepare(operations, fixture.mutation("seal-first").await?, &first_closed, first, &[], 0, rows, fixture.bytes).await?;
    let first_cut = first_plan.evidence_cut.clone();
    seal::publish(operations, &first_plan, fixture.bytes).await?;
    let reader = io::call::<CreateSnapshot>(operations, CreateSnapshotRequest { at: Some(first_cut.clone()) }).await?;
    let reader_cut = reader.resolved_at.ok_or("Retain the Reader Cut")?;
    let reader_snapshot = reader.snapshot.ok_or("Retain the Reader Lease")?;
    let (_, second_closed) = fixture.native_closed("second", "cycle", Vec::new()).await?;
    let second = lifecycle::read::<Execution>(operations, "second", None, fixture.bytes).await?.ok_or("Read Second Execution")?;
    let second_plan =
        seal::prepare(operations, fixture.mutation("seal-second").await?, &second_closed, second, &[], 0, rows, fixture.bytes).await?;
    let second_cut = second_plan.evidence_cut.clone();
    seal::publish(operations, &second_plan, fixture.bytes).await?;

    // Ownership and Unfinished File Productions Refuse Deletion Before Its Mutation.
    let first = lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.ok_or("Read Sealed First Execution")?;
    let first_seal = first.value.seal.clone().ok_or("Read the Current Seal")?;
    let wrong = Guard {
        case: "case-second".into(),
        cycle: "cycle".into(),
        seal: first_seal.clone(),
    };
    assert!(
        deletion::prepare(operations, fixture.mutation("wrong-owner").await?, &first_closed, first.clone(), wrong, rows).await.is_err()
    );
    let response = io::call::<CreateFileProduction>(
        operations,
        CreateFileProductionRequest {
            mutation: Some(fixture.mutation("file-production").await?),
            metadata: Some(FileMetadata {
                id: Identifier::new("pending-file")?,
                scope: Some(Scope::Execution("first").identifier()?),
                media_type: "application/octet-stream".into(),
                expected_length: Some(1),
                expected_sha256: None,
            }),
        },
    )
    .await?;
    let production = response.production.ok_or("Return the Admitted File Writer")?;
    let guard = Guard {
        case: first.value.case.clone(),
        cycle: first.value.cycle.clone(),
        seal: first_seal.clone(),
    };
    assert!(
        deletion::prepare(operations, fixture.mutation("pending-production").await?, &first_closed, first.clone(), guard, rows)
            .await
            .is_err()
    );
    io::call::<DeleteFileProduction>(
        operations,
        DeleteFileProductionRequest {
            mutation: Some(fixture.mutation("abandon-file").await?),
            session: Some(production.session),
        },
    )
    .await?;

    // DeleteScope Is the Only Effect; Replay Returns the Original Receipt.
    let guard = Guard {
        case: first.value.case.clone(),
        cycle: first.value.cycle.clone(),
        seal: first_seal,
    };
    let plan = deletion::prepare(operations, fixture.mutation("delete-first").await?, &first_closed, first, guard, rows).await?;
    let receipt = deletion::delete(operations, &plan, fixture.bytes).await?;
    assert_eq!(receipt, deletion::delete(operations, &plan, fixture.bytes).await?);
    assert!(lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.is_none());
    assert!(lifecycle::read::<Execution>(operations, "second", None, fixture.bytes).await?.is_some());
    let image = cycle::image(operations, "cycle", None, rows, fixture.bytes).await?;
    let cycle = lifecycle::read::<Cycle>(operations, "cycle", None, fixture.bytes).await?.ok_or("Read Cycle")?;
    assert_ne!(image.outcomes, cycle.value.outcomes);
    assert!(image.settled);
    cycle::finish(operations, fixture.mutation("finish-cycle").await?, cycle, 0, rows, fixture.bytes).await?;
    drop(first_closed);
    drop(second_closed);

    // Independent Readers and Unrelated Execution Retention Survive Restart.
    let fixture = fixture.reopen().await?;
    let operations = &fixture.runtime.operations;
    let saved = lifecycle::read::<Execution>(operations, "first", Some(reader_cut.clone()), fixture.bytes)
        .await?
        .ok_or("Read Deleted Evidence Through Its Reader Lease")?;
    assert_eq!(saved.value.state, State::Closed);
    assert!(lifecycle::read::<Execution>(operations, "second", Some(second_cut), fixture.bytes).await?.is_some());
    assert!(lifecycle::read::<Execution>(operations, "first", Some(first_cut), fixture.bytes).await.is_err());
    io::call::<DeleteSnapshot>(operations, DeleteSnapshotRequest { snapshot: Some(reader_snapshot) }).await?;
    let fixture = fixture.reopen().await?;
    assert!(lifecycle::read::<Execution>(&fixture.runtime.operations, "first", Some(reader_cut), fixture.bytes).await.is_err());
    fixture.runtime.stop().await?;
    Ok(())
}
