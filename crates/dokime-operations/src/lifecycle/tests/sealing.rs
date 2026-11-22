/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Seal Publication and Retention Proof

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{CreateSnapshot, CreateSnapshotRequest, Mutate, MutateRequest, MutationItem};
use harmos_protocol::model::ReadCut;
use prost::Message;

// Local Imports
use super::{Fixture, Proof, Reading};
use crate::lifecycle::{self, admission::{Closed, DrainedIntake, Gate}, cycle, io, seal};
use dokime_protocol::host::{EvaluatedOutcome, ExecutionOutcome, ResultStatus, execution_outcome::Attribution};
use dokime_protocol::model::{Metadata, Termination};
use dokime_storage::persisted::{Scope, bindings::{self, Binding, Protobuf}, shapes::{Cycle, Execution, Preview, State}};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub(super) async fn native_closed(&self, name: &str, cycle: &str, previews: Vec<String>) -> Proof<(Gate, Closed)> {
        let operations = &self.runtime.operations;
        let mut receipt = self.native_execution(name, cycle, previews.clone()).await?;
        let execution = lifecycle::read::<Execution>(operations, name, None, self.bytes).await?.ok_or("Read Execution")?;
        let gate = Gate::new(&execution.value)?;
        let permit = gate.enter().await?;
        for key in previews {
            let value = Preview { key, scope: Scope::Execution(name).identifier()?, source_cut: Vec::new(), kind: "envelope".into(), selection: Vec::new(), payload: vec![7] };
            let response = io::call::<Mutate>(operations, MutateRequest {
                mutation: Some(self.mutation(&format!("preview/{}", value.key)).await?), items: vec![MutationItem::Create(io::encode(&value, self.bytes)?)],
            }).await?;
            receipt = response.receipt.ok_or("Return Preview Durability")?;
        }
        drop(permit);
        let closed = gate.close().await;
        assert!(gate.enter().await.is_err());
        let outcome = ExecutionOutcome { attribution: Some(Attribution::Evaluated(EvaluatedOutcome { status: ResultStatus::Passed as i32, ..Default::default() })) };
        let outcome = Protobuf::product("dokime.host.v1.ExecutionOutcome", &outcome, self.bytes)?;
        let execution = lifecycle::read::<Execution>(operations, name, None, self.bytes).await?.ok_or("Read Execution for Closing")?;
        seal::close(operations, self.mutation(&format!("close/{name}")).await?, &closed, DrainedIntake { execution: name.into(), receipt },
            execution, outcome, Termination::Completed as i32, 0, NonZeroUsize::new(1).unwrap(), self.bytes).await?;
        Ok((gate, closed))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn frozen_schemas_survive_reopen_and_refuse_changed_graphs() -> Proof {
    // Freeze a Plugin Binding Through the Library Schema Operation.
    let fixture = Fixture::open().await?;
    let (key, graph) = bindings::schema::<Reading>()?;
    let value = Protobuf::product("dokime.model.v1.ExecutionProgressValue", &dokime_protocol::model::ExecutionProgressValue::default(), fixture.bytes)?;
    let binding = Binding { key, graph, message: value.message, descriptors: value.descriptors };
    lifecycle::register(&fixture.runtime.operations, &fixture.mutation("schemas").await?).await?;
    lifecycle::register_binding(&fixture.runtime.operations, &fixture.mutation("plugin-schema").await?, &binding, fixture.bytes).await?;
    let fixture = fixture.reopen().await?;

    // A Fresh Registration Identity Reuses the Frozen Graph Without Rewriting It.
    lifecycle::register(&fixture.runtime.operations, &fixture.mutation("reopened-schemas").await?).await?;
    lifecycle::register_binding(&fixture.runtime.operations, &fixture.mutation("reopened-plugin").await?, &binding, fixture.bytes).await?;
    let mut changed = binding;
    changed.graph.definitions[changed.graph.root][0].name = "changed".into();
    assert!(lifecycle::register_binding(&fixture.runtime.operations, &fixture.mutation("changed-plugin").await?, &changed, fixture.bytes).await.is_err());
    fixture.runtime.stop().await?;
    Ok(())
}

#[tokio::test]
async fn publication_uses_receipt_s_and_retains_c_through_restart() -> Proof {
    // Close Evidence and Prepare a Single Native Seal With Enumerable Previews.
    let fixture = Fixture::open().await?;
    let operations = &fixture.runtime.operations;
    lifecycle::register(operations, &fixture.mutation("schemas").await?).await?;
    fixture.native_cycle("cycle").await?;
    let (gate, closed) = fixture.native_closed("first", "cycle", vec!["first/envelope".into(), "first/grid".into()]).await?;
    let execution = lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.ok_or("Read Closed Execution")?;
    let plan = seal::prepare(operations, fixture.mutation("seal").await?, &closed, execution, &[], 0, NonZeroUsize::new(1).unwrap(), fixture.bytes).await?;
    let evidence_cut = plan.evidence_cut.clone();
    let reader = io::call::<CreateSnapshot>(operations, CreateSnapshotRequest { at: Some(evidence_cut.clone()) }).await?;
    let reader_cut = reader.resolved_at.ok_or("Retain an Independent Reader")?;

    // S Comes Only From the Durable Receipt and Replays Without Publishing Twice.
    let receipt = seal::publish(operations, &plan, fixture.bytes).await?;
    let replay = seal::publish(operations, &plan, fixture.bytes).await?;
    assert_eq!(receipt, replay);
    assert!(receipt.durable_at.as_ref().ok_or("Return S")?.sequence > evidence_cut.positions[0].sequence);
    assert!(gate.enter().await.is_err());
    let saved = lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.ok_or("Read the Published Seal")?;
    assert_eq!(saved.value.state, State::Sealed);
    let cut = ReadCut::decode(saved.value.seal.ok_or("Retain C in the Execution")?.evidence_cut.as_slice())?;
    assert_eq!(cut, evidence_cut);
    for key in ["first/envelope", "first/grid"] {
        assert!(lifecycle::read::<Preview>(operations, key, None, fixture.bytes).await?.is_none());
        assert!(lifecycle::read::<Preview>(operations, key, Some(evidence_cut.clone()), fixture.bytes).await?.is_some());
    }
    assert!(seal::abandon(operations, plan, fixture.bytes).await.is_err());
    drop(closed);
    let fixture = fixture.reopen().await?;
    for at in [evidence_cut, reader_cut] {
        let saved = lifecycle::read::<Execution>(&fixture.runtime.operations, "first", Some(at), fixture.bytes).await?.ok_or("Reopen Retained Execution Evidence")?;
        assert_eq!(saved.value.state, State::Closed);
    }
    fixture.runtime.stop().await?;
    Ok(())
}

#[tokio::test]
async fn a_late_cycle_conflict_preserves_execution_and_every_preview() -> Proof {
    // Prepare a Seal, Then Change the Owning Cycle Through Its Native CAS Owner.
    let fixture = Fixture::open().await?;
    let operations = &fixture.runtime.operations;
    lifecycle::register(operations, &fixture.mutation("schemas").await?).await?;
    fixture.native_cycle("cycle").await?;
    let (_, closed) = fixture.native_closed("first", "cycle", vec!["first/envelope".into()]).await?;
    let execution = lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.ok_or("Read Closed Execution")?;
    let plan = seal::prepare(operations, fixture.mutation("seal").await?, &closed, execution, &[], 0, NonZeroUsize::new(1).unwrap(), fixture.bytes).await?;
    let cycle = lifecycle::read::<Cycle>(operations, "cycle", None, fixture.bytes).await?.ok_or("Read Cycle")?;
    let metadata = Protobuf::product("dokime.model.v1.Metadata", &Metadata { label: "Changed Cycle".into(), ..Default::default() }, fixture.bytes)?;
    cycle::set_metadata(operations, fixture.mutation("metadata").await?, cycle, metadata, 0, fixture.bytes).await?;

    // A Refused Compound Mutation Publishes Neither the Seal Nor Preview Deletions.
    assert!(seal::publish(operations, &plan, fixture.bytes).await.is_err());
    let saved = lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.ok_or("Keep the Closed Execution")?;
    assert_eq!(saved.value.state, State::Closed);
    assert!(saved.value.seal.is_none());
    assert!(lifecycle::read::<Preview>(operations, "first/envelope", None, fixture.bytes).await?.is_some());
    seal::abandon(operations, plan, fixture.bytes).await?;
    fixture.runtime.stop().await?;
    Ok(())
}
