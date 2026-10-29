/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Pin Sealed Evidence Revisions And Verify Exact Dependencies

use harmos::storage::{ObjectRevision, ObjectVersion, ReadBatch};
use serde::Serialize;

use super::error::{Error, Result};
use crate::{runtime::storage::execution::ExecutionView, storage::Manager};
use dokime_domain::model::{
    execution::{result::CyclePhase, saved::ExecutionRevision},
    exports::EvidenceSubject,
    identity::{NodeId, TestCycleId, TestExecutionId},
};

pub(super) struct Snapshot {
    pub subject: EvidenceSubject,
    pub node_id: NodeId,
    pub cycle_id: TestCycleId,
    pub cycle: serde_json::Value,
    pub cycle_revision: Option<CycleRevision>,
    pub cycle_report: Option<harmos::storage::FileReference>,
    pub executions: Vec<ExecutionRevision>,
    pub filename: String,
    storage: Manager,
}

pub(super) struct CycleRevision {
    source: ReadBatch<ObjectRevision<dokime_domain::model::execution::result::TestCycle>>,
}

impl Serialize for CycleRevision {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut revision = serializer.serialize_struct("CycleRevision", 3)?;
        revision.serialize_field("cycle_id", &self.source.value.test_cycle_id)?;
        revision.serialize_field("position", &self.source.position)?;
        revision.serialize_field("version", &self.source.version)?;
        revision.end()
    }
}

impl Snapshot {
    pub async fn open(storage: &Manager, subject: EvidenceSubject) -> Result<Self> {
        let session = storage.clone();
        match &subject {
            EvidenceSubject::Execution(value) => {
                let id = TestExecutionId::from_uuid(value).map_err(|_| Error::Invalid("execution identity must be a UUID"))?;
                let view = ExecutionView::open(storage, &id, ObjectVersion::Latest)
                    .await?
                    .ok_or_else(|| Error::Missing { kind: "execution", id: id.to_string() })?;
                if view.execution().saved.is_none() {
                    return Err(Error::Unsealed("execution"));
                }
                let view = view.snapshot().await?;
                let source = view.revision()?;
                let execution = &source.value;
                let selected = ExecutionRevision {
                    execution_id: id.clone(),
                    position: source.position.clone(),
                    version: source.version.clone(),
                };
                Ok(Self {
                    subject: EvidenceSubject::Execution(id.to_string()),
                    node_id: view.source_snapshot()?.owner_node_id.clone(),
                    cycle_id: execution.test_cycle_id.clone(),
                    cycle: serde_json::to_value(&view.context()?.cycle.cycle)?,
                    cycle_revision: None,
                    cycle_report: None,
                    executions: vec![selected],
                    filename: format!("execution-{id}.zip"),
                    storage: storage.clone(),
                })
            }
            EvidenceSubject::Cycle(value) => {
                let id = TestCycleId::from_uuid(value).map_err(|_| Error::Invalid("cycle identity must be a UUID"))?;
                let source = crate::storage::objects::cycle::read(&session, &id, ObjectVersion::Latest)
                    .await?
                    .ok_or_else(|| Error::Missing { kind: "cycle", id: id.to_string() })?;
                if source.value.phase != CyclePhase::Sealed {
                    return Err(Error::Unsealed("cycle"));
                }
                let seal = source.value.saved.as_ref().ok_or(Error::Unsealed("cycle"))?;
                crate::runtime::storage::execution::check_seal_position(&source.position, &seal.dependencies_saved)?;
                let executions = seal.executions.clone();
                let cycle_report = Some(seal.report.clone());
                Ok(Self {
                    subject: EvidenceSubject::Cycle(id.to_string()),
                    node_id: seal.owner_node_id.clone(),
                    cycle_id: id.clone(),
                    cycle: serde_json::to_value(&source.value)?,
                    cycle_revision: Some(CycleRevision { source }),
                    cycle_report,
                    executions,
                    filename: format!("cycle-{id}.zip"),
                    storage: storage.clone(),
                })
            }
        }
    }

    pub async fn execution(&self, selected: &ExecutionRevision) -> Result<ExecutionView> {
        let view = ExecutionView::open(&self.storage, &selected.execution_id, ObjectVersion::Exact(selected.version.clone()))
            .await?
            .ok_or_else(|| Error::Missing {
                kind: "execution revision",
                id: selected.execution_id.to_string(),
            })?;
        let revision = view.revision()?;
        if revision.position != selected.position
            || revision.version != selected.version
            || revision.value.test_execution_id != selected.execution_id
            || revision.value.test_cycle_id != self.cycle_id
        {
            return Err(Error::Invalid("cycle membership differs from its exact saved execution revision"));
        }
        if revision.value.saved.is_none() {
            return Err(Error::Unsealed("execution"));
        }
        let view = view.snapshot().await?;
        if view.source_snapshot()?.owner_node_id != self.node_id {
            return Err(Error::Invalid("execution node differs from the sealed cycle owner"));
        }
        Ok(view)
    }

    pub fn storage(&self) -> &Manager {
        &self.storage
    }
}
