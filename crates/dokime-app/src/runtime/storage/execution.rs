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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{
        FileReference, ObjectRevision, ObjectVersion, ObjectVersionToken, QueryValue, ReadLimits,
        SharedQueryValue, StorageCursor, StorageFileReader, StorageView,
    },
};

// Local Imports
use crate::{
    Error, Result,
    execution::context::{ExecutionContext, source::CapturedSource},
};
use dokime::{
    core::{NodeId, TestExecutionId},
    execution::{
        TestExecution,
        saved::{
            ExecutionCheckpoint, ExecutionOutput, ExecutionOutputContent, ExecutionSource,
            output_validation_memory, validate_outputs,
        },
    },
    storage::Manager,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct ExecutionView {
    view: StorageView,
    execution: Option<SharedQueryValue<ObjectRevision<TestExecution>>>,
    captured: Option<SharedQueryValue<CapturedSource>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionView {
    pub async fn open(
        storage: &Manager,
        view: StorageView,
        id: &TestExecutionId,
        version: ObjectVersion,
        limit: NonZeroUsize,
    ) -> Result<Option<Self>> {
        let Some(execution) =
            dokime::storage::objects::execution::read_in(storage, &view, id, version, limit)
                .await?
        else {
            return Ok(None);
        };
        Self::opened(view, execution)
    }

    fn opened(
        view: StorageView,
        execution: QueryValue<ObjectRevision<TestExecution>>,
    ) -> Result<Option<Self>> {
        let validation = view.begin_computation()?;
        // Native Point Reads Verify Exact Revision Visibility Within This View
        if let Some(seal) = &execution.value.saved {
            check_seal_position(&execution.position, &seal.dependencies_saved)?;
        }
        let execution = execution.try_into_shared()?;
        validation.check()?;
        drop(validation);
        Ok(Some(Self {
            view,
            execution: Some(execution),
            captured: None,
        }))
    }

    pub async fn open_checkpoint(
        storage: &Manager,
        view: StorageView,
        checkpoint: &ExecutionCheckpoint,
        limit: NonZeroUsize,
    ) -> Result<Option<Self>> {
        let validation = view.begin_computation()?;
        checkpoint
            .validate()
            .map_err(dokime::storage::Error::from)?;
        let selection = checkpoint
            .version
            .memory_size()
            .checked_add(std::mem::size_of::<ObjectVersion>())
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| {
                Error::Storage(dokime::storage::Error::Invalid(
                    "execution checkpoint selector memory overflow",
                ))
            })?;
        let _selection = storage.prepare_production(selection)?;
        let version = ObjectVersion::Exact(checkpoint.version.clone());
        validation.check()?;
        drop(validation);
        let Some(inspection) =
            Self::open(storage, view, &checkpoint.execution_id, version, limit).await?
        else {
            return Ok(None);
        };
        let validation = inspection.view.begin_computation()?;
        if inspection.revision()?.position != checkpoint.position
            || inspection.revision()?.version != checkpoint.version
            || inspection.revision()?.value.test_execution_id != checkpoint.execution_id
            || inspection.source()?.evidence_through != checkpoint.evidence_through
        {
            return Err(Error::Storage(dokime::storage::Error::Invalid(
                "checkpoint differs from its inspected execution seal",
            )));
        }
        validation.check()?;
        drop(validation);
        Ok(Some(inspection))
    }

    pub fn checkpoint(&self, limit: NonZeroUsize) -> Result<QueryValue<ExecutionCheckpoint>> {
        // A Snapshot Of A Sealed Revision Keeps Its Seal; Only Pre-Report Captures Lack One.
        if self.captured.is_some() && self.execution().saved.is_none() {
            return Err(Error::Storage(dokime::storage::Error::Invalid(
                "captured pre-report execution has no sealed checkpoint",
            )));
        }
        let source = self.source()?;
        let checkpoint =
            self.view
                .project_shared(self.revision()?, limit, |revision, allowance| {
                    let fields = [
                        (
                            revision.value.test_execution_id.memory_size(),
                            std::mem::size_of::<TestExecutionId>(),
                        ),
                        (
                            revision.position.memory_size(),
                            std::mem::size_of::<StorageCursor>(),
                        ),
                        (
                            revision.version.memory_size(),
                            std::mem::size_of::<ObjectVersionToken>(),
                        ),
                        (
                            source.evidence_through.memory_size(),
                            std::mem::size_of::<StorageCursor>(),
                        ),
                    ];
                    let bytes = fields.into_iter().try_fold(
                        std::mem::size_of::<ExecutionCheckpoint>(),
                        |bytes, (owned, inline)| {
                            if owned == usize::MAX {
                                return None;
                            }
                            bytes.checked_add(owned.saturating_sub(inline))
                        },
                    );
                    if bytes.is_none_or(|bytes| bytes > allowance.get()) {
                        return Err(harmos::Error::ResourceExhausted {
                            resource: "execution checkpoint projection memory",
                        });
                    }
                    let checkpoint = ExecutionCheckpoint {
                        execution_id: revision.value.test_execution_id.clone(),
                        position: revision.position.clone(),
                        version: revision.version.clone(),
                        evidence_through: source.evidence_through.clone(),
                    };
                    checkpoint.validate().map_err(|source| {
                        harmos::Error::backend(
                            "validate inspected execution checkpoint",
                            harmos::BackendFailure::Corrupt,
                            source,
                        )
                    })?;
                    Ok(checkpoint)
                })?;
        Ok(checkpoint)
    }

    /// Binds a captured source to the evidence `view` it reads under. Every
    /// read, the exact pre-report revision and each output included, inherits
    /// `view`'s budget. `stall` is a test hook: each output read stalls that
    /// long within its budget.
    pub(crate) async fn captured(
        storage: &Manager,
        view: StorageView,
        captured: SharedQueryValue<CapturedSource>,
        owner: &NodeId,
        limit: NonZeroUsize,
        stall: std::time::Duration,
    ) -> Result<Self> {
        let validation = view.begin_computation()?;
        captured.check_origin(storage)?;
        if &captured.owner_node_id != owner {
            return Err(Error::Storage(dokime::storage::Error::Invalid(
                "captured execution source belongs to another node",
            )));
        }
        if captured.memory_size() == usize::MAX || captured.memory_size() > limit.get() {
            return Err(harmos::Error::ResourceExhausted {
                resource: "captured execution source memory",
            }
            .into());
        }
        let saved = view.saved()?;
        if !saved.covers(&captured.evidence_through)? {
            return Err(Error::InvalidReportInputs(
                "captured evidence exceeds saved view".into(),
            ));
        }
        captured.validate_header()?;
        if let Some(anchor) = &captured.execution
            && !saved.covers(&anchor.position)?
        {
            return Err(Error::InvalidReportInputs(
                "captured anchor exceeds saved view".into(),
            ));
        }
        let scratch = output_validation_memory(&captured.outputs)
            .checked_add(captured.context.validation_memory())
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| {
                Error::Storage(dokime::storage::Error::Invalid(
                    "captured execution validation memory overflow",
                ))
            })?;
        let _validation = storage.prepare_production(scratch)?;
        validate_outputs(
            &captured.execution_id,
            &captured.evidence_through,
            &captured.outputs,
        )
        .map_err(dokime::storage::Error::from)?;

        validation.check()?;
        drop(validation);
        let execution = if let Some(anchor) = &captured.execution {
            let revision = dokime::storage::objects::execution::read_in(
                storage,
                &view,
                &captured.execution_id,
                ObjectVersion::Exact(anchor.version.clone()),
                limit,
            )
            .await?
            .ok_or_else(|| {
                Error::InvalidReportInputs("captured primary anchor is missing".into())
            })?;
            if revision.position != anchor.position
                || revision.version != anchor.version
                || revision.value.test_execution_id != captured.execution_id
                || revision.value.saved.is_some()
            {
                return Err(Error::InvalidReportInputs(
                    "captured primary anchor differs from stored revision".into(),
                ));
            }
            Some(revision.try_into_shared()?)
        } else {
            None
        };
        let validation = view.begin_computation()?;
        captured.context.validate(&captured.lifecycle)?;
        // Bind Pre-Report Artifact Attribution To The Exact Unsealed Owner.
        for output in &captured.outputs {
            let ExecutionOutputContent::Artifact { record, .. } = &output.content else {
                continue;
            };
            if record
                .scope
                .test_cycle_id
                .as_ref()
                .is_some_and(|id| id != &captured.lifecycle.test_cycle_id)
                || record
                    .scope
                    .test_case_id
                    .as_ref()
                    .is_some_and(|id| id != &captured.lifecycle.test_case_id)
            {
                return Err(Error::Storage(dokime::storage::Error::Invalid(
                    "captured artifact scope differs from its pre-report execution owner",
                )));
            }
        }
        validation.check()?;
        drop(validation);
        let inspection = Self {
            view,
            execution,
            captured: Some(captured),
        };
        for output in inspection.outputs()? {
            if !stall.is_zero() {
                let read = inspection.view.begin_computation()?;
                tokio::time::sleep(stall).await;
                read.check()?;
            }
            verify_file(inspection.read_file(&output.file).await?).await?;
        }
        Ok(inspection)
    }

    pub async fn snapshot(&self, storage: &Manager) -> Result<Self> {
        if let Some(execution) = &self.execution {
            execution.check_origin(storage)?;
        }
        if let Some(source) = &self.captured {
            source.check_origin(storage)?;
            return Ok(self.clone());
        }
        let captured = self.read_source().await?.try_into_shared()?;
        Ok(Self {
            view: self.view.clone(),
            execution: self.execution.clone(),
            captured: Some(captured),
        })
    }

    pub fn context(&self) -> Result<&ExecutionContext> {
        self.captured
            .as_ref()
            .map(|source| &source.context)
            .ok_or_else(|| {
                Error::Storage(dokime::storage::Error::Invalid(
                    "execution context requires a snapshot",
                ))
            })
    }

    pub fn captured_at(&self) -> Result<&dokime::core::Timestamp> {
        self.captured
            .as_ref()
            .map(|source| &source.captured_at)
            .ok_or_else(|| {
                Error::Storage(dokime::storage::Error::Invalid(
                    "execution capture time requires a snapshot",
                ))
            })
    }

    pub fn project_context<U: MemorySize>(
        &self,
        limit: NonZeroUsize,
        map: impl FnOnce(&ExecutionContext, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        self.project_source(limit, |source, allowance| map(&source.context, allowance))
    }

    pub(crate) fn project_source<U: MemorySize>(
        &self,
        limit: NonZeroUsize,
        map: impl FnOnce(&CapturedSource, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        self.check_projection(limit)?;
        let source = self.captured.as_ref().ok_or_else(|| {
            Error::Storage(dokime::storage::Error::Invalid(
                "execution evidence requires a snapshot",
            ))
        })?;
        Ok(self.view.project_shared(source, limit, map)?)
    }

    fn check_projection(&self, projection: NonZeroUsize) -> Result<()> {
        if projection > self.view.decoded_bytes()? {
            return Err(harmos::Error::ResourceExhausted {
                resource: "execution snapshot original projection ceiling",
            }
            .into());
        }
        Ok(())
    }

    pub fn view(&self) -> &StorageView {
        &self.view
    }

    pub fn revision(&self) -> Result<&SharedQueryValue<ObjectRevision<TestExecution>>> {
        self.execution.as_ref().ok_or_else(|| {
            Error::InvalidReportInputs("execution has no persisted primary anchor".into())
        })
    }

    pub fn execution(&self) -> &TestExecution {
        if let Some(execution) = &self.execution
            && (execution.value.saved.is_some() || self.captured.is_none())
        {
            return &execution.value;
        }
        &self
            .captured
            .as_ref()
            .expect("captured view has lifecycle metadata")
            .lifecycle
    }

    pub(crate) async fn read_source(&self) -> Result<QueryValue<CapturedSource>> {
        crate::execution::context::read::read_execution(&self.view, self.revision()?).await
    }

    pub(crate) fn source(&self) -> Result<&ExecutionSource> {
        self.revision()?
            .value
            .saved
            .as_ref()
            .map(|seal| &seal.source)
            .ok_or_else(|| {
                Error::InvalidReportInputs("execution has no captured source dependencies".into())
            })
    }

    pub(crate) fn captured_source(&self) -> Option<&SharedQueryValue<CapturedSource>> {
        self.captured.as_ref()
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

    pub(crate) fn project_outputs<U: MemorySize>(
        &self,
        limit: NonZeroUsize,
        map: impl FnOnce(&[ExecutionOutput], NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let projected = match &self.captured {
            Some(source) => self
                .view
                .project_shared(source, limit, |source, allowance| {
                    map(&source.outputs, allowance)
                }),
            None => self
                .view
                .project_shared(self.revision()?, limit, |revision, allowance| {
                    let seal = revision.value.saved.as_ref().ok_or_else(|| {
                        harmos::Error::backend(
                            "project inspected execution outputs",
                            harmos::BackendFailure::Corrupt,
                            dokime::storage::Error::Invalid(
                                "execution has no captured source dependencies",
                            ),
                        )
                    })?;
                    map(&seal.source.outputs, allowance)
                }),
        };
        projected.map_err(Into::into)
    }

    pub fn context_records(
        &self,
        storage: &Manager,
        page: ReadLimits,
        projection: NonZeroUsize,
    ) -> Result<super::records::RecordReader> {
        self.check_projection(projection)?;
        super::records::RecordReader::open(storage, self, page, projection, true)
    }

    pub fn context_telemetry(
        &self,
        storage: &Manager,
        page: ReadLimits,
        projection: NonZeroUsize,
    ) -> Result<super::telemetry::TelemetryReader> {
        self.check_projection(projection)?;
        super::telemetry::TelemetryReader::open(storage, self, page, projection, true)
    }

    pub async fn read_file(&self, file: &FileReference) -> Result<StorageFileReader> {
        let owned = self
            .captured
            .as_ref()
            .is_some_and(|source| source.outputs.iter().any(|output| &output.file == file))
            || self
                .execution()
                .saved
                .as_ref()
                .is_some_and(|seal| seal.files().any(|owned| owned == file));
        if !owned {
            return Err(Error::Storage(dokime::storage::Error::Invalid(
                "file is not an exact captured dependency of this execution",
            )));
        }
        if !self.view.saved()?.covers(&file.position)? {
            return Err(Error::Storage(dokime::storage::Error::Invalid(
                "execution file exceeds the original saved view",
            )));
        }
        Ok(self.view.read_file(file).await?)
    }
}

/* === Public Functions === */
pub(super) async fn verify_file(mut reader: StorageFileReader) -> Result<()> {
    let chunk = NonZeroUsize::new(64 * 1024).unwrap();
    while reader.next_chunk(chunk).await?.is_some() {}
    Ok(())
}

pub(crate) fn check_seal_position(
    position: &StorageCursor,
    dependencies: &StorageCursor,
) -> Result<()> {
    if !position.covers(dependencies)? || position.sequence == dependencies.sequence {
        return Err(Error::Storage(dokime::storage::Error::Invalid(
            "seal revision does not follow saved dependencies",
        )));
    }
    Ok(())
}

/* === Private Functions === */
/* === Tests === */
