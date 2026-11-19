/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Retained Native Content Metadata

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::num::NonZeroUsize;
use std::path::Path;
use std::result;

// Third Party Imports
use harmos_protocol::model::FileReference;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MapKey, ReflectMessage, Value};
use prost_types::Timestamp;
use sha2::{Digest, Sha256};
use tempfile::TempPath;
use zip::{CompressionMethod, DateTime, System, ZipWriter, write::FullFileOptions};

// Local Imports
use crate::persisted::Scope;
use crate::{Error, Evidence, Execution, Report, Result, StatisticsRead, Store};
use dokime_protocol::host::CyclePhase;
use dokime_protocol::host::CycleSummary;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::content::ExecutionFileMetadata;
use dokime_protocol::host::evidence::OperatorNote;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::model::{ArtifactMetadata, MapValue, RecordContracts, origin};
use harmos_protocol::model::Declarations;

/* === Constants === */

pub const CONTENT_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct ArchivedContent {
    pub template_label: Option<String>,

    pub files: Vec<ArchivedNativeFile>,

    /// Original StoredArtifact protobuf bytes retained by the content owner.
    pub artifacts: Vec<Vec<u8>>,

    pub execution_files: Vec<StoredExecutionFile>,

    pub notes: Vec<OperatorNote>,

    pub report: Option<ArchivedNativeFile>,
}

struct ArchivedExecution {
    execution: Execution,
    evidence: Vec<Evidence>,
    seal: Vec<u8>,
    reports: Vec<Report>,
    artifacts: Vec<StoredArtifact>,
    files: Vec<StoredExecutionFile>,
    notes: Vec<OperatorNote>,
}

pub struct ArchivedNativeFile {
    pub reference: FileReference,
    pub path: TempPath,
}

#[derive(Clone, PartialEq, Message)]
pub struct StoredArtifact {
    #[prost(message, optional, tag = "1")]
    pub request: Option<ArtifactsCreateRequest>,

    #[prost(message, optional, tag = "2")]
    pub artifact: Option<ArtifactMetadata>,
}

#[derive(Clone, PartialEq, Message)]
pub struct StoredExecutionFile {
    #[prost(message, optional, tag = "1")]
    pub metadata: Option<ExecutionFileMetadata>,

    #[prost(message, optional, tag = "2")]
    pub file: Option<FileReference>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn has_incompatible_content(&self) -> Result<bool> {
        Ok(self.lock()?.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name IN ('content_objects','content_chunks'))",
            [],
            |row| row.get(0),
        )?)
    }
    pub fn write_cycle_archive(
        &self, cycle: &str, page: NonZeroUsize, output: &mut impl Write, contents: &mut HashMap<String, ArchivedContent>,
        mut content: ArchivedContent,
    ) -> Result<()> {
        let mut summary = self.client_cycle(cycle)?.unwrap_or_else(|| CycleSummary {
            cycle_id: cycle.into(),
            ..Default::default()
        });
        summary.sealed = true;
        summary.phase = CyclePhase::Sealed as i32;
        summary.report_file = None;
        summary.evidence_file = None;
        let timestamp = summary.updated_at.as_ref().ok_or(Error::Data("Export Cycle: Retry Its Seal to Establish Its Seal Timestamp"))?;
        let options = archive_options(timestamp)?;
        let mut archive = ZipWriter::new_stream(output);
        let mut entries = Vec::new();
        let count: i64 = self.lock()?.query_row("SELECT COUNT(*) FROM executions WHERE cycle=?1", [cycle], |row| row.get(0))?;
        let width = count.to_string().len().max(2);
        let mut after = 0i64;
        let mut index = 0usize;
        loop {
            let ids = {
                let connection = self.lock()?;
                let mut query =
                    connection.prepare("SELECT rowid,id FROM executions WHERE cycle=?1 AND rowid>?2 ORDER BY rowid LIMIT ?3")?;
                query
                    .query_map(rusqlite::params![cycle, after, page.get() as i64], |row| {
                        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                    })?
                    .collect::<result::Result<Vec<_>, _>>()?
            };
            if ids.is_empty() {
                break;
            }
            for (position, id) in ids {
                after = position;
                index += 1;
                let content = contents.remove(&id).unwrap_or_default();
                let label = content.template_label.as_deref().unwrap_or(&id);
                let prefix = format!("executions/{index:0width$} {}/", sanitize_label(label));
                let members = self.write_execution_members(&id, &mut archive, &prefix, content)?;
                entries.extend(members);
            }
        }
        content.notes.sort_by(|left, right| left.note_id.cmp(&right.note_id));
        content.artifacts.sort();
        let metadata = serde_json::json!({
            "cycle": project_message("dokime.host.v1.CycleSummary", &summary.encode_to_vec())?,
            "notes": content.notes.iter().map(|note| project_message("dokime.host.v1.evidence.OperatorNote", &note.encode_to_vec())).collect::<Result<Vec<_>>>()?,
            "artifacts": content.artifacts.iter().map(|bytes| artifact_projection(&StoredArtifact::decode(bytes.as_slice())?)).collect::<Result<Vec<_>>>()?,
        });
        write_entry(&mut archive, "data/cycle.json", options.clone(), &serde_json::to_vec_pretty(&metadata)?, &mut entries)?;
        write_native_files(
            &mut archive,
            "",
            Scope::Cycle(cycle).identifier()?.as_str(),
            content.files,
            &content.artifacts,
            options.clone(),
            &mut entries,
        )?;
        if let Some(report) = content.report {
            entries.push(write_file(
                &mut archive,
                "report.html",
                options.clone(),
                &report.path,
                report.reference.length,
                &report.reference.sha256,
            )?);
        }
        write_manifest(&mut archive, "manifest.json", options.clone(), &entries)?;
        archive.finish()?;
        Ok(())
    }

    pub fn write_execution_archive(&self, execution: &str, output: &mut impl Write, content: ArchivedContent) -> Result<()> {
        let mut archive = ZipWriter::new_stream(output);
        self.write_execution_members(execution, &mut archive, "", content)?;
        archive.finish()?;
        Ok(())
    }

    fn write_execution_members<W: Write + Seek>(
        &self, execution: &str, archive: &mut ZipWriter<W>, prefix: &str, mut content: ArchivedContent,
    ) -> Result<Vec<serde_json::Value>> {
        content.files.sort_by_cached_key(|file| file.reference.encode_to_vec());
        content.artifacts.sort();
        content.execution_files.sort_by_cached_key(Message::encode_to_vec);
        content.notes.sort_by(|left, right| left.note_id.cmp(&right.note_id));
        let metadata = self.execution_archive(execution, &content)?;
        let timestamp = metadata
            .execution
            .sealed_at
            .as_ref()
            .ok_or(Error::Data("Export Execution: Restore Its Seal Timestamp or Select a Fresh Storage Root"))?;
        let options = archive_options(timestamp)?;
        let mut entries = Vec::new();
        for (kind, value) in evidence_projection(&metadata)? {
            let bytes = serde_json::to_vec_pretty(&value)?;
            write_entry(archive, &format!("{prefix}data/{kind}.json"), options.clone(), &bytes, &mut entries)?;
        }
        let connection = self.lock()?;
        let mut query =
            connection.prepare("SELECT id,stream,length,sha256 FROM telemetry_files WHERE execution=?1 AND finalized=1 ORDER BY stream")?;
        for row in query.query_map([execution], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)? as u64, row.get::<_, Vec<u8>>(3)?))
        })? {
            let (id, stream, length, sha256) = row?;
            let member = format!("{prefix}telemetry/{}.arrow", stream.replace('.', "-"));
            crate::validate_execution_path(&member)?;
            let path = crate::telemetry::catalog::path(self.database.parent().unwrap(), &id)?;
            entries.push(write_file(archive, &member, options.clone(), &path, length, &sha256)?);
        }
        drop(query);
        drop(connection);
        write_native_files(
            archive,
            prefix,
            Scope::Execution(execution).identifier()?.as_str(),
            content.files,
            &content.artifacts,
            options.clone(),
            &mut entries,
        )?;
        if let Some(report) = content.report {
            entries.push(write_file(
                archive,
                &format!("{prefix}report.html"),
                options.clone(),
                &report.path,
                report.reference.length,
                &report.reference.sha256,
            )?);
        }
        let member = format!("{prefix}manifest.json");
        let relative = entries
            .iter()
            .filter(|entry| entry["path"] != prefix)
            .cloned()
            .map(|mut entry| {
                entry["path"] = entry["path"].as_str().unwrap().strip_prefix(prefix).unwrap().into();
                entry
            })
            .collect::<Vec<_>>();
        let bytes = manifest_bytes(&relative)?;
        write_entry(archive, &member, options.clone(), &bytes, &mut entries)?;
        Ok(entries)
    }

    fn execution_archive(&self, execution: &str, content: &ArchivedContent) -> Result<ArchivedExecution> {
        let connection = self.lock()?;
        let (state, payload, seal): (i32, Vec<u8>, Vec<u8>) =
            connection.query_row("SELECT state,payload,seal FROM executions WHERE id=?1", [execution], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
        if state != 2 || crate::store::digest(&connection, execution)? != seal {
            return Err(Error::Invalid("Download Evidence: Select a Valid Sealed Execution".into()));
        }
        let mut query = connection.prepare("SELECT payload FROM evidence WHERE execution=?1 ORDER BY position")?;
        let evidence = query
            .query_map([execution], |row| row.get::<_, Vec<u8>>(0))?
            .map(|row| Ok(Evidence::decode(row?.as_slice())?))
            .collect::<Result<Vec<_>>>()?;
        let mut query = connection.prepare("SELECT payload FROM reports WHERE execution=?1 ORDER BY id")?;
        let reports = query
            .query_map([execution], |row| row.get::<_, Vec<u8>>(0))?
            .map(|row| Ok(Report::decode(row?.as_slice())?))
            .collect::<Result<Vec<_>>>()?;
        Ok(ArchivedExecution {
            execution: Execution::decode(payload.as_slice())?,
            evidence,
            seal,
            reports,
            artifacts: content.artifacts.iter().map(|bytes| Ok(StoredArtifact::decode(bytes.as_slice())?)).collect::<Result<Vec<_>>>()?,
            files: content.execution_files.clone(),
            notes: content.notes.clone(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn write_native_files<W: Write + Seek>(
    archive: &mut ZipWriter<W>, prefix: &str, owner: &str, mut files: Vec<ArchivedNativeFile>, artifacts: &[Vec<u8>],
    options: FullFileOptions<'static, 'static>, entries: &mut Vec<serde_json::Value>,
) -> Result<()> {
    files.sort_by_cached_key(|file| file.reference.encode_to_vec());
    let artifacts = artifacts.iter().map(|bytes| StoredArtifact::decode(bytes.as_slice())).collect::<result::Result<Vec<_>, _>>()?;
    for file in files {
        let reference = &file.reference;
        let metadata = reference.metadata.as_ref().ok_or(Error::Data("Export Native File: Restore Its Metadata"))?;
        if metadata.scope.as_deref() != Some(owner) {
            return Err(Error::Data("Export Native File: Select the Owner's Own File Reference"));
        }
        let identity = hex(&Sha256::digest(reference.encode_to_vec()));
        let directory = if artifacts.iter().any(|entry| entry.artifact.as_ref().is_some_and(|artifact| artifact.upload_id == metadata.id)) {
            "artifacts"
        } else {
            "files"
        };
        let member = format!("{prefix}{directory}/{identity}");
        entries.push(write_file(archive, &member, options.clone(), &file.path, file.reference.length, &file.reference.sha256)?);
        write_entry(archive, &format!("{member}.pb"), options.clone(), &reference.encode_to_vec(), entries)?;
    }
    Ok(())
}

fn write_file<W: Write + Seek>(
    archive: &mut ZipWriter<W>, member: &str, options: FullFileOptions<'static, 'static>, path: &Path, length: u64, sha256: &[u8],
) -> Result<serde_json::Value> {
    let mut input = File::open(path)?;
    archive.start_file(member, options.clone())?;
    let mut hash = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; CONTENT_CHUNK_BYTES];
    loop {
        let length = input.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        size += length as u64;
        hash.update(&buffer[..length]);
        archive.write_all(&buffer[..length])?;
    }
    let digest = hash.finalize();
    if size != length || digest.as_slice() != sha256 {
        return Err(Error::Data("Export Native File: Restore Bytes Matching Its Retained Reference"));
    }
    Ok(serde_json::json!({"path":member,"size":size,"sha256":hex(&digest)}))
}

fn write_entry<W: Write + Seek>(
    archive: &mut ZipWriter<W>, member: &str, options: FullFileOptions<'static, 'static>, bytes: &[u8],
    entries: &mut Vec<serde_json::Value>,
) -> Result<()> {
    archive.start_file(member, options.clone())?;
    archive.write_all(bytes)?;
    entries.push(serde_json::json!({"path":member,"size":bytes.len(),"sha256":hex(&Sha256::digest(bytes))}));
    Ok(())
}

fn manifest_bytes(entries: &[serde_json::Value]) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec_pretty(&serde_json::json!({"version":1,"entries":entries}))?)
}

fn write_manifest<W: Write + Seek>(
    archive: &mut ZipWriter<W>, member: &str, options: FullFileOptions<'static, 'static>, entries: &[serde_json::Value],
) -> Result<()> {
    archive.start_file(member, options.clone())?;
    archive.write_all(&manifest_bytes(entries)?)?;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sanitize_label(label: &str) -> String {
    label.chars().map(|character| if u32::from(character) < 32 || "<>:\"/\\|?*".contains(character) { '-' } else { character }).collect()
}

fn archive_options(timestamp: &Timestamp) -> Result<FullFileOptions<'static, 'static>> {
    let date = if (315_532_800..4_354_819_200).contains(&timestamp.seconds) {
        let text = timestamp.to_string();
        let year = text[0..4].parse().map_err(|_| Error::Data("Export Archive: Restore Its Seal Date"))?;
        let number = |start, end| text[start..end].parse::<u8>().map_err(|_| Error::Data("Export Archive: Restore Its Seal Date"));
        DateTime::from_date_and_time(year, number(5, 7)?, number(8, 10)?, number(11, 13)?, number(14, 16)?, number(17, 19)?)
            .map_err(|_| Error::Data("Export Archive: Restore Its Seal Date"))?
    } else {
        DateTime::default()
    };
    let mut options = FullFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(date)
        .system(System::Unix)
        .unix_permissions(0o644);
    // NTFS Time Retains Dates Outside DOS's Range and Subsecond Seal Precision.
    let ticks = (i128::from(timestamp.seconds) + 11_644_473_600) * 10_000_000 + i128::from(timestamp.nanos) / 100;
    let ticks = u64::try_from(ticks).map_err(|_| Error::Data("Export Archive: Restore a Seal Date After 1601"))?;
    let mut extra = vec![0; 4];
    extra.extend_from_slice(&1u16.to_le_bytes());
    extra.extend_from_slice(&24u16.to_le_bytes());
    for _ in 0..3 {
        extra.extend_from_slice(&ticks.to_le_bytes());
    }
    options.add_extra_field(0x000a, extra, false)?;
    Ok(options)
}

fn evidence_projection(archive: &ArchivedExecution) -> Result<BTreeMap<String, serde_json::Value>> {
    let execution = &archive.execution;
    let mut kinds = BTreeMap::<String, Vec<serde_json::Value>>::new();
    for kind in [
        "steps",
        "records",
        "criteria",
        "prompts",
        "notes",
        "parameters",
        "outcome",
        "artifacts",
        "files",
        "execution",
        "execution-failures",
        "stream-failures",
        "cleanup-failures",
        "reports",
        "logs",
        "progress",
        "traceability",
        "capture-gaps",
        "statistics",
        "documents",
    ] {
        kinds.insert(kind.into(), Vec::new());
    }
    let mut catalog = project_message("harmos.model.v1.Declarations", &execution.catalog)?;
    let declarations = Declarations::decode(execution.catalog.as_slice())?;
    let types = declarations
        .types
        .map(DescriptorPool::from_file_descriptor_set)
        .transpose()
        .map_err(|source| Error::Invalid(format!("Project Declarations: Restore Their Frozen Types: {source}")))?;
    for (index, section) in declarations.sections.iter().enumerate() {
        let descriptor = dokime_protocol::descriptor_pool()
            .get_message_by_name(&section.r#type)
            .or_else(|| types.as_ref().and_then(|pool| pool.get_message_by_name(&section.r#type)));
        if let Some(descriptor) = descriptor {
            let value = DynamicMessage::decode(descriptor, section.payload.as_slice())?;
            catalog["sections"][index]["payload"] = project_dynamic(&value)?;
        }
    }
    let origin = execution.origin.as_ref().map(|origin| project_message("dokime.model.v1.Origin", &origin.encode_to_vec())).transpose()?;
    kinds.get_mut("execution").unwrap().push(serde_json::json!({"identifier":execution.identifier,"cycle":execution.cycle,"origin":origin,"catalog":catalog,"seal":hex(&archive.seal),"termination":execution.termination,"sealed_at":execution.sealed_at.as_ref().map(ToString::to_string)}));
    for report in &archive.reports {
        kinds.get_mut("reports").unwrap().push(serde_json::json!({"identifier":report.identifier,"document":report.document.as_ref().map(|document| project_message("dokime.model.v1.Document", &document.encode_to_vec())).transpose()?,"execution_seal":hex(&report.execution_seal),"html":String::from_utf8_lossy(&report.html),"seal":hex(&report.seal)}));
    }
    for evidence in &archive.evidence {
        let (kind, message) = match evidence.kind.as_str() {
            kind if kind.starts_with("step:") || kind.starts_with("attempt:") => ("steps", None),
            kind if kind.starts_with("record:") => ("records", Some("dokime.host.v1.evidence.RecordsAppendRequest")),
            kind if kind.starts_with("criterion:") => ("criteria", Some("dokime.plugin.v1.CriteriaEvaluateResponse")),
            "prompt" => ("prompts", Some("dokime.host.v1.PromptsCreateRequest")),
            "answer" => ("prompts", Some("dokime.host.v1.PromptsCreateResponse")),
            "client:answer" => ("prompts", Some("dokime.host.v1.PromptsCreateResponse")),
            "log" => ("logs", Some("dokime.host.v1.evidence.LogsAppendRequest")),
            "progress" => ("progress", Some("dokime.model.v1.ExecutionProgressValue")),
            "traceability" => ("traceability", Some("dokime.model.v1.TraceabilityEntry")),
            "capture:gap" => ("capture-gaps", Some("dokime.model.v1.CaptureGap")),
            "read:statistics" => ("statistics", None),
            "document:part" => ("documents", Some("dokime.host.v1.DocumentsAppendPartsRequest")),
            "execution:failure" => ("execution-failures", None),
            "streams:failure" => ("stream-failures", None),
            "cleanup:failure" => ("cleanup-failures", None),
            kind => return Err(Error::Invalid(format!("Export Evidence: Add a Named JSON Projection for Evidence Kind {kind}"))),
        };
        let mut payload = match message {
            Some(message) => project_message(message, &evidence.payload)?,
            None if evidence.kind.ends_with(":failure") => serde_json::json!(
                String::from_utf8(evidence.payload.clone())
                    .map_err(|source| Error::Invalid(format!("Project Failure: Restore Its UTF-8 Cause: {source}")))?
            ),
            None if evidence.kind == "read:statistics" => {
                let read = StatisticsRead::decode(evidence.payload.as_slice())?;
                serde_json::json!({"request":read.request.as_ref().map(|request| project_message("dokime.host.v1.EvidenceStatisticsRequest", &request.encode_to_vec())).transpose()?,"positions":read.positions,"response":read.response.as_ref().map(|response| project_message("dokime.host.v1.EvidenceStatisticsResponse", &response.encode_to_vec())).transpose()?})
            }
            None => serde_json::json!(evidence.payload),
        };
        if kind == "records" {
            let record = RecordsAppendRequest::decode(evidence.payload.as_slice())?;
            let catalog = Declarations::decode(execution.catalog.as_slice())?;
            let pool = catalog
                .types
                .map(DescriptorPool::from_file_descriptor_set)
                .transpose()
                .map_err(|source| Error::Invalid(format!("Project Records: Restore Their Frozen Types: {source}")))?;
            for section in catalog.sections {
                if section.r#type != "dokime.model.v1.RecordContracts" {
                    continue;
                }
                let contracts = RecordContracts::decode(section.payload.as_slice())?;
                let Some(contract) = contracts
                    .records
                    .iter()
                    .find(|contract| contract.metadata.as_ref().is_some_and(|metadata| metadata.identifier == record.kind))
                else {
                    continue;
                };
                let descriptor = pool
                    .as_ref()
                    .and_then(|pool| pool.get_message_by_name(&contract.message))
                    .ok_or(Error::Data("Project Records: Restore Their Frozen Row Descriptor"))?;
                let row = DynamicMessage::decode(descriptor, record.payload.as_slice())?;
                payload["payload"] = project_dynamic(&row)?;
            }
        }
        kinds.get_mut(kind).unwrap().push(serde_json::json!({"position":evidence.position,"kind":evidence.kind,"step":evidence.step,"operation":evidence.operation,"attempt":evidence.attempt,"source_ns":evidence.source_ns,"receipt_ns":evidence.receipt_ns,"payload":payload}));
    }
    for note in &archive.notes {
        kinds.get_mut("notes").unwrap().push(project_message("dokime.host.v1.evidence.OperatorNote", &note.encode_to_vec())?);
    }
    for artifact in &archive.artifacts {
        kinds.get_mut("artifacts").unwrap().push(artifact_projection(artifact)?);
    }
    for file in &archive.files {
        kinds.get_mut("files").unwrap().push(serde_json::json!({"metadata":file.metadata.as_ref().map(|metadata| project_message("dokime.host.v1.content.ExecutionFileMetadata", &metadata.encode_to_vec())).transpose()?}));
    }
    if let Some(outcome) = &execution.outcome {
        kinds.get_mut("outcome").unwrap().push(project_message("dokime.host.v1.ExecutionOutcome", &outcome.encode_to_vec())?);
    }
    if let Some(origin::Kind::PluginRun(run)) = execution.origin.as_ref().and_then(|origin| origin.kind.as_ref())
        && let Some(case) = &run.test_case
    {
        kinds
            .get_mut("parameters")
            .unwrap()
            .push(project_message("dokime.model.v1.MapValue", &MapValue { values: case.parameters.clone() }.encode_to_vec())?);
    }
    Ok(kinds.into_iter().map(|(kind, values)| (kind, serde_json::Value::Array(values))).collect())
}

fn artifact_projection(artifact: &StoredArtifact) -> Result<serde_json::Value> {
    Ok(
        serde_json::json!({"artifact":artifact.artifact.as_ref().map(|metadata| project_message("dokime.model.v1.ArtifactMetadata", &metadata.encode_to_vec())).transpose()?,"request":artifact.request.as_ref().map(|request| project_message("dokime.host.v1.content.ArtifactsCreateRequest", &request.encode_to_vec())).transpose()?}),
    )
}

fn project_message(name: &str, bytes: &[u8]) -> Result<serde_json::Value> {
    let descriptor = dokime_protocol::descriptor_pool()
        .get_message_by_name(name)
        .ok_or_else(|| Error::Invalid(format!("Project Evidence: Restore Descriptor {name}")))?;
    let message = DynamicMessage::decode(descriptor, bytes)?;
    project_dynamic(&message)
}

fn project_dynamic(message: &DynamicMessage) -> Result<serde_json::Value> {
    let mut object = BTreeMap::<String, serde_json::Value>::new();
    for field in message.descriptor().fields() {
        if field.supports_presence() && !message.has_field(&field) {
            continue;
        }
        let value = message.get_field(&field);
        object.insert(field.name().into(), project_value(&value)?);
    }
    Ok(serde_json::to_value(object)?)
}

fn project_value(value: &Value) -> Result<serde_json::Value> {
    Ok(match value {
        Value::Bool(value) => serde_json::json!(value),
        Value::I32(value) => serde_json::json!(value),
        Value::I64(value) => serde_json::json!(value),
        Value::U32(value) => serde_json::json!(value),
        Value::U64(value) => serde_json::json!(value),
        Value::F32(value) if value.is_finite() => serde_json::json!(value),
        Value::F32(value) => serde_json::json!(value.to_string()),
        Value::F64(value) if value.is_finite() => serde_json::json!(value),
        Value::F64(value) => serde_json::json!(value.to_string()),
        Value::String(value) => serde_json::json!(value),
        Value::Bytes(value) => serde_json::json!(value.as_ref()),
        Value::EnumNumber(value) => serde_json::json!(value),
        Value::Message(value) => project_dynamic(value)?,
        Value::List(values) => serde_json::Value::Array(values.iter().map(project_value).collect::<Result<Vec<_>>>()?),
        Value::Map(values) => {
            let mut object = BTreeMap::<String, serde_json::Value>::new();
            for (key, value) in values {
                let key = match key {
                    MapKey::Bool(value) => value.to_string(),
                    MapKey::I32(value) => value.to_string(),
                    MapKey::I64(value) => value.to_string(),
                    MapKey::U32(value) => value.to_string(),
                    MapKey::U64(value) => value.to_string(),
                    MapKey::String(value) => value.clone(),
                };
                object.insert(key, project_value(value)?);
            }
            serde_json::to_value(object)?
        }
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Evidence, Execution};
    use dokime_protocol::host::{ExecutionOutcome, UnevaluatedOutcome, execution_outcome};
    use dokime_protocol::host::{PromptsCreateRequest, PromptsCreateResponse};
    use dokime_protocol::model::RecordContract;
    use dokime_protocol::model::{Metadata, Origin, PluginRun, Suite, Suites, Termination, TestCase, TestCaseTemplate};
    use dokime_protocol::plugin::CriteriaEvaluateResponse;
    use harmos_protocol::model::Section;
    use prost_types::FileDescriptorSet;
    use std::io::Cursor;
    use zip::{ZipArchive, extra_fields::ExtraField};

    #[test]
    fn archive_label_sanitization_preserves_units_and_spaces() {
        assert_eq!(sanitize_label("Temperature [°C] / A:B<C>D\\E|F?G*H\0"), "Temperature [°C] - A-B-C-D-E-F-G-H-");
        assert_eq!(sanitize_label("Keep . spaces \u{7f}\u{85}"), "Keep . spaces \u{7f}\u{85}");
    }

    #[test]
    fn archive_timestamps_retain_epoch_and_subsecond_seal_times() -> Result<()> {
        for (timestamp, ticks) in [
            (Timestamp { seconds: 0, nanos: 0 }, 116_444_736_000_000_000u64),
            (
                Timestamp {
                    seconds: 1_732_018_482,
                    nanos: 123_456_700,
                },
                133_764_920_821_234_567u64,
            ),
        ] {
            let mut bytes = Vec::new();
            let mut archive = ZipWriter::new_stream(&mut bytes);
            let mut entries = Vec::new();
            write_entry(&mut archive, "report.html", archive_options(&timestamp)?, b"report", &mut entries)?;
            archive.finish()?;
            let mut archive = ZipArchive::new(Cursor::new(bytes))?;
            let entry = archive.by_name("report.html")?;
            let modified = entry.extra_data_fields().find_map(|field| match field {
                ExtraField::Ntfs(value) => Some(value.mtime()),
                _ => None,
            });
            assert_eq!(modified, Some(ticks));
        }
        Ok(())
    }

    #[test]
    fn cycle_archive_has_ordered_folders_complete_manifests_and_seal_times() -> Result<()> {
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let time = Timestamp { seconds: 1_732_018_482, nanos: 0 };
        let suites = Suites {
            suites: vec![Suite {
                templates: vec![TestCaseTemplate {
                    metadata: Some(Metadata {
                        identifier: "confirmation".into(),
                        label: "Confirmation [°C]".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let catalog = Declarations {
            sections: vec![Section {
                r#type: "dokime.model.v1.Suites".into(),
                payload: suites.encode_to_vec(),
            }],
            ..Default::default()
        };
        let ids = ["z-first", "a-second", "n-never-started"];
        store.save_cycle(&CycleSummary {
            cycle_id: "cycle".into(),
            updated_at: Some(time),
            ..Default::default()
        })?;
        for (index, id) in ids.iter().enumerate() {
            let execution = Execution {
                identifier: (*id).into(),
                cycle: "cycle".into(),
                catalog: catalog.encode_to_vec(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::PluginRun(PluginRun {
                        plugin: "plugin".into(),
                        template: "confirmation".into(),
                        digest: vec![1],
                        started_at: Some(time),
                        test_case: Some(TestCase::default()),
                    })),
                }),
                ..Default::default()
            };
            store.create(&execution)?;
            if index < 2 {
                store.append(
                    id,
                    Evidence {
                        kind: "step:start".into(),
                        step: "confirm".into(),
                        attempt: 1,
                        receipt_ns: 42,
                        ..Default::default()
                    },
                )?;
                store.close(
                    id,
                    Termination::Completed,
                    ExecutionOutcome {
                        attribution: Some(execution_outcome::Attribution::NotEvaluated(UnevaluatedOutcome {
                            termination: Termination::Completed as i32,
                            reason: String::new(),
                        })),
                    },
                )?;
                store.seal(
                    id,
                    Timestamp {
                        seconds: time.seconds + index as i64 * 2,
                        nanos: 0,
                    },
                )?;
            } else {
                store.seal_unstarted(
                    id,
                    Timestamp {
                        seconds: time.seconds + index as i64 * 2,
                        nanos: 0,
                    },
                )?;
            }
        }
        let mut outputs = Vec::new();
        for _ in 0..2 {
            let mut contents = HashMap::new();
            for id in ids {
                contents.insert(
                    id.into(),
                    ArchivedContent {
                        template_label: Some("Confirmation [°C]".into()),
                        report: Some(report(id.as_bytes())?),
                        ..Default::default()
                    },
                );
            }
            let content = ArchivedContent {
                report: Some(report(b"cycle")?),
                ..Default::default()
            };
            let mut bytes = Vec::new();
            store.write_cycle_archive("cycle", NonZeroUsize::new(1).unwrap(), &mut bytes, &mut contents, content)?;
            outputs.push(bytes);
        }
        assert_eq!(Sha256::digest(&outputs[0]), Sha256::digest(&outputs[1]));
        let mut archive = ZipArchive::new(Cursor::new(&outputs[0]))?;
        let data = [
            "artifacts",
            "capture-gaps",
            "cleanup-failures",
            "criteria",
            "documents",
            "execution",
            "execution-failures",
            "files",
            "logs",
            "notes",
            "outcome",
            "parameters",
            "progress",
            "prompts",
            "records",
            "reports",
            "statistics",
            "steps",
            "stream-failures",
            "traceability",
        ];
        let mut expected = Vec::new();
        for index in 1..=3 {
            let prefix = format!("executions/{index:02} Confirmation [°C]/");
            expected.extend(data.iter().map(|kind| format!("{prefix}data/{kind}.json")));
            expected.push(format!("{prefix}report.html"));
            expected.push(format!("{prefix}manifest.json"));
        }
        expected.extend(["data/cycle.json".into(), "report.html".into(), "manifest.json".into()]);
        assert_eq!(archive.file_names().collect::<result::Result<Vec<_>, _>>()?, expected.iter().map(String::as_str).collect::<Vec<_>>());
        for name in &expected {
            let mut entry = archive.by_name(name)?;
            let timestamp = match name.strip_prefix("executions/") {
                Some(suffix) if !suffix.is_empty() => Timestamp {
                    seconds: time.seconds + (suffix[..2].parse::<i64>().unwrap() - 1) * 2,
                    nanos: 0,
                },
                _ => time,
            };
            assert_eq!(
                entry.last_modified(),
                Some(DateTime::from_date_and_time(2024, 11, 19, 12, 14, 42 + (timestamp.seconds - time.seconds) as u8).unwrap())
            );
            if !name.ends_with("manifest.json") {
                continue;
            }
            let manifest: serde_json::Value = serde_json::from_reader(&mut entry)?;
            drop(entry);
            let prefix = name.strip_suffix("manifest.json").unwrap();
            for member in manifest["entries"].as_array().unwrap() {
                let mut bytes = Vec::new();
                archive.by_name(&format!("{prefix}{}", member["path"].as_str().unwrap()))?.read_to_end(&mut bytes)?;
                assert_eq!(member["size"], bytes.len());
                assert_eq!(member["sha256"], hex(&Sha256::digest(&bytes)));
            }
        }
        let steps: serde_json::Value = serde_json::from_reader(archive.by_name("executions/01 Confirmation [°C]/data/steps.json")?)?;
        assert_eq!(
            steps,
            serde_json::json!([{"attempt":1,"kind":"step:start","operation":"","payload":[],"position":1,"receipt_ns":42,"source_ns":0,"step":"confirm"}])
        );
        let outcome: serde_json::Value = serde_json::from_reader(archive.by_name("executions/03 Confirmation [°C]/data/outcome.json")?)?;
        assert_eq!(outcome[0]["not_evaluated"]["termination"], Termination::NotRun as i32);
        Ok(())
    }

    #[test]
    fn evidence_projection_decodes_fixed_records_criteria_prompts_notes_and_parameters() -> Result<()> {
        let value = dokime_protocol::model::Value {
            kind: Some(dokime_protocol::model::value::Kind::UnsignedInteger(u64::MAX)),
        };
        let contracts = RecordContracts {
            records: vec![RecordContract {
                metadata: Some(Metadata {
                    identifier: "reading".into(),
                    ..Default::default()
                }),
                message: "dokime.model.v1.Value".into(),
            }],
        };
        let catalog = Declarations {
            types: Some(FileDescriptorSet::decode(dokime_protocol::descriptor_pool().encode_to_vec().as_slice())?),
            sections: vec![Section {
                r#type: "dokime.model.v1.RecordContracts".into(),
                payload: contracts.encode_to_vec(),
            }],
            ..Default::default()
        };
        let execution = Execution {
            catalog: catalog.encode_to_vec(),
            origin: Some(Origin {
                kind: Some(origin::Kind::PluginRun(PluginRun {
                    test_case: Some(TestCase {
                        parameters: HashMap::from([("limit".into(), value.clone())]),
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };
        let record = RecordsAppendRequest {
            kind: "reading".into(),
            payload: value.encode_to_vec(),
            ..Default::default()
        };
        let criterion = CriteriaEvaluateResponse { passed: true };
        let prompt = PromptsCreateRequest {
            prompt: "confirm".into(),
            ..Default::default()
        };
        let answer = PromptsCreateResponse {
            prompt: "confirm".into(),
            accepted: true,
            ..Default::default()
        };
        let evidence = [
            ("record:reading", record.encode_to_vec()),
            ("criterion:confirmed", criterion.encode_to_vec()),
            ("prompt", prompt.encode_to_vec()),
            ("answer", answer.encode_to_vec()),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (kind, payload))| Evidence {
            position: index as u64 + 1,
            kind: kind.into(),
            payload,
            ..Default::default()
        })
        .collect();
        let archive = ArchivedExecution {
            execution,
            evidence,
            notes: vec![OperatorNote {
                note_id: "note".into(),
                ..Default::default()
            }],
            seal: Vec::new(),
            reports: Vec::new(),
            artifacts: Vec::new(),
            files: Vec::new(),
        };
        let projection = evidence_projection(&archive)?;
        assert_eq!(projection["records"][0]["payload"]["payload"]["unsigned_integer"], u64::MAX);
        assert_eq!(projection["criteria"][0]["payload"]["passed"], true);
        assert_eq!(projection["prompts"][1]["payload"]["accepted"], true);
        assert_eq!(projection["notes"][0]["note_id"], "note");
        assert_eq!(projection["parameters"][0]["values"]["limit"]["unsigned_integer"], u64::MAX);
        let json = serde_json::to_string(&projection["records"])?;
        assert!(json.contains("18446744073709551615"));
        assert!(!json.contains("18,446"));
        Ok(())
    }

    fn report(bytes: &[u8]) -> Result<ArchivedNativeFile> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(bytes)?;
        Ok(ArchivedNativeFile {
            path: file.into_temp_path(),
            reference: FileReference {
                length: bytes.len() as u64,
                sha256: Sha256::digest(bytes).to_vec(),
                ..Default::default()
            },
        })
    }
}
