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
use futures_util::future::BoxFuture;
use harmos::MemorySize;

use harmos::storage::{
    FileMetadata, FileReference, QueryValue, SharedQueryValue, StorageCursor, StorageFile,
    StorageProductionPreparation,
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
const MAX_CAPTURE_BYTES: usize = 128 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedExecutionReportInput {
    pub final_report: bool,
    pub owner_node_id: NodeId,
    pub execution: ExecutionRevision,
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

struct ReportFile {
    data: SharedQueryValue<ReportFileData>,
    creating: tokio::sync::Mutex<Option<BoxFuture<'static, harmos::Result<StorageFile>>>>,
    producer: Option<StorageFile>,
    written: bool,
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
    pub(crate) async fn prepare_final_execution_report(
        &self,
        snapshot: &ExecutionView,
        outputs: QueryValue<Vec<ExecutionOutput>>,
    ) -> Result<PreparedExecutionReport> {
        outputs.check_origin(self.storage())?;
        if let Some(reason) = self.providers.final_report_refusal(snapshot.execution()) {
            return Err(Error::InvalidReportInputs(reason));
        }
        let maximum = NonZeroUsize::new(MAX_CAPTURE_BYTES).expect("nonzero report limit");
        let html = HtmlReport::from_snapshot(self, snapshot, true, maximum).await?;
        if html.captured_input()?.owner_node_id != self.state_read().identity.node.node_id {
            return Err(Error::InvalidReportInputs(
                "report producing node identity differs".into(),
            ));
        }
        let render = move || PreparedExecutionReport::prepare_html(html, outputs);
        let bytes = std::mem::size_of_val(&render)
            .checked_add(std::mem::size_of::<StorageProductionPreparation>())
            .ok_or_else(|| {
                Error::InvalidReportInputs("report rendering task memory overflows".into())
            })?;
        let memory = self.storage().prepare_production(allowance(bytes)?)?;
        tokio::task::spawn_blocking(move || {
            let _memory = memory;
            render()
        })
        .await?
    }

    /// Materializes and renders a sealed execution's report again, as cycle
    /// finalization does for each member, and returns its HTML. Benchmarks
    /// report materialization; nothing is stored.
    #[doc(hidden)]
    pub async fn render_saved_execution_report(
        &self,
        execution: &dokime::core::TestExecutionId,
    ) -> Result<Vec<u8>> {
        let reads = &self.config().storage_reads;
        let view = self
            .storage()
            .view_with_limits(reads.finalization())
            .await?;
        let limit = view.decoded_bytes()?;
        let inspection = ExecutionView::open_through(
            self.storage(),
            view,
            reads.execution,
            execution,
            harmos::storage::ObjectVersion::Latest,
            limit,
        )
        .await?
        .ok_or_else(|| Error::InvalidReportInputs("report execution is not saved".into()))?;
        let maximum = NonZeroUsize::new(MAX_CAPTURE_BYTES).expect("nonzero report limit");
        let html = HtmlReport::from_snapshot(self, &inspection, true, maximum).await?;
        tokio::task::spawn_blocking(move || {
            let maximum = NonZeroUsize::new(MAX_CAPTURE_BYTES).expect("nonzero report limit");
            let length = html.length(maximum)?;
            html.bytes(length)
        })
        .await?
    }
}

impl CapturedExecutionReportInput {
    pub fn validation_memory(&self) -> usize {
        self.context.validation_memory()
    }

    pub fn validate(&self) -> Result<()> {
        self.execution
            .version
            .check_position(&self.execution.position)?;
        if !self.evidence_through.covers(&self.execution.position)?
            || self.input.cycle.test_executions.len() != 1
        {
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

    fn execution(&self) -> Result<&TestExecution> {
        self.input
            .cycle
            .test_executions
            .first()
            .filter(|execution| execution.test_execution_id == self.execution.execution_id)
            .ok_or_else(|| Error::InvalidReportInputs("report execution identity differs".into()))
    }

    fn source_memory(&self, outputs: &[ExecutionOutput]) -> usize {
        std::mem::size_of::<CapturedSource>()
            .saturating_add(self.owner_node_id.memory_size())
            .saturating_add(self.execution.memory_size())
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

    fn source(&self, outputs: &[ExecutionOutput]) -> CapturedSource {
        CapturedSource {
            captured_at: self.input.generated_at.clone(),
            owner_node_id: self.owner_node_id.clone(),
            execution: self.execution.clone(),
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

    pub fn prepare(
        manager: Manager,
        captured: SharedQueryValue<CapturedExecutionReportInput>,
        outputs: QueryValue<Vec<ExecutionOutput>>,
    ) -> Result<Self> {
        let source = captured
            .try_project(
                allowance(captured.source_memory(&outputs))?,
                |captured, _| Ok(captured.source(&outputs)),
            )?
            .try_into_shared()?;
        let html = HtmlReport::from_capture(&manager, &captured, &source)?;
        Self::prepare_html(html, outputs)
    }

    fn prepare_html(html: HtmlReport, outputs: QueryValue<Vec<ExecutionOutput>>) -> Result<Self> {
        let manager = html.manager.clone();
        let captured = html.captured_input()?.clone();
        captured.check_origin(&manager)?;
        outputs.check_origin(&manager)?;
        let scratch =
            output_validation_memory(&outputs).saturating_add(captured.validation_memory());
        let _scratch = manager.prepare_production(allowance(scratch)?)?;
        captured.validate_final()?;
        validate_outputs(
            &captured.execution.execution_id,
            &captured.evidence_through,
            &outputs,
        )
        .map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        drop(_scratch);
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

    pub async fn save_dependencies(
        &mut self,
        sealed_at: &Timestamp,
    ) -> Result<QueryValue<ExecutionSeal>> {
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
            .saturating_add(self.captured.execution.memory_size())
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
        // Stage Both Original Files Before Publishing Their Ordered References.
        tokio::try_join!(
            self.context.write(&self.manager),
            self.report.write(&self.manager),
        )?;
        let context = self.context.finish(&self.manager).await?;
        let report = self.report.finish(&self.manager).await?;
        self.manager.flush_until(&report.position).await?;
        // Use the fixed report position, not a later unrelated saved frontier.
        let dependencies_saved = report.position.clone();
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
                execution: self.captured.execution.clone(),
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
            creating: tokio::sync::Mutex::new(None),
            producer: None,
            written: false,
            operation_memory: None,
        }
    }

    fn context(manager: &Manager, captured: &SharedQueryValue<CapturedSource>) -> Result<Self> {
        captured.check_origin(manager)?;
        let native_bytes = allowance(NativeSource::encoding_memory(captured)?)?;
        let native = captured.try_project(native_bytes, |value, _| {
            NativeSource::try_from(value).map_err(native_projection_error)
        })?;
        let maximum = NonZeroUsize::new(MAX_CAPTURE_BYTES).expect("nonzero report limit");
        let length = native.encoded_length(maximum)?;
        let execution_id = &captured.execution.execution_id;
        let metadata_bytes = execution_id
            .as_str()
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
                        report_file_id(execution_id.as_str(), ".context")?,
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
        let context = Self::context(manager, html.captured_source()?)?;
        let maximum = NonZeroUsize::new(MAX_CAPTURE_BYTES).expect("nonzero report limit");
        let length = html.length(maximum)?;
        let execution_id = &captured.execution.execution_id;
        let suffix = ".final-report";
        let metadata_bytes = execution_id
            .as_str()
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
                        report_file_id(execution_id.as_str(), suffix)?,
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

    async fn finish(&mut self, manager_reference: &Manager) -> Result<FileReference> {
        self.write(manager_reference).await?;
        self.producer
            .as_mut()
            .expect("written file producer")
            .finish()
            .await
            .map_err(Into::into)
    }

    async fn write(&mut self, manager_reference: &Manager) -> Result<()> {
        if !self.written {
            // Reconcile A Previous Partial Write
            if let Some(producer) = self.producer.as_mut() {
                producer.abort().await?;
                self.producer = None;
            }
            // Retain The Original Creation Across Cancelled Waits
            if self.creating.get_mut().is_none() {
                let manager = manager_reference.clone();
                let data = self.data.clone();
                let metadata_bytes = data.metadata.memory_size();
                let future = async move { manager.create_file(data.metadata.clone()).await };
                if self.operation_memory.is_none() {
                    let bytes = std::mem::size_of_val(&future)
                        .saturating_add(metadata_bytes)
                        .saturating_add(std::mem::size_of::<StorageFile>());
                    self.operation_memory = Some(
                        // Native Create/Finish Retains Its Own Destination Admission.
                        // This Charge Covers Our Boxed Wait And Metadata Copy.
                        manager_reference.prepare_production(allowance(bytes)?)?,
                    );
                }
                *self.creating.get_mut() = Some(Box::pin(future));
            }
            let outcome = self
                .creating
                .get_mut()
                .as_mut()
                .expect("retained file creation")
                .await;
            self.creating.get_mut().take();
            self.producer = Some(outcome?);

            let producer = self.producer.as_mut().expect("retained file producer");
            producer.write_all(&self.data.bytes).await?;
            self.written = true;
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
