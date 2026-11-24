/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Lifecycle Transition Proof

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{Mutate, MutateRequest, MutationItem, WatchRows, WatchRowsResponse, watch_rows_response::Event};
use harmos_protocol::model::{Declarations, PageRequest, Receipt};
use harmos_runtime::operations::{NativeInvocation, StreamInput, StreamSettings, native_stream};
use harmos_runtime::storage::query::ObjectVersions;
use prost::Message;

// Local Imports
use super::{Fixture, Proof};
use crate::lifecycle::{self, admission::Gate, cycle, execution, io, rows};
use dokime_protocol::host::PlannedCase;
use dokime_protocol::model::{ExecutionProgressValue, Import, Metadata, Origin, TestCase, origin};
use dokime_storage::persisted::{
    Case, Scope,
    bindings::Protobuf,
    shapes::{Cycle, Execution, Parameters, Progress, State, Steps},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub(super) async fn native_cycle(&self, name: &str) -> Proof<Receipt> {
        let value = Cycle {
            key: name.into(),
            scope: Scope::Cycle(name).identifier()?,
            metadata: Protobuf::product("dokime.model.v1.Metadata", &Metadata::default(), self.bytes)?,
            sealed: false,
            outcomes: Default::default(),
        };
        cycle::create(&self.runtime.operations, self.mutation(&format!("cycle/{name}")).await?, value, 0, self.bytes)
            .await
            .map_err(Into::into)
    }

    pub(super) async fn native_admission(&self, name: &str, cycle: &str, previews: Vec<String>) -> Proof<execution::Plan> {
        let operations = &self.runtime.operations;
        let case = format!("case-{name}");
        let origin = Origin {
            kind: Some(origin::Kind::Import(Import {
                importer: "thermal.importer".into(),
                metadata: Some(Metadata::default()),
                ..Default::default()
            })),
        };
        let planned = PlannedCase {
            case_id: case.clone(),
            origin: Some(origin.clone()),
            ..Default::default()
        };
        let saved = Case {
            key: case.clone(),
            scope: Scope::Case(&case).identifier()?,
            cycle: cycle.into(),
            payload: planned.encode_to_vec(),
        };
        let request = MutateRequest {
            mutation: Some(self.mutation(&format!("case/{name}")).await?),
            items: vec![MutationItem::Create(io::encode(&saved, self.bytes)?)],
        };
        io::call::<Mutate>(operations, request).await?;
        let scope = Scope::Execution(name).identifier()?;
        let execution = Execution {
            key: name.into(),
            scope: scope.clone(),
            case,
            cycle: cycle.into(),
            origin: Protobuf::product("dokime.model.v1.Origin", &origin, self.bytes)?,
            catalog: Declarations::default().encode_to_vec(),
            state: State::Open,
            outcome: None,
            termination: 0,
            seal: None,
            previews,
        };
        let parameters = Parameters {
            key: name.into(),
            scope: scope.clone(),
            value: Protobuf::product("dokime.model.v1.TestCase", &TestCase::default(), self.bytes)?,
        };
        let steps = Steps {
            key: name.into(),
            scope: scope.clone(),
            value: Protobuf::product("dokime.model.v1.TestCase", &TestCase::default(), self.bytes)?,
        };
        let progress = Progress {
            key: name.into(),
            scope,
            value: Protobuf::product("dokime.model.v1.ExecutionProgressValue", &ExecutionProgressValue::default(), self.bytes)?,
        };
        let admission = execution::Admission {
            execution,
            parameters,
            steps,
            progress,
            streams: Vec::new(),
            receipt_ns: 0,
        };
        execution::prepare(operations, self.mutation(&format!("admit/{name}")).await?, admission, self.bytes).await.map_err(Into::into)
    }

    pub(super) async fn native_execution(&self, name: &str, cycle: &str, previews: Vec<String>) -> Proof<Receipt> {
        let plan = self.native_admission(name, cycle, previews).await?;
        execution::admit(&self.runtime.operations, &plan, self.bytes).await.map_err(Into::into)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/// Receive One Watch Event; a Resync or a Closed Stream Names How the Watch Ended.
async fn frame(input: &StreamInput<WatchRowsResponse>, watching: &mut NativeInvocation<WatchRows>) -> Proof<Event> {
    let Some(frame) = input.recv().await else {
        let ended = watching.await.map(|response| response.event.clone());
        return Err(format!("Keep the Native Revision Watch Open; It Ended With {ended:?}").into());
    };
    match frame.event.clone() {
        Some(Event::ResyncRequired(reason)) => Err(format!("Keep the Native Revision Watch Current: {reason}").into()),
        event => event.ok_or_else(|| "Return a Native Revision Watch Event".into()),
    }
}

/* === Tests === */

#[tokio::test]
async fn admission_replays_and_stale_cycle_conditions_refuse_every_item() -> Proof {
    // Register and Create a Cycle Through Its Native Transition Owner.
    let fixture = Fixture::open().await?;
    let operations = &fixture.runtime.operations;
    lifecycle::register(operations, &fixture.mutation("schemas").await?).await?;
    fixture.native_cycle("cycle").await?;
    let first = fixture.native_admission("first", "cycle", Vec::new()).await?;
    let stale = fixture.native_admission("stale", "cycle", Vec::new()).await?;

    // Admission Updates the Cycle Guard; Its Exact Plan Replays the Original Receipt.
    let receipt = execution::admit(operations, &first, fixture.bytes).await?;
    let replay = execution::admit(operations, &first, fixture.bytes).await?;
    assert_eq!(receipt, replay);
    assert!(execution::admit(operations, &stale, fixture.bytes).await.is_err());
    assert!(lifecycle::read::<Execution>(operations, "stale", None, fixture.bytes).await?.is_none());
    assert!(lifecycle::read::<Progress>(operations, "stale", None, fixture.bytes).await?.is_none());

    // Read a Bounded Opening Image and Register Its Native Revision Watch.
    let mut page =
        rows::list::<Execution>(operations, None, ObjectVersions::Latest, None, PageRequest { page_size: 1, ..Default::default() }).await?;
    let at = page.resolved_at.clone().ok_or("Return the Opening Image Cut")?;
    let (output, input) = native_stream(StreamSettings {
        items: NonZeroUsize::new(8).unwrap(),
        bytes: fixture.bytes,
    })?;
    let mut watching = rows::watch::<Execution>(operations, None, at.clone(), output)?;
    let mut found = false;
    loop {
        found |= rows::decode::<Execution>(&page, fixture.bytes)?.iter().any(|value| value.key == "first");
        if page.next_page_token.is_empty() {
            break;
        }
        page = rows::list::<Execution>(
            operations,
            None,
            ObjectVersions::Latest,
            Some(at.clone()),
            PageRequest {
                page_size: 1,
                page_token: page.next_page_token,
                ..Default::default()
            },
        )
        .await?;
    }
    assert!(found);
    let next = fixture.native_admission("next", "cycle", Vec::new()).await?;
    execution::admit(operations, &next, fixture.bytes).await?;
    let mut observed = false;
    loop {
        match frame(&input, &mut watching).await? {
            Event::Object(object) => observed |= object.value.item.as_str() == "next",
            Event::DurableProgress(()) if observed => break,
            _ => {}
        }
    }
    drop(input);
    watching.request_cancel();
    assert!(watching.await.is_err());

    // A Held Image Does Not Include the Later Admission.
    assert!(lifecycle::read::<Execution>(operations, "next", Some(at), fixture.bytes).await?.is_none());
    let first = lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.ok_or("Read the Native Execution")?;
    assert_eq!(first.value.cycle, "cycle");
    fixture.runtime.stop().await?;
    Ok(())
}

#[tokio::test]
async fn resource_revisions_commit_with_execution_guards() -> Proof {
    // Open a Native Execution and Hold Its Resource Revisions.
    let fixture = Fixture::open().await?;
    let operations = &fixture.runtime.operations;
    lifecycle::register(operations, &fixture.mutation("schemas").await?).await?;
    fixture.native_cycle("cycle").await?;
    fixture.native_execution("first", "cycle", Vec::new()).await?;
    let execution = lifecycle::read::<Execution>(operations, "first", None, fixture.bytes).await?.ok_or("Read Execution")?;
    let progress = lifecycle::read::<Progress>(operations, "first", None, fixture.bytes).await?.ok_or("Read Progress")?;
    let gate = Gate::new(&execution.value)?;
    let permit = gate.enter().await?;
    let value = Protobuf::product(
        "dokime.model.v1.ExecutionProgressValue",
        &ExecutionProgressValue {
            completed: 7,
            total: 11,
            ..Default::default()
        },
        fixture.bytes,
    )?;
    let mutation = fixture.mutation("progress").await?;

    // Preserve the Original CAS Input for Replay; a New Identity Cannot Reuse It.
    let revision = execution::Revision {
        mutation: mutation.clone(),
        permit: &permit,
        execution: execution.clone(),
        receipt_ns: 0,
    };
    let receipt = execution::set_progress(operations, revision, progress.clone(), value.clone(), fixture.bytes).await?;
    let revision = execution::Revision {
        mutation,
        permit: &permit,
        execution: execution.clone(),
        receipt_ns: 0,
    };
    let replay = execution::set_progress(operations, revision, progress.clone(), value.clone(), fixture.bytes).await?;
    assert_eq!(receipt, replay);
    let revision = execution::Revision {
        mutation: fixture.mutation("stale-progress").await?,
        permit: &permit,
        execution,
        receipt_ns: 0,
    };
    assert!(execution::set_progress(operations, revision, progress, value, fixture.bytes).await.is_err());
    let progress = lifecycle::read::<Progress>(operations, "first", None, fixture.bytes).await?.ok_or("Read Durable Progress")?;
    let value = ExecutionProgressValue::decode(progress.value.value.payload.as_slice())?;
    assert_eq!(value.completed, 7);
    let image = cycle::image(operations, "cycle", None, NonZeroUsize::new(1).unwrap(), fixture.bytes).await?;
    assert!(!image.settled);
    fixture.runtime.stop().await?;
    Ok(())
}
