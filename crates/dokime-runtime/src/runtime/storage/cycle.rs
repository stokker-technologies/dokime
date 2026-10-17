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
use std::sync::Arc;

// Third Party Imports
use harmos::storage::StorageCursor;

// Local Imports
use crate::{runtime::Error, runtime::Result, installed_plugins::RuntimeAppearanceFrame};
use dokime_domain::model::identity::NodeId;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::time::Timestamp;
use crate::execution::TestCycle;
use crate::execution::saved::ExecutionRevision;
use crate::execution::saved::validate_cycle_revisions;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// A cycle frozen for its report: header, member revisions and presentation.
#[derive(Clone)]
pub(crate) struct CycleSnapshot(Arc<State>);

struct State {
    saved: StorageCursor,
    source: Arc<TestCycle>,
    revisions: Arc<Vec<ExecutionRevision>>,
    appearance: Arc<RuntimeAppearanceFrame>,
    captured_at: Timestamp,
    owner_node_id: NodeId,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl CycleSnapshot {
    pub(crate) fn capture(
        saved: StorageCursor,
        source: Arc<TestCycle>,
        revisions: Arc<Vec<ExecutionRevision>>,
        appearance: Arc<RuntimeAppearanceFrame>,
        owner_node_id: &NodeId,
    ) -> Result<Self> {
        if source.saved.is_some() || source.session_id.is_none() {
            return Err(invalid(
                "final cycle snapshot requires an unsealed source and frozen session",
            ));
        }
        validate_cycle_revisions(&source, &saved, &revisions)
            .map_err(crate::storage::Error::from)?;
        // A Final Report Is Generated At Its Frozen Cycle's Last Update; The
        // Durable Publication Check Refuses Any Other Header Time.
        let captured_at = source.updated_at.clone();
        Ok(Self(Arc::new(State {
            saved,
            source,
            revisions,
            appearance,
            captured_at,
            owner_node_id: owner_node_id.clone(),
        })))
    }

    /// Frozen Presentation Metadata For An Unsealed Cycle Report.
    /// The Operation Supplies Its Separately Materialized Member Inputs.
    pub(crate) fn current(
        saved: StorageCursor,
        source: Arc<TestCycle>,
        appearance: Arc<RuntimeAppearanceFrame>,
        owner_node_id: &NodeId,
    ) -> Result<Self> {
        if source.saved.is_some() || source.session_id.is_none() {
            return Err(invalid(
                "current cycle snapshot requires an unsealed source and its session",
            ));
        }
        Ok(Self(Arc::new(State {
            saved,
            source,
            revisions: Arc::new(Vec::new()),
            appearance,
            captured_at: crate::model::time::now_utc(),
            owner_node_id: owner_node_id.clone(),
        })))
    }

    /// The saved cut the snapshot was frozen at.
    pub(crate) fn saved(&self) -> &StorageCursor {
        &self.0.saved
    }
    pub(crate) fn source(&self) -> &Arc<TestCycle> {
        &self.0.source
    }
    pub(crate) fn revisions(&self) -> &Arc<Vec<ExecutionRevision>> {
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
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(message: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(message))
}

/* === Tests === */
