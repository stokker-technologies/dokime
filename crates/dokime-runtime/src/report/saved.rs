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
use std::{
    collections::BTreeMap,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    sync::{Arc, LazyLock},
};

// Third Party Imports
use harmos::{
    Message,
    storage::{FileChunk, FileMetadata, FileReference, StorageCursor, StorageFile},
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

static REPORT_PREFIX_DIGEST: LazyLock<Sha256> = LazyLock::new(|| {
    let mut digest = Sha256::new();
    digest.update(super::output::REPORT_PARTS.0.as_bytes());
    digest
});

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedExecutionReportInput {
    pub report_defaults: crate::config::ReportDefaults,
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
    captured: harmos::storage::ReadBatch<CapturedExecutionReportInput>,
    outputs: Arc<harmos::storage::ReadBatch<Vec<ExecutionOutput>>>,
    context: ReportFile,
    report: ReportFile,
    sealed_at: Option<Timestamp>,
}

pub(super) struct ReportFile {
    data: harmos::storage::ReadBatch<ReportFileData>,
    producer: Option<StorageFile>,
    written: u64,
    finished: Option<FileReference>,
}

struct ReportFileData {
    metadata: FileMetadata,
    spool: std::sync::Mutex<std::fs::File>,
}

struct ReportWriter {
    output: BufWriter<std::fs::File>,
    digest: Sha256,
    length: u64,
}

#[derive(Clone, Copy)]
struct ReportFileIdentity<'a> {
    owner: &'a str,
    nonce: uuid::Uuid,
    suffix: &'a str,
    media_type: &'a str,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ReportFileData {
    fn context(source: &NativeSource, identity: ReportFileIdentity<'_>, length: usize) -> Result<Self> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(report_allocation_error)?;
        source.encode_into(&mut bytes);
        if bytes.len() != length {
            return Err(report_invalid("report encoding differs from reported length"));
        }
        let mut writer = ReportWriter::new()?;
        writer.write_all(&bytes)?;
        writer.finish(identity)
    }
}

impl ReportFileIdentity<'_> {
    fn id_length(&self) -> Result<usize> {
        let mut length = self.owner.len();
        for part in [1, uuid::fmt::Hyphenated::LENGTH, self.suffix.len()] {
            let Some(total) = length.checked_add(part) else {
                return Err(report_invalid("report identity length overflows"));
            };
            length = total;
        }
        Ok(length)
    }

    fn id(&self) -> Result<Identifier> {
        let mut id = report_text(self.id_length()?)?;
        id.push_str(self.owner);
        id.push('.');
        let mut nonce = [0u8; uuid::fmt::Hyphenated::LENGTH];
        let nonce = self.nonce.hyphenated().encode_lower(&mut nonce);
        id.push_str(nonce);
        id.push_str(self.suffix);
        Ok(Identifier::new(id).map_err(crate::storage::Error::from)?)
    }

    fn metadata(&self, length: u64, digest: [u8; 32]) -> Result<FileMetadata> {
        let id = self.id()?;
        let mut scope = report_text(self.owner.len())?;
        scope.push_str(self.owner);
        let scope = Identifier::new(scope).map_err(crate::storage::Error::from)?;

        let mut media_type = report_text(self.media_type.len())?;
        media_type.push_str(self.media_type);
        Ok(FileMetadata {
            id,
            scope: Some(scope),
            media_type,
            expected_length: Some(length),
            expected_sha256: Some(digest),
        })
    }
}

impl RuntimeHandle {
    // Benchmarks And Callers Read The Immutable Published Bytes Of Sealed Subjects.
    pub async fn render_saved_execution_report(
        &self, execution: &dokime_domain::model::identity::TestExecutionId,
    ) -> Result<harmos::storage::ReadBatch<Vec<u8>>> {
        let session = self.storage().clone();
        let inspection = ExecutionView::open(self.storage(), execution, harmos::storage::ObjectVersion::Latest)
            .await?
            .ok_or_else(|| Error::InvalidReportInputs("execution is not saved".into()))?;
        let seal =
            inspection.execution().saved.as_ref().ok_or_else(|| Error::InvalidReportInputs("execution has no published report".into()))?;
        Ok(crate::storage::read_file_bytes(&session, &seal.report).await?)
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
            report_defaults: self.report_defaults,
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
    pub(crate) fn captured_input(&self) -> &harmos::storage::ReadBatch<CapturedExecutionReportInput> {
        &self.captured
    }

    pub(crate) fn settled_outputs(&self) -> &[ExecutionOutput] {
        self.outputs.as_slice()
    }

    pub(super) fn prepare_html(
        html: HtmlReport, outputs: Arc<harmos::storage::ReadBatch<Vec<ExecutionOutput>>>, saved: &StorageCursor,
    ) -> Result<Self> {
        let captured = html.captured_input()?.clone();
        captured.validate()?;
        validate_outputs(&captured.execution_id, saved, outputs.as_slice())
            .map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        let (context, report) = ReportFile::capture(&html)?;
        if outputs
            .iter()
            .any(|output| output.file.metadata.id == context.data.metadata.id || output.file.metadata.id == report.data.metadata.id)
        {
            return Err(Error::InvalidReportInputs("report production reuses an output file identity".into()));
        }
        Ok(Self {
            captured,
            outputs,
            context,
            report,
            sealed_at: None,
        })
    }

    pub(crate) async fn publish_current(
        &mut self, manager: &Manager, revision: &ExecutionRevision, schema_settings: &harmos::config::SchemaSettings,
    ) -> Result<dokime_domain::model::reports::wire::MaterializedReport> {
        self.captured.validate()?;
        tokio::try_join!(self.context.write(manager), self.report.write(manager))?;
        let context = self.context.finish(manager).await?;
        let report = self.report.finish(manager).await?;
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
            schema_settings,
        )?;
        publication.finish().await?;
        Ok(dokime_domain::model::reports::wire::MaterializedReport { report })
    }

    pub async fn save_dependencies(
        &mut self, manager: &Manager, sealed_at: &Timestamp, revision: &ExecutionRevision,
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
        tokio::try_join!(self.context.write(manager), self.report.write(manager),)?;
        let context = self.context.finish(manager).await?;
        let report = self.report.finish(manager).await?;
        let last = [&context.position, &report.position].into_iter().max_by_key(|cursor| cursor.sequence).unwrap();
        let dependencies_saved = manager.flush_until(last).await?;
        for file in [&context, &report] {
            if !dependencies_saved.covers(&file.position)? {
                return Err(Error::InvalidReportInputs("report dependency is not durable".into()));
            }
        }
        let outputs = self.outputs.as_slice().to_vec();
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
    fn new(data: harmos::storage::ReadBatch<ReportFileData>) -> Self {
        Self {
            data,
            producer: None,
            written: 0,
            finished: None,
        }
    }

    pub(super) fn render(html: &HtmlReport, owner: &str, nonce: uuid::Uuid, suffix: &str) -> Result<Self> {
        let identity = ReportFileIdentity {
            owner,
            nonce,
            suffix,
            media_type: "text/html",
        };
        let mut writer = ReportWriter::new()?;
        html.write(&mut writer)?;
        let data = writer.finish(identity)?;
        Ok(Self::new(harmos::storage::ReadBatch::new(data)))
    }

    fn context(captured: &CapturedSource, nonce: uuid::Uuid) -> Result<Self> {
        let identity = ReportFileIdentity {
            owner: captured.execution_id.as_str(),
            nonce,
            suffix: ".context",
            media_type: MEDIA_TYPE,
        };
        let seed = harmos::storage::ReadBatch::new(());
        let build = |_| NativeSource::try_from(captured);
        let native = seed.try_map(build)?;

        let length = native.encoded_size();
        let seed = harmos::storage::ReadBatch::new(());
        let encode = |_| ReportFileData::context(&native, identity, length);
        let data = seed.try_map(encode)?;
        drop(native);
        Ok(Self::new(data))
    }

    fn capture(html: &HtmlReport) -> Result<(Self, Self)> {
        let captured = html.captured_input()?;
        let nonce = uuid::Uuid::now_v7();
        let evidence = html.captured_source()?;
        let context = Self::context(evidence, nonce)?;
        let report = Self::render(html, captured.execution_id.as_str(), nonce, ".report")?;
        Ok((context, report))
    }

    pub(super) async fn finish(&mut self, manager: &Manager) -> Result<FileReference> {
        if let Some(reference) = &self.finished {
            return Ok(reference.clone());
        }
        self.write(manager).await?;
        let reference = self.producer.as_mut().expect("created report file").finish().await?;
        self.finished = Some(reference.clone());
        Ok(reference)
    }

    async fn write(&mut self, manager: &Manager) -> Result<()> {
        if self.finished.is_some() {
            return Ok(());
        }
        let batch_bytes = usize::try_from(manager.info()?.file_batch_bytes)
            .map_err(|_| Error::InvalidReportInputs("file batch size overflows".into()))?;
        // Concurrent File Producers Share The Configured Pending Write Budget.
        let request = &manager.settings().request;
        let per_file = (request.max_pending_write_bytes.get() / request.max_active_file_operations.get()).max(1);
        let batch_bytes = batch_bytes.min(per_file);
        if self.producer.is_none() {
            self.producer = Some(manager.create_file(self.data.metadata.clone()).await?);
        }

        let total = self.data.metadata.expected_length.expect("spooled report length");
        while self.written < total {
            let length = (total - self.written).min(batch_bytes as u64) as usize;
            let data = self.data.clone();
            let offset = self.written;
            let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
                let mut spool = data.spool.lock().map_err(|_| report_invalid("report spool lock is poisoned"))?;
                spool.seek(SeekFrom::Start(offset))?;
                let mut bytes = Vec::new();
                bytes.try_reserve_exact(length).map_err(report_allocation_error)?;
                bytes.resize(length, 0);
                spool.read_exact(&mut bytes)?;
                Ok(bytes)
            })
            .await??;
            let end = offset + length as u64;
            let producer = self.producer.as_mut().expect("created report file");
            let bytes = FileChunk::new(bytes);
            append_report_chunk(manager, producer, offset, bytes).await?;
            if producer.length() != end {
                return Err(Error::InvalidReportInputs("report append returned a different offset".into()));
            }
            self.written = end;
        }
        Ok(())
    }
}

impl ReportWriter {
    fn new() -> Result<Self> {
        let spool = tempfile::tempfile()?;
        Ok(Self {
            output: BufWriter::with_capacity(1024 * 1024, spool),
            digest: Sha256::new(),
            length: 0,
        })
    }

    fn finish(mut self, identity: ReportFileIdentity<'_>) -> Result<ReportFileData> {
        self.output.flush()?;
        let mut spool = self.output.into_inner().map_err(|error| error.into_error())?;
        spool.seek(SeekFrom::Start(0))?;
        let metadata = identity.metadata(self.length, self.digest.finalize().into())?;
        Ok(ReportFileData {
            metadata,
            spool: std::sync::Mutex::new(spool),
        })
    }
}

impl Write for ReportWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let length = self.length.checked_add(bytes.len() as u64).ok_or_else(|| std::io::Error::other("report length overflows"))?;
        self.output.write_all(bytes)?;
        // Every Standalone Report Begins With The Same Embedded Application.
        // Reuse Its Hash State Only For That Exact Prefix At Offset Zero.
        if self.length == 0 && bytes == super::output::REPORT_PARTS.0.as_bytes() {
            self.digest = (*REPORT_PREFIX_DIGEST).clone();
        } else {
            self.digest.update(bytes);
        }
        self.length = length;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn append_report_chunk(manager: &Manager, producer: &mut StorageFile, offset: u64, bytes: FileChunk) -> Result<()> {
    loop {
        let result = producer.write_retained_at(offset, bytes.clone()).await;
        if !matches!(result, Err(harmos::storage::Error::ResourceExhausted { resource: "file append" })) {
            return result.map_err(Into::into);
        }
        // Worker Concurrency Does Not Bound Queued Producers. Drain Shared
        // Writes And Let Peer Files Progress Before Retrying The Same Chunk.
        manager.flush().await?;
        tokio::task::yield_now().await;
    }
}

fn report_text(length: usize) -> Result<String> {
    let mut value = String::new();
    value.try_reserve_exact(length).map_err(report_allocation_error)?;
    Ok(value)
}

fn report_invalid(message: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(message))
}

fn report_allocation_error(_: std::collections::TryReserveError) -> Error {
    report_invalid("report allocation failed")
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    // The Published Payload Of A Fixed Input; Changing It Changes Every Sealed Report Digest.
    const PAYLOAD_SHA256: &str = "8c4a6d2aa30cbc895613b8c9c1eeef0e5de409661abc65d95800cbb4197ccc12";

    #[tokio::test]
    async fn report_digest_is_stable() {
        let captured: CapturedExecutionReportInput = serde_json::from_str(include_str!("fixtures/execution-report-input.json")).unwrap();
        let evidence = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(captured.source(&[]))).unwrap();
        let captured = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(captured)).unwrap();
        let html = HtmlReport::from_capture(captured, evidence).unwrap();
        let source = html.captured_source().unwrap();
        let (context, report) = ReportFile::capture(&html).unwrap();
        let mut context_bytes = Vec::new();
        context.data.spool.lock().unwrap().read_to_end(&mut context_bytes).unwrap();
        let decoded = NativeSource::decode(&context_bytes).unwrap();
        let decoded = decoded.into_source().unwrap();
        assert!(decoded == **source);
        drop(decoded);
        let context_owner = context.data.clone();
        drop(context);
        drop(context_owner);

        let mut bytes = Vec::new();
        report.data.spool.lock().unwrap().read_to_end(&mut bytes).unwrap();
        assert_eq!(report.data.metadata.expected_length, Some(bytes.len() as u64));
        assert_eq!(report.data.metadata.expected_sha256, Some(Sha256::digest(&bytes[..]).into()));
        let html = std::str::from_utf8(&bytes).unwrap();
        let payload = html.split_once(r#"<script id="hub-report-data" type="application/json">"#).unwrap().1;
        let payload = payload.split_once("</script>").unwrap().0;
        let digest = Sha256::digest(payload.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(digest, PAYLOAD_SHA256);
        drop(report);
    }

    #[test]
    fn report_spool_hashes_incremental_writes_and_rewinds() {
        let identity = ReportFileIdentity {
            owner: "spooled.report",
            nonce: uuid::Uuid::nil(),
            suffix: ".report",
            media_type: "text/html",
        };
        let mut writer = ReportWriter::new().unwrap();
        let block = vec![0xa5; 1024 * 1024];
        let mut expected = Sha256::new();
        for _ in 0..17 {
            writer.write_all(&block).unwrap();
            expected.update(&block);
        }
        let data = writer.finish(identity).unwrap();
        assert_eq!(data.metadata.expected_length, Some(17 * 1024 * 1024));
        assert_eq!(data.metadata.expected_sha256, Some(expected.finalize().into()));
        let mut spool = data.spool.lock().unwrap();
        assert_eq!(spool.stream_position().unwrap(), 0);
        assert_eq!(spool.metadata().unwrap().len(), 17 * 1024 * 1024);
        let mut prefix = [0; 128];
        spool.read_exact(&mut prefix).unwrap();
        assert_eq!(prefix, [0xa5; 128]);
    }

    #[test]
    fn report_file_metadata_identifies_content() {
        let nonce = uuid::Uuid::nil();
        let identity = ReportFileIdentity {
            owner: "fixture.execution",
            nonce,
            suffix: ".context",
            media_type: MEDIA_TYPE,
        };
        for capacity in [0, 128] {
            let bytes = Vec::with_capacity(capacity);
            let metadata = identity.metadata(bytes.len() as u64, Sha256::digest(&bytes).into()).unwrap();
            let data = ReportFileData {
                metadata,
                spool: std::sync::Mutex::new(tempfile::tempfile().unwrap()),
            };
            assert_eq!(data.metadata.expected_length, Some(0));
            assert_eq!(data.metadata.scope.as_ref().unwrap().as_str(), identity.owner);
            assert_eq!(data.metadata.id.as_str(), "fixture.execution.00000000-0000-0000-0000-000000000000.context");
        }
    }
}
