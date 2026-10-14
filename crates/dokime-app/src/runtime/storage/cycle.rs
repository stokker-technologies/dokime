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
    storage::{
        ObjectVersion, QueryLimits, SharedQueryValue, StorageProductionPreparation, StorageView,
    },
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
    // Object reads admit only the storage manager's default limits; each member
    // is read under them while `view` may carry the finalization budget.
    objects: QueryLimits,
    source: SharedQueryValue<TestCycle>,
    revisions: SharedQueryValue<Vec<ExecutionRevision>>,
    appearance: SharedQueryValue<RuntimeAppearanceFrame>,
    captured_at: SharedQueryValue<Timestamp>,
    owner_node_id: SharedQueryValue<NodeId>,
    _memory: StorageProductionPreparation,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl CycleSnapshot {
    pub(crate) fn capture(
        storage: &Manager,
        view: StorageView,
        objects: QueryLimits,
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
            objects,
            source,
            revisions,
            appearance,
            captured_at,
            owner_node_id,
            _memory: memory,
        })))
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
        let member = ExecutionView::open_through(
            storage,
            self.0.view.clone(),
            self.0.objects,
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
