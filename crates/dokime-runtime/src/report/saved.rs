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
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use harmos::{
    Message,
    operations::Host,
    storage::{FileMetadata, FileProducer, FileReference, MutationId, StorageCursor},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Local Imports
use super::{CycleReportInput, html::HtmlReport};
use crate::records::RecordContract;
use crate::storage::Manager;
use crate::telemetry::contract::TelemetryContract;
use crate::{
    RuntimeHandle,
    execution::context::captured::{
        native::source::{MEDIA_TYPE, Source as NativeSource},
        source::CapturedSource,
    },
    runtime::Error,
    runtime::Result,
    runtime::storage::execution::ExecutionView,
};
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionRevision;
use dokime_domain::model::execution::saved::ExecutionSeal;
use dokime_domain::model::execution::saved::ExecutionSource;
use dokime_domain::model::execution::saved::validate_outputs;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::NodeId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedExecutionReportInput {
    pub final_report: bool,
    pub owner_node_id: NodeId,
    pub execution_id: dokime_domain::model::identity::TestExecutionId,
    pub execution: Option<ExecutionRevision>,
    pub lifecycle_id: String,
    pub evidence_through: StorageCursor,
    pub telemetry_contracts: BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>,
    pub record_contracts: BTreeMap<Identifier, BTreeMap<u32, RecordContract>>,
    pub input: CycleReportInput,
    pub context: crate::execution::context::ExecutionContext,
}

pub struct PreparedExecutionReport {
    captured: Arc<CapturedExecutionReportInput>,
    outputs: Vec<ExecutionOutput>,
    context: ReportFile,
    report: ReportFile,
    sealed_at: Option<Timestamp>,
}

pub(super) struct ReportFile {
    data: Arc<ReportFileData>,
    window: Option<Identifier>,
    producer: Option<FileProducer>,
    written: usize,
    finished: Option<FileReference>,
}

struct ReportFileData {
    metadata: FileMetadata,
    bytes: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    // Benchmarks And Callers Read The Immutable Published Bytes Of Sealed Subjects.
    pub async fn render_saved_execution_report(&self, execution: &dokime_domain::model::identity::TestExecutionId) -> Result<Vec<u8>> {
        let inspection = ExecutionView::open(self.storage(), execution, harmos::storage::ObjectVersion::Latest)
            .await?
            .ok_or_else(|| Error::InvalidReportInputs("execution is not saved".into()))?;
        let seal =
            inspection.execution().saved.as_ref().ok_or_else(|| Error::InvalidReportInputs("execution has no published report".into()))?;
        Ok(crate::storage::read_file_bytes(self.storage(), &seal.report).await?)
    }
}

impl CapturedExecutionReportInput {
    pub fn validate(&self) -> Result<()> {
        if let Some(anchor) = &self.execution {
            anchor.version.check_position(&anchor.position)?;
            self.evidence_through.history.check(&anchor.position.history)?;
            if anchor.execution_id != self.execution_id {
                return Err(Error::InvalidReportInputs("report primary anchor identity differs".into()));
            }
        }
        uuid::Uuid::parse_str(&self.lifecycle_id).map_err(|_| Error::InvalidReportInputs("invalid report lifecycle identity".into()))?;
        if self.input.cycle.test_executions.len() != 1 {
            return Err(Error::InvalidReportInputs("report requires one exact saved execution".into()));
        }
        let execution = self.execution()?;
        if self.context.session_id != self.input.session_id || self.context.cycle.cycle.test_cycle_id != self.input.cycle.test_cycle_id {
            return Err(Error::InvalidReportInputs("captured report and inspection context identity differs".into()));
        }
        self.context.validate(execution)?;
        if execution.saved.is_some() || execution.test_cycle_id != self.input.cycle.test_cycle_id {
            return Err(Error::InvalidReportInputs("report input is sealed or belongs to a different cycle".into()));
        }
        if self.final_report
            && (matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running)
                || execution.finished_at.is_none()
                || execution.active_step_attempt.is_some())
        {
            return Err(Error::InvalidReportInputs("final report input has not settled".into()));
        }
        for (kind, versions) in &self.record_contracts {
            if versions.is_empty() {
                return Err(Error::InvalidReportInputs("captured record type has no declaration".into()));
            }
            for (version, contract) in versions {
                if kind != &contract.record_type || *version == 0 || *version != contract.schema_version {
                    return Err(Error::InvalidReportInputs("captured record declaration identity differs".into()));
                }
            }
        }
        for record in &self.input.records {
            self.record_contracts.get(&record.record_type).and_then(|versions| versions.get(&record.schema_version)).ok_or_else(|| {
                Error::InvalidReportInputs(format!(
                    "captured record declaration is missing for {} version {}",
                    record.record_type, record.schema_version
                ))
            })?;
        }
        for (stream, versions) in &self.telemetry_contracts {
            if versions.is_empty() {
                return Err(Error::InvalidReportInputs("captured telemetry stream has no producer declaration".into()));
            }
            for (version, contract) in versions {
                if stream != &contract.stream || *version == 0 || *version != contract.schema_version || contract.foreign {
                    return Err(Error::InvalidReportInputs("captured telemetry declaration is not its producer contract".into()));
                }
            }
        }
        for sample in &self.input.telemetry {
            self.telemetry_contracts
                .get(&sample.stream)
                .and_then(|versions| versions.get(&sample.schema_version))
                .ok_or_else(|| Error::InvalidReportInputs("captured telemetry declaration is missing".into()))?;
        }
        Ok(())
    }

    pub(crate) fn validate_final(&self) -> Result<()> {
        if !self.final_report {
            return Err(Error::InvalidReportInputs("execution seal requires a final report capture".into()));
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
    pub(crate) fn captured_input(&self) -> &Arc<CapturedExecutionReportInput> {
        &self.captured
    }

    pub(crate) fn settled_outputs(&self) -> &Vec<ExecutionOutput> {
        &self.outputs
    }

    pub(super) fn prepare_html(html: HtmlReport, outputs: Vec<ExecutionOutput>, saved: &StorageCursor) -> Result<Self> {
        let captured = html.captured_input()?.clone();
        captured.validate()?;
        validate_outputs(&captured.execution_id, saved, &outputs).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        let (context, report) = ReportFile::capture(&html)?;
        if outputs
            .iter()
            .any(|output| output.file.metadata.id == context.data.metadata.id || output.file.metadata.id == report.data.metadata.id)
        {
            return Err(Error::InvalidReportInputs("report production reuses an output file identity".into()));
        }
        Ok(Self { captured, outputs, context, report, sealed_at: None })
    }

    pub(crate) async fn publish_current(
        &mut self, manager: &Manager, operations: &impl Host<Error = harmos::Error>, revision: &ExecutionRevision,
    ) -> Result<dokime_domain::model::reports::wire::MaterializedReport> {
        self.captured.validate()?;
        tokio::try_join!(self.context.write(manager, operations), self.report.write(manager, operations))?;
        let context = self.context.finish(manager, operations).await?;
        let report = self.report.finish(manager, operations).await?;
        let last = [&context.position, &report.position].into_iter().max_by_key(|cursor| cursor.sequence).unwrap();
        let dependencies_saved = manager.flush_until(last).await?;
        let mut publication = crate::storage::reports::Prepared::new(
            manager,
            &crate::storage::reports::Input {
                owner: crate::storage::reports::Owner::Execution(&self.captured.execution_id),
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
        Ok(dokime_domain::model::reports::wire::MaterializedReport { report })
    }

    pub async fn save_dependencies(
        &mut self, manager: &Manager, operations: &impl Host<Error = harmos::Error>, sealed_at: &Timestamp, revision: &ExecutionRevision,
    ) -> Result<ExecutionSeal> {
        self.captured.validate_final()?;
        if self.sealed_at.as_ref().is_some_and(|original| original != sealed_at) {
            return Err(Error::InvalidReportInputs("report retry changes seal time".into()));
        }
        if self.sealed_at.is_none() {
            self.sealed_at = Some(sealed_at.clone());
        }
        manager.flush_until(&self.captured.evidence_through).await?;
        // Stage Frozen Context And HTML Before Publication.
        tokio::try_join!(self.context.write(manager, operations), self.report.write(manager, operations),)?;
        let context = self.context.finish(manager, operations).await?;
        let report = self.report.finish(manager, operations).await?;
        let last = [&context.position, &report.position].into_iter().max_by_key(|cursor| cursor.sequence).unwrap();
        let dependencies_saved = manager.flush_until(last).await?;
        for file in [&context, &report] {
            if !dependencies_saved.covers(&file.position)? {
                return Err(Error::InvalidReportInputs("report dependency is not durable".into()));
            }
        }
        let outputs = self.outputs.clone();
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
            sealed_at: self.sealed_at.clone().expect("retained seal time"),
        };
        seal.validate(self.captured.execution()?).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        Ok(seal)
    }
}

impl ReportFile {
    fn new(data: ReportFileData) -> Self {
        Self { data: Arc::new(data), window: None, producer: None, written: 0, finished: None }
    }

    pub(super) fn render(html: &HtmlReport, mut metadata: FileMetadata) -> Result<Self> {
        let bytes = html.render()?;
        metadata.expected_length = Some(bytes.len() as u64);
        metadata.expected_sha256 = Some(Sha256::digest(&bytes).into());
        Ok(Self::new(ReportFileData { metadata, bytes }))
    }

    fn context(captured: &CapturedSource, stem: &str) -> Result<Self> {
        let bytes = NativeSource::try_from(captured)?.encode_to_vec();
        let metadata = file_metadata(
            report_file_id(stem, ".context")?,
            Identifier::new(captured.execution_id.as_str()).map_err(|error| Error::InvalidReportInputs(error.to_string()))?,
            MEDIA_TYPE,
            &bytes,
        )?;
        Ok(Self::new(ReportFileData { metadata, bytes }))
    }

    fn capture(html: &HtmlReport) -> Result<(Self, Self)> {
        let captured = html.captured_input()?;
        let stem = format!("{}.{}", captured.execution_id, uuid::Uuid::now_v7());
        let context = Self::context(html.captured_source()?, &stem)?;
        let bytes = html.render()?;
        let metadata = file_metadata(
            report_file_id(&stem, ".report")?,
            Identifier::new(captured.execution_id.as_str()).map_err(|error| Error::InvalidReportInputs(error.to_string()))?,
            "text/html",
            &bytes,
        )?;
        Ok((context, Self::new(ReportFileData { metadata, bytes })))
    }

    pub(super) async fn finish(&mut self, manager: &Manager, operations: &impl Host<Error = harmos::Error>) -> Result<FileReference> {
        if let Some(reference) = &self.finished {
            return Ok(reference.clone());
        }
        self.write(manager, operations).await?;
        let reference = operations
            .call(harmos::storage::operations::FinishFile {
                mutation: self.mutation(manager, "finish")?,
                session: self.producer.as_ref().expect("created report file").session.clone(),
            })
            .await?
            .into_owned();
        self.finished = Some(reference.clone());
        self.window = None;
        Ok(reference)
    }

    fn mutation(&mut self, manager: &Manager, action: &str) -> Result<MutationId> {
        if self.window.is_none() {
            self.window = Some(manager.mutation_window()?.id);
        }
        Ok(MutationId {
            window: self.window.as_ref().expect("retained mutation window").clone(),
            key: Identifier::new(format!("{}.{}", self.data.metadata.id, action))
                .map_err(|error| Error::InvalidReportInputs(error.to_string()))?,
        })
    }

    async fn write(&mut self, manager: &Manager, operations: &impl Host<Error = harmos::Error>) -> Result<()> {
        use harmos::storage::operations::{AppendFile, CreateFile};

        if self.finished.is_some() {
            return Ok(());
        }
        let batch_bytes = usize::try_from(manager.info()?.file_batch_bytes)
            .map_err(|_| Error::InvalidReportInputs("file batch size overflows".into()))?;
        if self.producer.is_none() {
            self.producer = Some(
                operations
                    .call(CreateFile { mutation: self.mutation(manager, "create")?, metadata: self.data.metadata.clone() })
                    .await?
                    .into_owned(),
            );
            self.window = None;
        }
        while self.written < self.data.bytes.len() {
            let end = self.written.saturating_add(batch_bytes).min(self.data.bytes.len());
            let producer = operations
                .call(AppendFile {
                    mutation: self.mutation(manager, &format!("append.{}", self.written))?,
                    session: self.producer.as_ref().expect("created report file").session.clone(),
                    offset: self.written as u64,
                    bytes: self.data.bytes[self.written..end].to_vec(),
                })
                .await?
                .into_owned();
            if producer.length != end as u64 {
                return Err(Error::InvalidReportInputs("report append returned a different offset".into()));
            }
            self.producer = Some(producer);
            self.written = end;
            self.window = None;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn report_file_id(execution: &str, suffix: &str) -> Result<Identifier> {
    Identifier::new(format!("{execution}{suffix}")).map_err(|error| Error::InvalidReportInputs(error.to_string()))
}

fn file_metadata(id: Identifier, scope: Identifier, media_type: &str, bytes: &[u8]) -> Result<FileMetadata> {
    let length = u64::try_from(bytes.len()).map_err(|_| Error::Operation("report file length overflow".into()))?;
    let sha256 = Sha256::digest(bytes).into();
    Ok(FileMetadata { id, scope: Some(scope), media_type: media_type.into(), expected_length: Some(length), expected_sha256: Some(sha256) })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    // The Published Payload Of A Fixed Input; Changing It Changes Every Sealed Report Digest.
    const PAYLOAD_SHA256: &str = "883c5bfc20def0e49b9e8ae8a3319d745fa65bf82fba9a7f387818888d14ef20";

    #[test]
    fn report_digest_is_stable() {
        let captured: CapturedExecutionReportInput = serde_json::from_str(include_str!("fixtures/execution-report-input.json")).unwrap();
        let evidence = Arc::new(captured.source(&[]));
        let html = HtmlReport::from_capture(Arc::new(captured), evidence).unwrap();
        let (_, report) = ReportFile::capture(&html).unwrap();
        let bytes = &report.data.bytes;
        assert_eq!(report.data.metadata.expected_length, Some(bytes.len() as u64));
        assert_eq!(report.data.metadata.expected_sha256, Some(Sha256::digest(bytes).into()));
        let html = std::str::from_utf8(bytes).unwrap();
        let payload = html.split_once(r#"<script id="hub-report-data" type="application/json">"#).unwrap().1;
        let payload = payload.split_once("</script>").unwrap().0;
        let digest = Sha256::digest(payload.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(digest, PAYLOAD_SHA256);
    }
}
