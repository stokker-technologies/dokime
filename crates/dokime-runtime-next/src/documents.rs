/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fixed Snapshot Document Rendering

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::fs::File;
use std::{
    io::{BufWriter, Write},
    num::NonZeroUsize,
    path::Path,
};

// Third Party Imports
use harmos_protocol::v1 as h;
use harmos_protocol::v1::Catalog;
use harmos_protocol::v1::ReadCut;
use prost::Message;

use tempfile::NamedTempFile;

// Local Imports
use crate::{Error, Host, Result};
use dokime_operations_next::Result as OperationsResult;
use dokime_operations_next::client::HostIdentity;
use dokime_protocol::client::ReportMetadata;
use dokime_protocol::client::ReportSubject;
use dokime_protocol::v1;
use dokime_protocol::v1::Document;
use dokime_protocol::v1::DocumentScope;
use dokime_protocol::v1::Plot;
use dokime_protocol::v1::PlotFilter;
use dokime_protocol::v1::Section;
use dokime_storage::{Execution, ExecutionCut, Report, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct FixedSnapshot {
    store: Store,
    cut: ExecutionCut,
    counts: dokime_storage::OutcomeCounts,
    _read_cancellation: Option<dokime_storage::ReadCancellation>,
}

#[derive(Clone, Copy)]
pub struct RenderSettings {
    pub page_size: NonZeroUsize,
    pub display_points: NonZeroUsize,
}

pub struct RenderedDocument {
    output: NamedTempFile,
    pub(crate) execution: String,
    pub(crate) execution_seal: Vec<u8>,
    pub(crate) document: v1::Document,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl FixedSnapshot {
    pub fn execution(&self) -> &Execution {
        &self.cut.execution
    }
    pub fn seal(&self) -> &[u8] {
        &self.cut.seal
    }
}
impl RenderedDocument {
    pub fn path(&self) -> &Path {
        self.output.path()
    }
    pub fn into_file(self) -> File {
        self.output.into_file()
    }
    pub fn html(&self) -> Result<Vec<u8>> {
        Ok(std::fs::read(self.path())?)
    }
}
impl Host {
    pub(crate) fn render_report(
        &self, subject: &ReportSubject, document_id: &str, at: Option<&ReadCut>, identity: &HostIdentity, render_settings: RenderSettings,
    ) -> OperationsResult<(File, ReportMetadata)> {
        use std::io::{Seek, Write};
        let (store, _read_cancellation) = self.store.read_session()?;
        let (execution, cycle) = match subject.kind.as_ref() {
            Some(dokime_protocol::client::report_subject::Kind::Execution(id)) => (Some(id.as_str()), store.metadata(id)?.execution.cycle),
            Some(dokime_protocol::client::report_subject::Kind::Cycle(id)) => (None, id.clone()),
            None => return Err(dokime_operations_next::Error::Invalid("Render Report: Select an Execution or Cycle".into())),
        };
        let mut snapshots = Vec::new();
        let mut after = String::new();
        loop {
            let page = store.cycle_execution_page(&cycle, &after, render_settings.page_size)?;
            if page.is_empty() {
                break;
            }
            for id in page {
                after = id.clone();
                if execution.is_some_and(|selected| selected != id) {
                    continue;
                }
                let cut = store.metadata(&id)?;
                if cut.execution.cycle != cycle {
                    continue;
                }
                let scope = v1::DocumentScope {
                    cycle_id: cycle.clone(),
                    execution_id: id,
                };
                let (document, mut cut) = self
                    .current_document_in(&store, &scope, document_id)
                    .map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
                if let Some(at) = at {
                    let history = harmos_protocol::v1::History {
                        store: identity.backend_id.clone(),
                        epoch: scope.execution_id.clone(),
                    };
                    if execution.is_none()
                        || at.positions.len() != 1
                        || !at.revisions.is_empty()
                        || at.positions[0].history.as_ref() != Some(&history)
                        || at.positions[0].sequence > cut.frontier
                    {
                        return Err(dokime_operations_next::Error::Invalid(
                            "Render Report: Refresh the Execution's Fixed Evidence Cursor".into(),
                        ));
                    }
                    cut.frontier = at.positions[0].sequence;
                }
                snapshots.push((
                    document,
                    self.snapshot_in(store.clone(), cut).map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?,
                ));
            }
        }
        if snapshots.is_empty() {
            return Err(dokime_operations_next::Error::Invalid("Render Report: Select an Existing Execution or Cycle".into()));
        }
        let file = if snapshots.len() == 1 {
            let (document, snapshot) = snapshots.remove(0);
            self.render_document(&snapshot, document, render_settings)
                .map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?
                .into_file()
        } else {
            let mut output = tempfile::tempfile().map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
            output
                .write_all(b"<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>Cycle Report</title><body>")
                .map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
            for (document, snapshot) in snapshots {
                let rendered = self
                    .render_document(&snapshot, document, render_settings)
                    .map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
                let mut file = rendered.into_file();
                // Each Execution Is an Independent Embedded HTML Document.
                let mut bytes = String::new();
                use std::io::Read;
                file.read_to_string(&mut bytes).map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
                let body = bytes.split_once("<body>").map_or(bytes.as_str(), |(_, body)| body).trim_end_matches("</body></html>");
                output.write_all(b"<article>").map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
                output.write_all(body.as_bytes()).map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
                output.write_all(b"</article>").map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
            }
            output.write_all(b"</body></html>").map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
            output.rewind().map_err(|error| dokime_operations_next::Error::Invalid(error.to_string()))?;
            output
        };
        let metadata = dokime_protocol::client::ReportMetadata {
            test_cycle_id: cycle,
            test_execution_id: execution.map(str::to_owned),
            identifier: if execution.is_some() { "execution".into() } else { "cycle".into() },
            origin: dokime_protocol::client::ReportOrigin::Current as i32,
            ..Default::default()
        };
        Ok((file, metadata))
    }

    pub fn current_document(&self, scope: &DocumentScope, identifier: &str) -> Result<(Document, ExecutionCut)> {
        self.current_document_in(&self.store, scope, identifier)
    }
    pub(crate) fn current_document_in(&self, store: &Store, scope: &DocumentScope, identifier: &str) -> Result<(Document, ExecutionCut)> {
        if scope.execution_id.is_empty() {
            if !identifier.is_empty() && identifier != "default" {
                return Err(Error::Invalid("Capture Cycle Document: Select the Default Document".into()));
            }
            let execution = store
                .cycle_execution_page(&scope.cycle_id, "", NonZeroUsize::new(1).unwrap())?
                .into_iter()
                .next()
                .ok_or_else(|| Error::Invalid("Capture Cycle Document: Create or Import an Execution in This Cycle First".into()))?;
            let mut cut = store.cut(&execution)?;
            cut.frontier = store.cycle_evidence_frontier(&scope.cycle_id)?;
            let sections = [
                ("events", "Records", v1::EvidenceCard::Records),
                ("artifacts", "Artifacts", v1::EvidenceCard::Artifacts),
                ("media", "Media", v1::EvidenceCard::Media),
            ]
            .into_iter()
            .map(|(id, label, kind)| v1::Section {
                metadata: Some(v1::Metadata {
                    identifier: format!("cycle.{id}"),
                    label: label.into(),
                    ..Default::default()
                }),
                cards: vec![v1::Card {
                    metadata: Some(v1::Metadata {
                        identifier: format!("cycle.{id}.list"),
                        label: label.into(),
                        ..Default::default()
                    }),
                    body: Some(v1::card::Body::Evidence(kind as i32)),
                    ..Default::default()
                }],
                ..Default::default()
            })
            .collect();
            return Ok((
                v1::Document {
                    metadata: Some(v1::Metadata {
                        identifier: "default".into(),
                        label: "Cycle Report".into(),
                        ..Default::default()
                    }),
                    sections,
                },
                cut,
            ));
        }
        let cut = store.cut(&scope.execution_id)?;
        if cut.execution.cycle != scope.cycle_id {
            return Err(Error::Invalid("Capture Document: Select the Execution in Its Own Cycle".into()));
        }
        let catalog = h::Catalog::decode(cut.execution.catalog.as_slice())?;
        let mut documents = Vec::new();
        for section in &catalog.sections {
            if section.r#type != "dokime.v1.Suites" {
                continue;
            }
            let suites = v1::Suites::decode(section.payload.as_slice())?;
            for suite in suites.suites {
                for template in suite.templates {
                    let matched = match cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
                        Some(v1::origin::Kind::PluginRun(run)) => {
                            template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == run.template)
                        }
                        _ => true,
                    };
                    if matched {
                        documents.extend(template.documents.into_iter().filter(|document| {
                            identifier.is_empty() || document.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier)
                        }));
                    }
                }
            }
        }
        if documents.is_empty() && identifier.is_empty() {
            documents.push(v1::Document {
                metadata: Some(v1::Metadata {
                    identifier: "execution".into(),
                    label: "Execution Report".into(),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
        if documents.len() != 1 {
            return Err(Error::Invalid("Capture Document: Select One Document ID from the Frozen Execution Catalog".into()));
        }
        let mut document = documents.remove(0);
        let id = document.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
        apply_preferences(store, &cut.execution.identifier, id, &mut document.sections)?;
        Ok((document, cut))
    }

    pub fn snapshot(&self, execution: &str) -> Result<FixedSnapshot> {
        let cut = self.store.cut(execution)?;
        self.snapshot_cut(cut)
    }
    pub fn snapshot_cut(&self, cut: ExecutionCut) -> Result<FixedSnapshot> {
        let (store, guard) = self.store.read_session()?;
        let mut snapshot = self.snapshot_in(store, cut)?;
        snapshot._read_cancellation = Some(guard);
        Ok(snapshot)
    }
    pub(crate) fn snapshot_in(&self, store: Store, cut: ExecutionCut) -> Result<FixedSnapshot> {
        let outcomes = store.cycle_outcomes(&cut.execution.cycle)?;
        let counts = dokime_storage::OutcomeCounts::from_outcomes(&outcomes);
        Ok(FixedSnapshot {
            store,
            cut,
            counts,
            _read_cancellation: None,
        })
    }
    pub fn render_document(&self, snapshot: &FixedSnapshot, document: Document, settings: RenderSettings) -> Result<RenderedDocument> {
        let (store, cancellation) = snapshot.store.read_session()?;
        let snapshot = FixedSnapshot {
            store,
            cut: snapshot.cut.clone(),
            counts: snapshot.counts.clone(),
            _read_cancellation: Some(cancellation),
        };
        let mut file = NamedTempFile::with_prefix("dokime-render-")?;
        let mut output = BufWriter::new(&mut file);
        let title = document.metadata.as_ref().map_or("", |metadata| metadata.label.as_str());
        write!(
            output,
            "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>{}</title><body><h1>{}</h1>",
            escape(title),
            escape(title)
        )?;
        write!(
            output,
            "<dl><dt>Execution</dt><dd>{}</dd><dt>Cycle</dt><dd>{}</dd></dl>",
            escape(&snapshot.cut.execution.identifier),
            escape(&snapshot.cut.execution.cycle)
        )?;
        let counts = &snapshot.counts;
        write!(
            output,
            "<dl><dt>Evaluated</dt><dd>{}</dd><dt>Not Evaluated</dt><dd>{}</dd><dt>Declared</dt><dd>{}</dd></dl>",
            counts.evaluated, counts.not_evaluated, counts.declared
        )?;
        let catalog = h::Catalog::decode(snapshot.cut.execution.catalog.as_slice())?;
        let mut offset = 0;
        loop {
            let expansion = dokime_operations_next::client::document_expansion::expand(
                &snapshot.store,
                &snapshot.cut,
                &document,
                offset,
                settings.page_size.get(),
            )
            .map_err(|error| Error::Invalid(error.to_string()))?;
            for section in &expansion.document.sections {
                render_section(&mut output, &snapshot, section, &catalog, settings)?;
            }
            match expansion.next {
                Some(next) => offset = next,
                None => break,
            }
        }
        write!(output, "</body></html>")?;
        output.flush()?;
        drop(output);
        Ok(RenderedDocument {
            output: file,
            execution: snapshot.cut.execution.identifier.clone(),
            execution_seal: snapshot.cut.seal.clone(),
            document,
        })
    }
    pub fn publish_report(&self, output: &RenderedDocument) -> Result<Report> {
        self.reports.publish(output)
    }
    pub fn reports(&self, execution: &str) -> Result<Vec<Report>> {
        Ok(self.store.reports(execution)?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn render_section(
    output: &mut impl Write, snapshot: &FixedSnapshot, section: &Section, catalog: &Catalog, settings: RenderSettings,
) -> Result<()> {
    if section.repeat.is_some() {
        return Err(Error::Invalid("Render Document: Expand Repeated Sections Before Rendering".into()));
    }
    let label = section.metadata.as_ref().map_or("", |metadata| metadata.label.as_str());
    write!(output, "<section><h2>{}</h2>", escape(label))?;
    for card in &section.cards {
        let filters = card
            .filters
            .iter()
            .map(|filter| v1::PlotFilter {
                expression: filter.expression.clone(),
                operator: filter.operator,
                value: filter.value.clone(),
            })
            .collect::<Vec<_>>();
        match card.body.as_ref() {
            Some(v1::card::Body::Markdown(text)) => write!(output, "<pre>{}</pre>", escape(text))?,
            Some(v1::card::Body::Plot(plot)) => render_plot(output, snapshot, plot, &filters, catalog, settings)?,
            Some(v1::card::Body::Table(table)) => {
                write!(output, "<table>")?;
                for row in &table.rows {
                    write!(output, "<tr>")?;
                    let mut cells = row.cells.iter().collect::<Vec<_>>();
                    cells.sort_by_key(|(key, _)| *key);
                    for (key, value) in cells {
                        write!(output, "<td data-column=\"{}\">{}</td>", escape(key), escape(&crate::plot_html::display(value)))?;
                    }
                    write!(output, "</tr>")?;
                }
                write!(output, "</table>")?;
            }
            Some(v1::card::Body::Summary(summary)) => {
                write!(output, "<dl>")?;
                for tile in &summary.tiles {
                    let value =
                        tile.value.as_ref().ok_or_else(|| Error::Invalid("Render Summary: Supply Each Tile's Expression".into()))?;
                    let value = snapshot.store.document_value_filtered(&snapshot.cut, value, &filters)?;
                    write!(
                        output,
                        "<dt>{}</dt><dd>{}</dd>",
                        escape(tile.metadata.as_ref().map_or("", |metadata| metadata.label.as_str())),
                        escape(&crate::plot_html::display(&value))
                    )?;
                }
                write!(output, "</dl>")?;
            }
            Some(v1::card::Body::Evidence(kind)) => {
                if !filters.is_empty() {
                    return Err(Error::Invalid("Render Evidence Card: Clear Its Filters Until Evidence-Card Filtering Is Served".into()));
                }
                render_evidence(output, snapshot, *kind, settings)?;
            }
            None => return Err(Error::Invalid("Render Document: Supply Each Card's Body".into())),
        }
    }
    for child in &section.sections {
        render_section(output, snapshot, child, catalog, settings)?;
    }
    write!(output, "</section>")?;
    Ok(())
}

fn render_plot(
    output: &mut impl Write, snapshot: &FixedSnapshot, plot: &Plot, filters: &[PlotFilter], _catalog: &Catalog, settings: RenderSettings,
) -> Result<()> {
    let scope = v1::EvidenceScope {
        execution_id: Some(snapshot.cut.execution.identifier.clone()),
        cycle_id: Some(snapshot.cut.execution.cycle.clone()),
        ..Default::default()
    };
    let mut request =
        dokime_operations_next::client::document_plot::request(plot, scope).map_err(|error| Error::Invalid(error.to_string()))?;
    request.filters.extend_from_slice(filters);
    request.max_points = Some(plot.display_points.unwrap_or(settings.display_points.get() as u32) as u64);
    let envelope = dokime_operations_next::client::plots_snapshots_list::snapshot_at(
        &snapshot.store,
        &snapshot.cut,
        &request,
        settings.display_points.get().max(4),
        0,
        "v1",
    )
    .map_err(|error| Error::Invalid(error.to_string()))?;
    let presentation = dokime_operations_next::client::document_plot::presentation(&snapshot.store, &snapshot.cut, plot)?;
    crate::plot_html::render(output, plot, &envelope, &presentation)
}
fn render_evidence(output: &mut impl Write, snapshot: &FixedSnapshot, kind: i32, settings: RenderSettings) -> Result<()> {
    let kind = v1::EvidenceCard::try_from(kind).map_err(|_| Error::Invalid("Render Evidence: Select a Declared Card Kind".into()))?;
    write!(output, "<ol>")?;
    if kind == v1::EvidenceCard::Notes {
        let mut after = String::new();
        loop {
            let page = snapshot.store.notes_page(
                Some(&snapshot.cut.execution.cycle),
                Some(&snapshot.cut.execution.identifier),
                true,
                &after,
                settings.page_size.get(),
            )?;
            if page.is_empty() {
                break;
            }
            for note in page {
                after = note.note_id.clone();
                write!(output, "<li data-note=\"{}\">{}</li>", escape(&note.note_id), escape(&note.message))?;
            }
        }
    } else if matches!(kind, v1::EvidenceCard::Artifacts | v1::EvidenceCard::Media) {
        let mut after = String::new();
        loop {
            let page = snapshot.store.artifacts_page(&after, settings.page_size.get())?;
            if page.is_empty() {
                break;
            }
            for artifact in page {
                after = artifact.artifact_id.clone();
                if artifact.scope.as_ref().and_then(|scope| scope.execution_id.as_deref()) != Some(&snapshot.cut.execution.identifier) {
                    continue;
                }
                let media = artifact
                    .content_type
                    .as_ref()
                    .is_some_and(|mime| mime.starts_with("image/") || mime.starts_with("audio/") || mime.starts_with("video/"));
                if kind == v1::EvidenceCard::Media && !media {
                    continue;
                }
                write!(
                    output,
                    "<li data-artifact=\"{}\">{} ({} bytes)</li>",
                    escape(&artifact.artifact_id),
                    escape(&artifact.filename),
                    artifact.size_bytes
                )?;
            }
        }
    } else if kind == v1::EvidenceCard::StreamContext {
        let catalog = h::Catalog::decode(snapshot.cut.execution.catalog.as_slice())?;
        for stream in catalog.streams {
            write!(output, "<li>{}</li>", escape(&stream.id))?;
        }
    } else {
        let mut after = 0;
        loop {
            let page =
                snapshot.store.evidence_page(&snapshot.cut.execution.identifier, after, snapshot.cut.frontier, settings.page_size)?;
            if page.is_empty() {
                break;
            }
            for evidence in page {
                after = evidence.position;
                let selected = match kind {
                    v1::EvidenceCard::Steps => evidence.kind.starts_with("step:"),
                    v1::EvidenceCard::Criteria => evidence.kind.starts_with("criterion:"),
                    v1::EvidenceCard::Traceability => evidence.kind.starts_with("traceability"),
                    v1::EvidenceCard::Records => evidence.kind.starts_with("record:"),
                    v1::EvidenceCard::Health => evidence.kind.starts_with("health:"),
                    _ => return Err(Error::Invalid("Render Evidence: Select a Declared Card Kind".into())),
                };
                if !selected {
                    continue;
                }
                if kind == v1::EvidenceCard::Records {
                    let record = dokime_protocol::context::evidence::RecordsAppendRequest::decode(evidence.payload.as_slice())?;
                    let catalog = h::Catalog::decode(snapshot.cut.execution.catalog.as_slice())?;
                    let mut message = None;
                    for section in &catalog.sections {
                        if section.r#type == "dokime.v1.RecordContracts" {
                            let records = v1::RecordContracts::decode(section.payload.as_slice())?;
                            message = records
                                .records
                                .into_iter()
                                .find(|contract| contract.metadata.as_ref().is_some_and(|metadata| metadata.identifier == record.kind))
                                .map(|contract| contract.message);
                        }
                    }
                    let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(
                        catalog.types.ok_or_else(|| Error::Invalid("Render Records: Restore Frozen Descriptors".into()))?,
                    )
                    .map_err(|error| Error::Invalid(error.to_string()))?;
                    let descriptor = message
                        .and_then(|message| pool.get_message_by_name(&message))
                        .ok_or_else(|| Error::Invalid("Render Records: Restore This Record's Frozen Descriptor".into()))?;
                    let row = prost_reflect::DynamicMessage::decode(descriptor, record.payload.as_slice())?;
                    write!(output, "<li>{}<dl>", escape(&record.kind))?;
                    for (field, value) in row.fields() {
                        let value = match value {
                            prost_reflect::Value::String(value) => value.clone(),
                            prost_reflect::Value::Bool(value) => value.to_string(),
                            prost_reflect::Value::I32(value) => value.to_string(),
                            prost_reflect::Value::I64(value) => value.to_string(),
                            prost_reflect::Value::U32(value) => value.to_string(),
                            prost_reflect::Value::U64(value) => value.to_string(),
                            prost_reflect::Value::F32(value) => value.to_string(),
                            prost_reflect::Value::F64(value) => value.to_string(),
                            _ => continue,
                        };
                        write!(output, "<dt>{}</dt><dd>{}</dd>", escape(field.name()), escape(&value))?;
                    }
                    write!(output, "</dl></li>")?;
                } else {
                    write!(
                        output,
                        "<li data-position=\"{}\">{} {}</li>",
                        evidence.position,
                        escape(&evidence.kind),
                        escape(&evidence.step)
                    )?;
                }
            }
        }
    }
    write!(output, "</ol>")?;
    Ok(())
}
fn apply_preferences(store: &Store, execution: &str, document: &str, sections: &mut [Section]) -> Result<()> {
    for section in sections {
        for card in &mut section.cards {
            if let Some(v1::card::Body::Plot(plot)) = &mut card.body {
                let id = plot.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
                if let Some((points, _)) = store.plot_preferences(execution, document, id)? {
                    plot.display_points = points;
                }
            }
        }
        apply_preferences(store, execution, document, &mut section.sections)?;
    }
    Ok(())
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

/* === Tests === */
