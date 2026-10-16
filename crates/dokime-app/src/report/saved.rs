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
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
// Third Party Imports
use crate::execution::context::native::Encode;
use harmos::{MemorySize, operations::Host};

use harmos::storage::{
    FileMetadata, FileProducer, FileReference, MutationId, QueryValue, SharedQueryValue,
    StorageCursor, StorageProductionPreparation,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
// Local Imports
use super::{CycleReportInput, html::HtmlReport};
use crate::{
    Error, Result, RuntimeHandle,
    execution::context::{
        native::source::{MEDIA_TYPE, Source as NativeSource},
        source::CapturedSource,
    },
    runtime::storage::execution::ExecutionView,
};
use dokime::core::{Identifier, NodeId, Timestamp};
use dokime::execution::saved::{
    ExecutionOutput, ExecutionRevision, ExecutionSeal, ExecutionSource, output_validation_memory,
    validate_outputs,
};
use dokime::execution::{ExecutionStatus, TestExecution};
use dokime::records::RecordContract;
use dokime::storage::Manager;
use dokime::telemetry::contract::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedExecutionReportInput {
    pub final_report: bool,
    pub owner_node_id: NodeId,
    pub execution_id: dokime::core::TestExecutionId,
    pub execution: Option<ExecutionRevision>,
    pub lifecycle_id: String,
    pub evidence_through: StorageCursor,
    pub telemetry_contracts: BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>,
    pub record_contracts: BTreeMap<Identifier, BTreeMap<u32, RecordContract>>,
    pub input: CycleReportInput,
    pub context: crate::execution::context::ExecutionContext,
}

pub struct PreparedExecutionReport {
    manager: Manager,
    captured: SharedQueryValue<CapturedExecutionReportInput>,
    outputs: QueryValue<Vec<ExecutionOutput>>,
    context: ReportFile,
    report: ReportFile,
    sealed_at: Option<QueryValue<Timestamp>>,
}

pub(super) struct ReportFile {
    data: SharedQueryValue<ReportFileData>,
    window: Option<Identifier>,
    producer: Option<FileProducer>,
    written: usize,
    finished: Option<FileReference>,
    operation_memory: Option<StorageProductionPreparation>,
}

struct ReportFileData {
    metadata: FileMetadata,
    bytes: Vec<u8>,
}

/* === Enums === */
/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for CapturedExecutionReportInput {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.execution_id.memory_size(),
                std::mem::size_of_val(&self.execution_id),
            ),
            (
                self.lifecycle_id.memory_size(),
                std::mem::size_of_val(&self.lifecycle_id),
            ),
            (
                self.context.memory_size(),
                std::mem::size_of_val(&self.context),
            ),
            (
                harmos::MemorySize::memory_size(&self.owner_node_id),
                std::mem::size_of_val(&self.owner_node_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.execution),
                std::mem::size_of_val(&self.execution),
            ),
            (
                harmos::MemorySize::memory_size(&self.evidence_through),
                std::mem::size_of_val(&self.evidence_through),
            ),
            (
                harmos::MemorySize::memory_size(&self.telemetry_contracts),
                std::mem::size_of_val(&self.telemetry_contracts),
            ),
            (
                self.record_contracts.memory_size(),
                std::mem::size_of_val(&self.record_contracts),
            ),
            (
                harmos::MemorySize::memory_size(&self.input),
                std::mem::size_of_val(&self.input),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl MemorySize for ReportFileData {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.metadata
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<FileMetadata>()),
            )
            .saturating_add(self.bytes.capacity())
    }
}

impl RuntimeHandle {
    // Benchmarks And Callers Read The Immutable Published Bytes Of Sealed Subjects.
    pub async fn render_saved_execution_report(
        &self,
        execution: &dokime::core::TestExecutionId,
    ) -> Result<Vec<u8>> {
        let view = self
            .storage()
            .view_with_limits(self.config().storage.finalization())
            .await?;
        let limit = view.decoded_bytes()?;
        let inspection = ExecutionView::open(
            self.storage(),
            view,
            execution,
            harmos::storage::ObjectVersion::Latest,
            limit,
        )
        .await?
        .ok_or_else(|| Error::InvalidReportInputs("execution is not saved".into()))?;
        let seal = inspection.execution().saved.as_ref().ok_or_else(|| {
            Error::InvalidReportInputs("execution has no published report".into())
        })?;
        let mut reader = inspection.read_file(&seal.report).await?;
        let mut bytes = Vec::new();
        while let Some(batch) = reader
            .next_chunk(NonZeroUsize::new(64 * 1024).unwrap())
            .await?
        {
            bytes.extend_from_slice(&batch);
        }
        Ok(bytes)
    }
}

impl CapturedExecutionReportInput {
    pub fn validation_memory(&self) -> usize {
        self.context.validation_memory()
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(anchor) = &self.execution {
            anchor.version.check_position(&anchor.position)?;
            self.evidence_through
                .history
                .check(&anchor.position.history)?;
            if anchor.execution_id != self.execution_id {
                return Err(Error::InvalidReportInputs(
                    "report primary anchor identity differs".into(),
                ));
            }
        }
        uuid::Uuid::parse_str(&self.lifecycle_id)
            .map_err(|_| Error::InvalidReportInputs("invalid report lifecycle identity".into()))?;
        if self.input.cycle.test_executions.len() != 1 {
            return Err(Error::InvalidReportInputs(
                "report requires one exact saved execution".into(),
            ));
        }
        let execution = self.execution()?;
        if self.context.session_id != self.input.session_id
            || self.context.cycle.cycle.test_cycle_id != self.input.cycle.test_cycle_id
        {
            return Err(Error::InvalidReportInputs(
                "captured report and inspection context identity differs".into(),
            ));
        }
        self.context.validate(execution)?;
        if execution.saved.is_some() || execution.test_cycle_id != self.input.cycle.test_cycle_id {
            return Err(Error::InvalidReportInputs(
                "report input is sealed or belongs to a different cycle".into(),
            ));
        }
        if self.final_report
            && (matches!(
                execution.status,
                ExecutionStatus::Pending | ExecutionStatus::Running
            ) || execution.finished_at.is_none()
                || execution.active_step_attempt.is_some())
        {
            return Err(Error::InvalidReportInputs(
                "final report input has not settled".into(),
            ));
        }
        for (kind, versions) in &self.record_contracts {
            if versions.is_empty() {
                return Err(Error::InvalidReportInputs(
                    "captured record type has no declaration".into(),
                ));
            }
            for (version, contract) in versions {
                if kind != &contract.record_type
                    || *version == 0
                    || *version != contract.schema_version
                {
                    return Err(Error::InvalidReportInputs(
                        "captured record declaration identity differs".into(),
                    ));
                }
            }
        }
        for record in &self.input.records {
            self.record_contracts
                .get(&record.record_type)
                .and_then(|versions| versions.get(&record.schema_version))
                .ok_or_else(|| {
                    Error::InvalidReportInputs(format!(
                        "captured record declaration is missing for {} version {}",
                        record.record_type, record.schema_version
                    ))
                })?;
        }
        for (stream, versions) in &self.telemetry_contracts {
            if versions.is_empty() {
                return Err(Error::InvalidReportInputs(
                    "captured telemetry stream has no producer declaration".into(),
                ));
            }
            for (version, contract) in versions {
                if stream != &contract.stream
                    || *version == 0
                    || *version != contract.schema_version
                    || contract.foreign
                {
                    return Err(Error::InvalidReportInputs(
                        "captured telemetry declaration is not its producer contract".into(),
                    ));
                }
            }
        }
        for sample in &self.input.telemetry {
            self.telemetry_contracts
                .get(&sample.stream)
                .and_then(|versions| versions.get(&sample.schema_version))
                .ok_or_else(|| {
                    Error::InvalidReportInputs("captured telemetry declaration is missing".into())
                })?;
        }
        Ok(())
    }

    pub(crate) fn validate_final(&self) -> Result<()> {
        if !self.final_report {
            return Err(Error::InvalidReportInputs(
                "execution seal requires a final report capture".into(),
            ));
        }
        self.validate()
    }

    pub(crate) fn execution(&self) -> Result<&TestExecution> {
        self.input
            .cycle
            .test_executions
            .first()
            .filter(|execution| execution.test_execution_id == self.execution_id)
            .ok_or_else(|| Error::InvalidReportInputs("report execution identity differs".into()))
    }

    pub(crate) fn source_memory(&self, outputs: &[ExecutionOutput]) -> usize {
        std::mem::size_of::<CapturedSource>()
            .saturating_add(self.owner_node_id.memory_size())
            .saturating_add(self.execution.memory_size())
            .saturating_add(self.execution_id.memory_size())
            .saturating_add(self.lifecycle_id.memory_size())
            .saturating_add(
                self.input
                    .cycle
                    .test_executions
                    .first()
                    .map_or(0, MemorySize::memory_size),
            )
            .saturating_add(self.evidence_through.memory_size())
            .saturating_add(self.context.memory_size())
            .saturating_add(self.record_contracts.memory_size())
            .saturating_add(self.telemetry_contracts.memory_size())
            .saturating_add(self.input.generated_at.memory_size())
            .saturating_add(outputs.iter().fold(
                std::mem::size_of::<Vec<ExecutionOutput>>(),
                |bytes, output| bytes.saturating_add(output.memory_size()),
            ))
    }

    pub(crate) fn source(&self, outputs: &[ExecutionOutput]) -> CapturedSource {
        CapturedSource {
            captured_at: self.input.generated_at.clone(),
            owner_node_id: self.owner_node_id.clone(),
            execution_id: self.execution_id.clone(),
            execution: self.execution.clone(),
            lifecycle_id: self.lifecycle_id.clone(),
            lifecycle: self.input.cycle.test_executions[0].clone(),
            evidence_through: self.evidence_through.clone(),
            context: self.context.clone(),
            record_contracts: self.record_contracts.clone(),
            telemetry_contracts: self.telemetry_contracts.clone(),
            outputs: outputs.to_vec(),
        }
    }
}

impl PreparedExecutionReport {
    pub(crate) fn captured_input(&self) -> &SharedQueryValue<CapturedExecutionReportInput> {
        &self.captured
    }

    pub(crate) fn settled_outputs(&self) -> &QueryValue<Vec<ExecutionOutput>> {
        &self.outputs
    }

    pub(super) fn prepare_html(
        html: HtmlReport,
        outputs: QueryValue<Vec<ExecutionOutput>>,
    ) -> Result<Self> {
        let manager = html.manager.clone();
        let captured = html.captured_input()?.clone();
        captured.check_origin(&manager)?;
        outputs.check_origin(&manager)?;
        let scratch =
            output_validation_memory(&outputs).saturating_add(captured.validation_memory());
        let _scratch = manager.prepare_production(allowance(scratch)?)?;
        captured.validate()?;
        validate_outputs(&captured.execution_id, &captured.evidence_through, &outputs)
            .map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        let (context, report) = ReportFile::capture(&manager, &html)?;
        if outputs.iter().any(|output| {
            output.file.metadata.id == context.data.metadata.id
                || output.file.metadata.id == report.data.metadata.id
        }) {
            return Err(Error::InvalidReportInputs(
                "report production reuses an output file identity".into(),
            ));
        }
        Ok(Self {
            manager,
            captured,
            outputs,
            context,
            report,
            sealed_at: None,
        })
    }

    pub(crate) async fn publish_current(
        &mut self,
        operations: &impl Host<Error = harmos::Error>,
        revision: &ExecutionRevision,
    ) -> Result<dokime::operations::reports::MaterializedReport> {
        self.captured.validate()?;
        tokio::try_join!(
            self.context.write(&self.manager, operations),
            self.report.write(&self.manager, operations)
        )?;
        let context = self.context.finish(&self.manager, operations).await?;
        let report = self.report.finish(&self.manager, operations).await?;
        let last = [&context.position, &report.position]
            .into_iter()
            .max_by_key(|cursor| cursor.sequence)
            .unwrap();
        let dependencies_saved = self.manager.flush_until(last).await?;
        let mut publication = dokime::storage::reports::Prepared::new(
            &self.manager,
            &dokime::storage::reports::Input {
                owner: dokime::storage::reports::Owner::Execution(&self.captured.execution_id),
                generated_at: &self.captured.input.generated_at,
                final_report: false,
                evidence_through: &self.captured.evidence_through,
                dependencies_saved: &dependencies_saved,
                executions: std::slice::from_ref(revision),
                inputs: std::slice::from_ref(&context),
                file: &report,
                cycle: None,
                execution: None,
            },
        )?;
        publication.finish().await?;
        Ok(dokime::operations::reports::MaterializedReport { report })
    }

    pub async fn save_dependencies(
        &mut self,
        operations: &impl Host<Error = harmos::Error>,
        sealed_at: &Timestamp,
        revision: &ExecutionRevision,
    ) -> Result<QueryValue<ExecutionSeal>> {
        self.captured.validate_final()?;
        if self
            .sealed_at
            .as_ref()
            .is_some_and(|original| &**original != sealed_at)
        {
            return Err(Error::InvalidReportInputs(
                "report retry changes seal time".into(),
            ));
        }
        if self.sealed_at.is_none() {
            self.sealed_at = Some(
                self.manager
                    .produce(allowance(sealed_at.memory_size())?, |_| {
                        Ok(sealed_at.clone())
                    })?
                    .try_into_value()?,
            );
        }
        // Reserve Seal Copies And Validation Scratch Before File Receipts Arrive.
        let bytes = std::mem::size_of::<ExecutionSeal>()
            .saturating_add(self.captured.owner_node_id.memory_size())
            .saturating_add(revision.memory_size())
            .saturating_add(
                self.captured
                    .evidence_through
                    .memory_size()
                    .saturating_mul(4),
            )
            .saturating_add(self.outputs.memory_size())
            .saturating_add(self.context.data.metadata.memory_size())
            .saturating_add(self.report.data.metadata.memory_size())
            .saturating_add(2 * std::mem::size_of::<FileReference>())
            .saturating_add(sealed_at.memory_size())
            .saturating_add(output_validation_memory(&self.outputs));
        let production = self.manager.prepare_production(allowance(bytes)?)?;
        self.manager
            .flush_until(&self.captured.evidence_through)
            .await?;
        // Stage Frozen Context And HTML Before Publication.
        tokio::try_join!(
            self.context.write(&self.manager, operations),
            self.report.write(&self.manager, operations),
        )?;
        let context = self.context.finish(&self.manager, operations).await?;
        let report = self.report.finish(&self.manager, operations).await?;
        let last = [&context.position, &report.position]
            .into_iter()
            .max_by_key(|cursor| cursor.sequence)
            .unwrap();
        let dependencies_saved = self.manager.flush_until(last).await?;
        for file in [&context, &report] {
            if !dependencies_saved.covers(&file.position)? {
                return Err(Error::InvalidReportInputs(
                    "report dependency is not durable".into(),
                ));
            }
        }
        let mut outputs = Vec::new();
        outputs
            .try_reserve_exact(self.outputs.len())
            .map_err(|error| Error::Operation(error.to_string()))?;
        if outputs.capacity() > self.outputs.len() {
            return Err(Error::InvalidReportInputs(
                "seal output capacity exceeds its preclaim".into(),
            ));
        }
        outputs.extend(self.outputs.iter().cloned());
        let seal = ExecutionSeal {
            source: ExecutionSource {
                owner_node_id: self.captured.owner_node_id.clone(),
                evidence_through: self.captured.evidence_through.clone(),
                execution: revision.clone(),
                context,
                outputs,
            },
            report,
            dependencies_saved,
            sealed_at: (**self.sealed_at.as_ref().expect("retained seal time")).clone(),
        };
        seal.validate(self.captured.execution()?)
            .map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        Ok(production.finish(seal)?.try_into_value()?)
    }
}

impl ReportFile {
    fn new(data: SharedQueryValue<ReportFileData>) -> Self {
        Self {
            data,
            window: None,
            producer: None,
            written: 0,
            finished: None,
            operation_memory: None,
        }
    }

    pub(super) fn render(
        manager: &Manager,
        html: &HtmlReport,
        metadata: QueryValue<FileMetadata>,
    ) -> Result<Self> {
        metadata.check_origin(manager)?;
        let length = html.length(NonZeroUsize::new(usize::MAX).unwrap())?;
        let bytes = length
            .saturating_add(metadata.memory_size())
            .saturating_add(std::mem::size_of::<ReportFileData>());
        let data = manager
            .produce(allowance(bytes)?, |_| {
                let bytes = html.bytes(length).map_err(native_projection_error)?;
                let mut metadata = (*metadata).clone();
                metadata.expected_length = Some(bytes.len() as u64);
                metadata.expected_sha256 = Some(Sha256::digest(&bytes).into());
                Ok(ReportFileData { metadata, bytes })
            })?
            .try_into_value()?
            .try_into_shared()?;
        Ok(Self::new(data))
    }

    fn context(
        manager: &Manager,
        captured: &SharedQueryValue<CapturedSource>,
        stem: &str,
    ) -> Result<Self> {
        captured.check_origin(manager)?;
        let native_bytes = allowance(NativeSource::encoding_memory(captured)?)?;
        let native = captured.try_project(native_bytes, |value, _| {
            NativeSource::try_from(value).map_err(native_projection_error)
        })?;
        let maximum = NonZeroUsize::new(usize::MAX).expect("nonzero address-space limit");
        let length = native.encoded_length(maximum)?;
        let execution_id = &captured.execution_id;
        let metadata_bytes = stem
            .len()
            .saturating_mul(2)
            .saturating_add(".context".len())
            .saturating_add(MEDIA_TYPE.len());
        let bytes = std::mem::size_of::<ReportFileData>()
            .saturating_add(length)
            .saturating_add(metadata_bytes);
        let data = native
            .try_project(allowance(bytes)?, |native, _| {
                (|| -> Result<_> {
                    let bytes = native.encode_counted(length)?;
                    let metadata = file_metadata(
                        report_file_id(stem, ".context")?,
                        Identifier::new(execution_id.as_str())
                            .map_err(|error| Error::InvalidReportInputs(error.to_string()))?,
                        MEDIA_TYPE,
                        &bytes,
                    )?;
                    Ok(ReportFileData { metadata, bytes })
                })()
                .map_err(native_projection_error)
            })?
            .try_into_shared()?;
        Ok(Self::new(data))
    }

    fn capture(manager: &Manager, html: &HtmlReport) -> Result<(Self, Self)> {
        let captured = html.captured_input()?;
        let _identity_memory = manager.prepare_production(allowance(256)?)?;
        let stem = format!("{}.{}", captured.execution_id, uuid::Uuid::new_v4());
        let context = Self::context(manager, html.captured_source()?, &stem)?;
        let maximum = NonZeroUsize::new(usize::MAX).expect("nonzero address-space limit");
        let length = html.length(maximum)?;
        let execution_id = &captured.execution_id;
        let suffix = ".report";
        let metadata_bytes = stem
            .len()
            .saturating_mul(2)
            .saturating_add(suffix.len())
            .saturating_add("text/html".len());
        let bytes = std::mem::size_of::<ReportFileData>()
            .saturating_add(length)
            .saturating_add(metadata_bytes);
        let data = captured
            .try_project(allowance(bytes)?, |_, _| {
                (|| -> Result<_> {
                    let bytes = html.bytes(length)?;
                    let metadata = file_metadata(
                        report_file_id(&stem, suffix)?,
                        Identifier::new(execution_id.as_str())
                            .map_err(|error| Error::InvalidReportInputs(error.to_string()))?,
                        "text/html",
                        &bytes,
                    )?;
                    Ok(ReportFileData { metadata, bytes })
                })()
                .map_err(native_projection_error)
            })?
            .try_into_shared()?;
        Ok((context, Self::new(data)))
    }

    pub(super) async fn finish(
        &mut self,
        manager: &Manager,
        operations: &impl Host<Error = harmos::Error>,
    ) -> Result<FileReference> {
        if let Some(reference) = &self.finished {
            return Ok(reference.clone());
        }
        self.write(manager, operations).await?;
        let reference = operations
            .call_as::<harmos::storage::operations::FinishFile>(
                harmos::storage::operations::FileMutation {
                    mutation: self.mutation("finish")?,
                    session: self
                        .producer
                        .as_ref()
                        .expect("created report file")
                        .session
                        .clone(),
                },
            )
            .await?
            .into_owned();
        self.finished = Some(reference.clone());
        Ok(reference)
    }

    fn mutation(&self, action: &str) -> Result<MutationId> {
        Ok(MutationId {
            window: self
                .window
                .as_ref()
                .expect("retained mutation window")
                .clone(),
            key: Identifier::new(format!("{}.{}", self.data.metadata.id, action))
                .map_err(|error| Error::InvalidReportInputs(error.to_string()))?,
        })
    }

    async fn write(
        &mut self,
        manager: &Manager,
        operations: &impl Host<Error = harmos::Error>,
    ) -> Result<()> {
        use harmos::storage::operations::{
            AppendFile, AppendFileRequest, CreateFile, CreateFileRequest,
        };

        if self.finished.is_some() {
            return Ok(());
        }
        let batch_bytes = usize::try_from(manager.info()?.file_chunk_bytes)
            .map_err(|_| Error::InvalidReportInputs("file batch size overflows".into()))?;
        if self.operation_memory.is_none() {
            let bytes = batch_bytes
                .saturating_add(self.data.metadata.memory_size())
                .saturating_add(4096);
            self.operation_memory = Some(manager.prepare_production(allowance(bytes)?)?);
        }
        if self.window.is_none() {
            self.window = Some(manager.mutation_window()?.id);
        }
        if self.producer.is_none() {
            self.producer = Some(
                operations
                    .call_as::<CreateFile>(CreateFileRequest {
                        mutation: self.mutation("create")?,
                        metadata: self.data.metadata.clone(),
                    })
                    .await?
                    .into_owned(),
            );
        }
        while self.written < self.data.bytes.len() {
            let end = self
                .written
                .saturating_add(batch_bytes)
                .min(self.data.bytes.len());
            let producer = operations
                .call_as::<AppendFile>(AppendFileRequest {
                    mutation: self.mutation(&format!("append.{}", self.written))?,
                    session: self
                        .producer
                        .as_ref()
                        .expect("created report file")
                        .session
                        .clone(),
                    offset: self.written as u64,
                    bytes: self.data.bytes[self.written..end].to_vec(),
                })
                .await?
                .into_owned();
            if producer.length != end as u64 {
                return Err(Error::InvalidReportInputs(
                    "report append returned a different offset".into(),
                ));
            }
            self.producer = Some(producer);
            self.written = end;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn native_projection_error(source: Error) -> harmos::Error {
    harmos::Error::Backend {
        operation: "encode native captured report",
        kind: harmos::BackendFailure::Corrupt,
        source: Box::new(source),
    }
}

fn allowance(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|_| bytes != usize::MAX)
        .ok_or_else(|| Error::InvalidReportInputs("report production memory overflows".into()))
}

fn report_file_id(execution: &str, suffix: &str) -> Result<Identifier> {
    let length = execution
        .len()
        .checked_add(suffix.len())
        .ok_or_else(|| Error::InvalidReportInputs("report file identity overflows".into()))?;
    let mut id = String::new();
    id.try_reserve_exact(length)
        .map_err(|error| Error::Operation(error.to_string()))?;
    if id.capacity() > length {
        return Err(Error::InvalidReportInputs(
            "report identity capacity exceeds its preclaim".into(),
        ));
    }
    id.push_str(execution);
    id.push_str(suffix);
    Identifier::new(id).map_err(|error| Error::InvalidReportInputs(error.to_string()))
}

fn file_metadata(
    id: Identifier,
    scope: Identifier,
    media_type: &str,
    bytes: &[u8],
) -> Result<FileMetadata> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| Error::Operation("report file length overflow".into()))?;
    let sha256 = Sha256::digest(bytes).into();
    Ok(FileMetadata {
        id,
        scope: Some(scope),
        media_type: media_type.into(),
        expected_length: Some(length),
        expected_sha256: Some(sha256),
    })
}

/* === Tests === */
