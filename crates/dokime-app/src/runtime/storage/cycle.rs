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
use std::{num::NonZeroUsize, sync::Arc};

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{ObjectVersion, SharedQueryValue, StorageProductionPreparation, StorageView},
};

// Local Imports
use super::execution::ExecutionView;
use crate::{Error, Result, installed_plugins::RuntimeAppearanceFrame};
use dokime::{
    core::{NodeId, SessionId, Timestamp},
    execution::{
        TestCycle,
        saved::{ExecutionRevision, validate_cycle_revisions},
    },
    storage::Manager,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub(crate) struct CycleSnapshot(Arc<State>);

struct State {
    view: StorageView,
    source: SharedQueryValue<TestCycle>,
    revisions: SharedQueryValue<Vec<ExecutionRevision>>,
    appearance: SharedQueryValue<RuntimeAppearanceFrame>,
    captured_at: SharedQueryValue<Timestamp>,
    owner_node_id: SharedQueryValue<NodeId>,
    /// The members of a cycle that is not sealed, captured as they stand;
    /// `None` for the sealed revisions a final report reads.
    current: Option<Vec<ExecutionView>>,
    _memory: StorageProductionPreparation,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl CycleSnapshot {
    pub(crate) fn capture(
        storage: &Manager,
        view: StorageView,
        source: SharedQueryValue<TestCycle>,
        revisions: SharedQueryValue<Vec<ExecutionRevision>>,
        appearance: SharedQueryValue<RuntimeAppearanceFrame>,
        owner_node_id: &NodeId,
    ) -> Result<Self> {
        source.check_origin(storage)?;
        revisions.check_origin(storage)?;
        appearance.check_origin(storage)?;
        let unit = NonZeroUsize::new(1).unwrap();
        // Verify Actual View Ownership Without Replacing Its Pin Or Read Ledger.
        view.project_shared(&source, unit, |_, _| Ok(()))?;
        view.project_shared(&revisions, unit, |_, _| Ok(()))?;
        view.project_shared(&appearance, unit, |_, _| Ok(()))?;
        // Projections Charge Their Own View Computation, So Validation Releases Its Guard First.
        let work = view.begin_computation()?;
        if source.saved.is_some() || source.session_id.is_none() {
            return Err(invalid(
                "final cycle snapshot requires an unsealed source and frozen session",
            ));
        }
        validate_cycle_revisions(&source, &view.saved()?, &revisions)
            .map_err(dokime::storage::Error::from)?;
        work.check()?;
        drop(work);
        // A Final Report Is Generated At Its Frozen Cycle's Last Update; The
        // Durable Publication Check Refuses Any Other Header Time.
        let captured_at = view
            .project_shared(
                &source,
                bound(source.updated_at.memory_size().saturating_add(64))?,
                |source, _| Ok(source.updated_at.clone()),
            )?
            .try_into_shared()?;
        let owner_node_id = view
            .project_shared(&source, bound(owner_node_id.memory_size())?, |_, _| {
                Ok(owner_node_id.clone())
            })?
            .try_into_shared()?;
        let memory = storage.prepare_production(bound(
            std::mem::size_of::<State>() + 2 * std::mem::size_of::<usize>(),
        )?)?;
        Ok(Self(Arc::new(State {
            view,
            source,
            revisions,
            appearance,
            captured_at,
            owner_node_id,
            current: None,
            _memory: memory,
        })))
    }

    /// A snapshot of a cycle that is not sealed, over members captured as
    /// they stand. It renders a report from current evidence and never seals.
    pub(crate) fn current(
        storage: &Manager,
        view: StorageView,
        source: SharedQueryValue<TestCycle>,
        members: Vec<ExecutionView>,
        appearance: SharedQueryValue<RuntimeAppearanceFrame>,
        owner_node_id: &NodeId,
    ) -> Result<Self> {
        source.check_origin(storage)?;
        appearance.check_origin(storage)?;
        if source.saved.is_some() || source.session_id.is_none() {
            return Err(invalid(
                "current cycle snapshot requires an unsealed source and its session",
            ));
        }
        for member in &members {
            if member.execution().test_cycle_id != source.test_cycle_id {
                return Err(invalid("current cycle snapshot member belongs elsewhere"));
            }
        }
        let revisions = storage
            .produce(
                bound(std::mem::size_of::<Vec<ExecutionRevision>>())?,
                |_| Ok(Vec::new()),
            )?
            .try_into_value()?
            .try_into_shared()?;
        let now = Timestamp::now_utc();
        let captured_at = storage
            .produce(bound(now.memory_size().saturating_add(64))?, |_| Ok(now))?
            .try_into_value()?
            .try_into_shared()?;
        let owner_node_id = storage
            .produce(bound(owner_node_id.memory_size())?, |_| {
                Ok(owner_node_id.clone())
            })?
            .try_into_value()?
            .try_into_shared()?;
        let memory = storage.prepare_production(bound(
            std::mem::size_of::<State>()
                .saturating_add(2 * std::mem::size_of::<usize>())
                .saturating_add(
                    members
                        .capacity()
                        .saturating_mul(std::mem::size_of::<ExecutionView>()),
                ),
        )?)?;
        Ok(Self(Arc::new(State {
            view,
            source,
            revisions,
            appearance,
            captured_at,
            owner_node_id,
            current: Some(members),
            _memory: memory,
        })))
    }

    /// Whether the members are the sealed revisions a final report reads.
    pub(crate) fn is_final(&self) -> bool {
        self.0.current.is_none()
    }

    /// How many members the report covers.
    pub(crate) fn len(&self) -> usize {
        match &self.0.current {
            Some(members) => members.len(),
            None => self.0.revisions.len(),
        }
    }

    pub(crate) fn view(&self) -> &StorageView {
        &self.0.view
    }
    pub(crate) fn source(&self) -> &SharedQueryValue<TestCycle> {
        &self.0.source
    }
    pub(crate) fn revisions(&self) -> &SharedQueryValue<Vec<ExecutionRevision>> {
        &self.0.revisions
    }
    pub(crate) fn appearance(&self) -> &RuntimeAppearanceFrame {
        &self.0.appearance
    }
    pub(crate) fn captured_at(&self) -> &Timestamp {
        &self.0.captured_at
    }
    pub(crate) fn owner_node_id(&self) -> &NodeId {
        &self.0.owner_node_id
    }
    pub(crate) fn session_id(&self) -> &SessionId {
        self.0
            .source
            .session_id
            .as_ref()
            .expect("validated frozen cycle session")
    }

    pub(crate) async fn member(&self, storage: &Manager, index: usize) -> Result<ExecutionView> {
        if let Some(members) = &self.0.current {
            return members
                .get(index)
                .ok_or_else(|| invalid("cycle snapshot member index is out of bounds"))?
                .snapshot(storage)
                .await;
        }
        let revision = self
            .0
            .revisions
            .get(index)
            .ok_or_else(|| invalid("cycle snapshot member index is out of bounds"))?;
        let _selection = storage.prepare_production(bound(
            revision
                .version
                .memory_size()
                .saturating_add(std::mem::size_of::<ObjectVersion>()),
        )?)?;
        let member = ExecutionView::open(
            storage,
            self.0.view.clone(),
            &revision.execution_id,
            ObjectVersion::Exact(revision.version.clone()),
            self.0.view.decoded_bytes()?,
        )
        .await?
        .ok_or_else(|| invalid("cycle snapshot member is missing"))?;
        let work = self.0.view.begin_computation()?;
        if member.revision().position != revision.position
            || member.execution().test_cycle_id != self.0.source.test_cycle_id
            || member.execution().saved.is_none()
        {
            return Err(invalid(
                "cycle snapshot member revision, owner or seal differs",
            ));
        }
        work.check()?;
        drop(work);
        member.snapshot(storage).await
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|_| bytes != usize::MAX)
        .ok_or_else(|| invalid("cycle snapshot memory overflows"))
}

fn invalid(message: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(message))
}

/* === Tests === */
