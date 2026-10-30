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
use harmos::storage::{FileReader, FileReference, ObjectRevision, ObjectVersion, StorageCursor};

// Local Imports
use crate::storage::Manager;
use crate::{
    execution::context::{ExecutionContext, captured::source::CapturedSource},
    runtime::Error,
    runtime::Result,
};
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::saved::ExecutionCheckpoint;
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::execution::saved::ExecutionSource;
use dokime_domain::model::execution::saved::validate_outputs;
use dokime_domain::model::identity::NodeId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct ExecutionView {
    storage: Manager,
    saved: StorageCursor,
    execution: Option<harmos::storage::ReadBatch<ObjectRevision<TestExecution>>>,
    captured: Option<CapturedSourceOwner>,
}

/* === Enums === */

#[derive(Clone)]
pub(crate) enum CapturedSourceOwner {
    Live(Arc<CapturedSource>),
    Saved(harmos::storage::ReadBatch<CapturedSource>),
}

/* === Traits === */

/* === Implementations === */

impl std::ops::Deref for CapturedSourceOwner {
    type Target = CapturedSource;
    fn deref(&self) -> &CapturedSource {
        match self {
            Self::Live(source) => source,
            Self::Saved(source) => source,
        }
    }
}

impl From<Arc<CapturedSource>> for CapturedSourceOwner {
    fn from(source: Arc<CapturedSource>) -> Self {
        Self::Live(source)
    }
}

impl From<harmos::storage::ReadBatch<CapturedSource>> for CapturedSourceOwner {
    fn from(source: harmos::storage::ReadBatch<CapturedSource>) -> Self {
        Self::Saved(source)
    }
}

impl ExecutionView {
    pub(crate) async fn from_seal_source(storage: &Manager, seal: &dokime_domain::model::execution::saved::ExecutionSeal) -> Result<Self> {
        let source = crate::execution::context::captured::read::read_file(storage, &seal.source.context).await?;
        if source.owner_node_id != seal.source.owner_node_id
            || source.execution_id != seal.source.execution.execution_id
            || source.evidence_through != seal.source.evidence_through
        {
            return Err(Error::InvalidReportInputs("captured file differs from its execution seal".into()));
        }
        if let Some(anchor) = &source.execution
            && !seal.source.execution.position.covers(&anchor.position)?
        {
            return Err(Error::InvalidReportInputs("captured anchor follows its published revision".into()));
        }
        let source = source.try_map(|mut source| {
            source.outputs = seal.source.outputs.clone();
            Ok::<_, Error>(source)
        })?;
        Self::captured(storage, source.into(), &seal.source.owner_node_id).await
    }

    pub async fn open(storage: &Manager, id: &TestExecutionId, version: ObjectVersion) -> Result<Option<Self>> {
        let saved = storage.cursor_saved();
        let Some(execution) = crate::storage::objects::execution::read(storage, id, version).await? else {
            return Ok(None);
        };
        if let Some(seal) = &execution.value.saved {
            check_seal_position(&execution.position, &seal.dependencies_saved)?;
        }
        Ok(Some(Self {
            storage: storage.clone(),
            saved,
            execution: Some(execution),
            captured: None,
        }))
    }

    pub async fn open_checkpoint(storage: &Manager, checkpoint: &ExecutionCheckpoint) -> Result<Option<Self>> {
        checkpoint.validate().map_err(crate::storage::Error::from)?;
        let version = ObjectVersion::Exact(checkpoint.version.clone());
        let Some(inspection) = Self::open(storage, &checkpoint.execution_id, version).await? else {
            return Ok(None);
        };
        if inspection.revision()?.position != checkpoint.position
            || inspection.revision()?.version != checkpoint.version
            || inspection.revision()?.value.test_execution_id != checkpoint.execution_id
            || inspection.source()?.evidence_through != checkpoint.evidence_through
        {
            return Err(Error::Storage(crate::storage::Error::Invalid("checkpoint differs from its inspected execution seal")));
        }
        Ok(Some(inspection))
    }

    pub fn checkpoint(&self) -> Result<ExecutionCheckpoint> {
        // A Snapshot Of A Sealed Revision Keeps Its Seal; Only Pre-Report Captures Lack One.
        if self.captured.is_some() && self.execution().saved.is_none() {
            return Err(Error::Storage(crate::storage::Error::Invalid("captured pre-report execution has no sealed checkpoint")));
        }
        let revision = self.revision()?;
        let checkpoint = ExecutionCheckpoint {
            execution_id: revision.value.test_execution_id.clone(),
            position: revision.position.clone(),
            version: revision.version.clone(),
            evidence_through: self.source()?.evidence_through.clone(),
        };
        checkpoint.validate().map_err(crate::storage::Error::from)?;
        Ok(checkpoint)
    }

    pub(crate) async fn captured(storage: &Manager, captured: CapturedSourceOwner, owner: &NodeId) -> Result<Self> {
        if &captured.owner_node_id != owner {
            return Err(Error::Storage(crate::storage::Error::Invalid("captured execution source belongs to another node")));
        }
        let saved = storage.cursor_saved();
        if !saved.covers(&captured.evidence_through)? {
            return Err(Error::InvalidReportInputs("captured evidence exceeds saved view".into()));
        }
        captured.validate_header()?;
        if let Some(anchor) = &captured.execution
            && !saved.covers(&anchor.position)?
        {
            return Err(Error::InvalidReportInputs("captured anchor exceeds saved view".into()));
        }
        validate_outputs(&captured.execution_id, &saved, &captured.outputs).map_err(crate::storage::Error::from)?;
        let execution = if let Some(anchor) = &captured.execution {
            let revision =
                crate::storage::objects::execution::read(storage, &captured.execution_id, ObjectVersion::Exact(anchor.version.clone()))
                    .await?
                    .ok_or_else(|| Error::InvalidReportInputs("captured primary anchor is missing".into()))?;
            if revision.position != anchor.position
                || revision.version != anchor.version
                || revision.value.test_execution_id != captured.execution_id
                || revision.value.saved.is_some()
            {
                return Err(Error::InvalidReportInputs("captured primary anchor differs from stored revision".into()));
            }
            Some(revision)
        } else {
            None
        };
        captured.context.validate(&captured.lifecycle)?;
        // Bind Pre-Report Artifact Attribution To The Exact Unsealed Owner.
        for output in &captured.outputs {
            let ExecutionOutputContent::Artifact { record, .. } = &output.content else {
                continue;
            };
            if record.scope.test_cycle_id.as_ref().is_some_and(|id| id != &captured.lifecycle.test_cycle_id)
                || record.scope.test_case_id.as_ref().is_some_and(|id| id != &captured.lifecycle.test_case_id)
            {
                return Err(Error::Storage(crate::storage::Error::Invalid(
                    "captured artifact scope differs from its pre-report execution owner",
                )));
            }
        }
        let inspection = Self {
            storage: storage.clone(),
            saved,
            execution,
            captured: Some(captured),
        };
        for output in inspection.outputs()? {
            verify_file(inspection.read_file(&output.file, whole()).await?).await?;
        }
        Ok(inspection)
    }

    pub async fn snapshot(&self) -> Result<Self> {
        if self.captured.is_some() {
            return Ok(self.clone());
        }
        let captured = self.read_source().await?.into();
        Ok(Self { captured: Some(captured), ..self.clone() })
    }

    pub(crate) async fn into_snapshot(mut self) -> Result<Self> {
        if self.captured.is_none() {
            self.captured = Some(self.read_source().await?.into());
        }
        Ok(self)
    }

    pub(crate) fn project_captured<T>(
        self, project: impl FnOnce(ObjectRevision<TestExecution>, CapturedSource) -> Result<T>,
    ) -> Result<harmos::storage::ReadBatch<T>> {
        let captured = self.captured.ok_or_else(|| Error::InvalidReportInputs("checkpoint source is missing".into()))?;
        let revision = self.execution.ok_or_else(|| Error::InvalidReportInputs("checkpoint revision is missing".into()))?;
        match captured {
            CapturedSourceOwner::Saved(captured) => revision.try_map_with(captured, project),
            CapturedSourceOwner::Live(captured) => {
                if Arc::strong_count(&captured) != 1 || Arc::weak_count(&captured) != 0 {
                    return Err(harmos::storage::Error::InvalidInput {
                        field: "checkpoint projection",
                        reason: "requires a unique captured source",
                    }
                    .into());
                }
                revision.try_map(|revision| {
                    let captured = Arc::try_unwrap(captured).map_err(|_| harmos::storage::Error::InvalidInput {
                        field: "checkpoint projection",
                        reason: "captured source became shared",
                    })?;
                    project(revision, captured)
                })
            }
        }
    }

    pub(crate) fn project_revision<T>(
        self, project: impl FnOnce(ObjectRevision<TestExecution>) -> Result<T>,
    ) -> Result<harmos::storage::ReadBatch<T>> {
        let revision = self.execution.ok_or_else(|| Error::InvalidReportInputs("checkpoint revision is missing".into()))?;
        revision.try_map(project)
    }

    pub fn context(&self) -> Result<&ExecutionContext> {
        self.captured_source()
            .map(|source| &source.context)
            .ok_or_else(|| Error::Storage(crate::storage::Error::Invalid("execution context requires a snapshot")))
    }

    pub fn captured_at(&self) -> Result<&dokime_domain::model::time::Timestamp> {
        self.captured_source()
            .map(|source| &source.captured_at)
            .ok_or_else(|| Error::Storage(crate::storage::Error::Invalid("execution capture time requires a snapshot")))
    }

    pub(crate) fn source_snapshot(&self) -> Result<&CapturedSource> {
        self.captured_source().ok_or_else(|| Error::Storage(crate::storage::Error::Invalid("execution evidence requires a snapshot")))
    }

    pub fn storage(&self) -> &Manager {
        &self.storage
    }

    pub fn session(&self) -> &crate::storage::Manager {
        &self.storage
    }

    pub fn saved(&self) -> &StorageCursor {
        &self.saved
    }

    pub fn revision(&self) -> Result<&ObjectRevision<TestExecution>> {
        self.execution.as_deref().ok_or_else(|| Error::InvalidReportInputs("execution has no persisted primary anchor".into()))
    }

    pub fn execution(&self) -> &TestExecution {
        if let Some(execution) = &self.execution
            && (execution.value.saved.is_some() || self.captured.is_none())
        {
            return &execution.value;
        }
        &self.captured.as_ref().expect("captured view has lifecycle metadata").lifecycle
    }

    pub(crate) async fn read_source(&self) -> Result<harmos::storage::ReadBatch<CapturedSource>> {
        crate::execution::context::captured::read::read_execution(&self.storage, self.revision()?).await
    }

    pub(crate) fn source(&self) -> Result<&ExecutionSource> {
        self.revision()?
            .value
            .saved
            .as_ref()
            .map(|seal| &seal.source)
            .ok_or_else(|| Error::InvalidReportInputs("execution has no captured source dependencies".into()))
    }

    pub(crate) fn captured_source(&self) -> Option<&CapturedSource> {
        self.captured.as_deref()
    }

    pub fn evidence_through(&self) -> Result<&StorageCursor> {
        match &self.captured {
            Some(source) => Ok(&source.evidence_through),
            None => Ok(&self.source()?.evidence_through),
        }
    }

    pub(crate) fn outputs(&self) -> Result<&[ExecutionOutput]> {
        match &self.captured {
            Some(source) => Ok(&source.outputs),
            None => Ok(&self.source()?.outputs),
        }
    }

    pub async fn context_records(&self) -> Result<super::records::RecordReader> {
        super::records::RecordReader::open(self, true).await
    }

    pub async fn read_file(&self, file: &FileReference, range: harmos::storage::ByteRange) -> Result<FileReader> {
        let owned = self.captured.as_ref().is_some_and(|source| source.outputs.iter().any(|output| &output.file == file))
            || self.execution().saved.as_ref().is_some_and(|seal| seal.files().any(|owned| owned == file));
        if !owned {
            return Err(Error::Storage(crate::storage::Error::Invalid("file is not an exact captured dependency of this execution")));
        }
        if !self.saved.covers(&file.position)? {
            return Err(Error::Storage(crate::storage::Error::Invalid("execution file exceeds the original saved view")));
        }
        Ok(self
            .storage
            .read_file(harmos::storage::FileRead {
                source: harmos::storage::FileSource::Stored(file.clone()),
                range,
            })
            .await?)
    }
}

/* === Public Functions === */

pub(super) async fn verify_file(mut reader: FileReader) -> Result<()> {
    while reader.next().await?.is_some() {}
    Ok(())
}

pub(crate) fn check_seal_position(position: &StorageCursor, dependencies: &StorageCursor) -> Result<()> {
    if !position.covers(dependencies)? || position.sequence == dependencies.sequence {
        return Err(Error::Storage(crate::storage::Error::Invalid("seal revision does not follow saved dependencies")));
    }
    Ok(())
}

/* === Private Functions === */

fn whole() -> harmos::storage::ByteRange {
    harmos::storage::ByteRange { offset: 0, limit: None }
}

/* === Tests === */
