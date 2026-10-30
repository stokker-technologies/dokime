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
    io::{self, Write},
};

// Third Party Imports
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};

// Local Imports
use super::*;
use crate::report::cycle::{AnnotationKind, Index};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type ReadMember<'a> = dyn Fn(usize) -> Result<crate::report::html::Member> + 'a;

/* === Structs === */

pub(in crate::report) struct Report<'a> {
    pub report_defaults: &'a crate::config::ReportDefaults,
    pub header: Header<'a>,
    pub index: &'a Index,
    pub appearance: &'a crate::installed_plugins::RuntimeAppearanceFrame,
    pub read: &'a ReadMember<'a>,
}

// Render Every Section While Its Typed Member Is Borrowed; Retain Only Encoded Rows,
// Spilled To Disk Only Above The Connection's Page Cache.
pub(in crate::report) struct MemberRows {
    connection: rusqlite::Connection,
}

struct Rows<'a, 'r>(&'a Report<'r>, Section);

struct DocumentWarningText<'a>(&'a str, &'a str);

struct SnapshotRows<'a, 'r>(&'a Report<'r>);

struct PlanRows<'a, 'r>(&'a Report<'r>);

struct Telemetry<'a, 'r>(&'a Report<'r>);

struct Samples<'a, 'r>(&'a Report<'r>, &'a [crate::report::cycle::Sample]);

struct Steps<'a, 'r>(&'a Report<'r>, &'a crate::report::cycle::Member);

struct Annotations<'a, 'r>(&'a Report<'r>, AnnotationKind);

/* === Enums === */

#[derive(Clone, Copy)]
enum Section {
    Artifacts,
    Cases,
    Records,
    Executions,
    Warnings,
    Templates,
    PlanCases,
    Queue,
}

/* === Traits === */

/* === Implementations === */

impl Report<'_> {
    pub(crate) fn write(&self, writer: &mut dyn Write) -> Result<()> {
        output::write_html(writer, self, REPORT_APP_TEMPLATE).map_err(|source| Error::InvalidReportInputs(source.to_string()))
    }
}

impl Report<'_> {
    pub(in crate::report) fn write_prepared(&self, writer: &mut dyn Write, rows: &MemberRows) -> Result<()> {
        output::write_html_with(writer, REPORT_APP_TEMPLATE, |writer| {
            let mut fields = output::Object::new(writer)?;
            fields.field("appearance", self.appearance)?;
            fields.field("report_defaults", self.report_defaults)?;
            fields.raw("case_runs", |writer| rows.array(writer, "cases"))?;
            fields.field("documents", &self.header.documents)?;
            fields.field("generated_at", &self.header.generated_at)?;
            fields.field(
                "hub",
                &Hub {
                    generator: "dokime-runtime",
                    name: "Dokime",
                    version: env!("CARGO_PKG_VERSION"),
                },
            )?;
            fields.field("plots", &self.header.plots.unwrap_or(&BTreeMap::new()))?;
            fields.field("schema_version", &1u8)?;
            fields.raw("snapshot", |writer| self.write_snapshot(writer, rows))?;
            fields.field("test_cycle_plan", &PlanRows(self))?;
            fields.field("test_executions", &Rows(self, Section::Executions))?;
            fields.raw("warnings", |writer| {
                writer.write_all(b"[")?;
                let mut first = true;
                for warning in [
                    (self.index.members == 0)
                        .then_some("report.no_executions: No executions are available yet; this report reflects the current cycle plan."),
                    self.header
                        .cycle
                        .result
                        .is_none()
                        .then_some("report.no_result_summary: No result summary has been produced for this cycle."),
                ]
                .into_iter()
                .flatten()
                {
                    separator(writer, &mut first)?;
                    output::write_json(writer, &warning)?;
                }
                rows.visit("warnings", |bytes| {
                    separator(writer, &mut first)?;
                    writer.write_all(bytes)
                })?;
                writer.write_all(b"]")
            })?;
            fields.finish()
        })
        .map_err(|source| Error::InvalidReportInputs(source.to_string()))
    }

    fn write_snapshot(&self, writer: &mut dyn Write, rows: &MemberRows) -> io::Result<()> {
        let snapshot = snapshot(self.header);
        let mut fields = output::Object::new(writer)?;
        fields.field("run", &snapshot.run)?;
        fields.field("session_id", &snapshot.session_id)?;
        fields.field("title", &snapshot.title)?;
        fields.field("revision", &snapshot.revision)?;
        fields.raw("records", |writer| rows.array(writer, "records"))?;
        fields.raw("telemetry", |writer| {
            let mut streams = output::Object::new(writer)?;
            let mut remaining = self.index.samples.as_slice();
            while let Some(first) = remaining.first() {
                let count = remaining.iter().take_while(|sample| sample.stream == first.stream).count();
                let (samples, rest) = remaining.split_at(count);
                remaining = rest;
                let maximum = self.report_defaults.plot_points as usize;
                let returned = selection(count, maximum).count();
                streams.raw(first.stream.as_str(), |writer| {
                    let mut stream = output::Object::new(writer)?;
                    stream.field("stream", &first.stream.as_str())?;
                    stream.field("spec", &None::<()>)?;
                    stream.field("summary", &BTreeMap::<(), ()>::new())?;
                    stream.raw("samples", |writer| {
                        writer.write_all(b"[")?;
                        let mut first = true;
                        for sample in selected_samples(samples, maximum) {
                            separator(writer, &mut first)?;
                            rows.row(writer, "samples", sample.member, sample.sample)?;
                        }
                        writer.write_all(b"]")
                    })?;
                    stream.field("max_samples", &maximum)?;
                    stream.field("total_samples", &count)?;
                    stream.field("dropped_samples", &(count - returned))?;
                    stream.field("downsampled", &(returned < count))?;
                    stream.field("updated_at", &samples.last().unwrap().timestamp)?;
                    stream.finish()
                })?;
            }
            streams.finish()
        })?;
        fields.field("pending_prompts", &snapshot.pending_prompts)?;
        for (name, kind, section) in [("notes", AnnotationKind::Note, "notes"), ("health", AnnotationKind::Health, "health")] {
            fields.raw(name, |writer| {
                writer.write_all(b"[")?;
                let mut first = true;
                for annotation in self.index.annotations.iter().filter(|annotation| annotation.kind == kind) {
                    separator(writer, &mut first)?;
                    rows.row(writer, section, annotation.member, annotation.position)?;
                }
                writer.write_all(b"]")
            })?;
        }
        fields.field("result_summary", &snapshot.result_summary)?;
        fields.field("traceability", &snapshot.traceability)?;
        fields.raw("artifacts", |writer| rows.array(writer, "artifacts"))?;
        fields.field("updated_at", &snapshot.updated_at)?;
        fields.finish()
    }
}

impl MemberRows {
    pub(in crate::report) fn new() -> Result<Self> {
        // A Private Temporary Database Creates Its Backing File Only When It Spills.
        let connection = rusqlite::Connection::open("").map_err(rows_error)?;
        connection
            .execute_batch(
                "PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA temp_store=FILE;
             PRAGMA cache_size=-8192; PRAGMA mmap_size=0;
             CREATE TABLE rows(section TEXT,member INTEGER,ordinal INTEGER,payload BLOB NOT NULL,
             PRIMARY KEY(section,member,ordinal)) WITHOUT ROWID; BEGIN;",
            )
            .map_err(rows_error)?;
        Ok(Self { connection })
    }

    pub(in crate::report) fn capture(
        &self, member: usize, captured: &crate::report::CapturedExecutionReportInput, header: Header<'_>,
    ) -> Result<()> {
        let input = &captured.input;
        let execution = &input.cycle.test_executions[0];
        self.append("cases", member, 0, &case_session(input, execution.snapshot.case()))?;
        for (ordinal, link) in input.artifact_links.iter().enumerate() {
            self.append("artifacts", member, ordinal, &ArtifactRow { link, download_href: &link.href })?;
        }
        for (ordinal, record) in input.records.iter().enumerate() {
            self.append("records", member, ordinal, &record_row(header, &record))?;
        }
        for (ordinal, (code, message)) in crate::report::document::document_warnings(input).enumerate() {
            self.append("warnings", member, ordinal, &DocumentWarningText(code, message))?;
        }
        for (ordinal, sample) in input.telemetry.iter().enumerate() {
            self.append("samples", member, ordinal, &telemetry::SampleValues(sample))?;
        }
        for ordinal in crate::report::previews::indices(captured.context.notes.len(), captured.report_defaults.table_rows as usize) {
            let note = &captured.context.notes[ordinal];
            self.append(
                "notes",
                member,
                ordinal,
                &NoteRow {
                    note_id: note.note_id.as_str(),
                    author: note.author.as_deref(),
                    text: &note.message,
                    timestamp: &note.created_at,
                    scope: &note.scope,
                },
            )?;
        }
        for ordinal in crate::report::previews::indices(captured.context.health.len(), captured.report_defaults.table_rows as usize) {
            let item = &captured.context.health[ordinal];
            self.append(
                "health",
                member,
                ordinal,
                &HealthRow {
                    health_id: item.health_item_id.as_str(),
                    level: &item.severity,
                    title: &item.message,
                    message: item.detail.as_deref(),
                    details: BTreeMap::new(),
                    created_at: &item.observed_at,
                    scope: &item.scope,
                },
            )?;
        }
        Ok(())
    }

    fn append(&self, section: &str, member: usize, ordinal: usize, value: &impl Serialize) -> Result<()> {
        let member = i64::try_from(member).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        let ordinal = i64::try_from(ordinal).map_err(|error| Error::InvalidReportInputs(error.to_string()))?;
        let mut bytes = Vec::new();
        output::write_json(&mut bytes, value)?;
        self.connection
            .prepare_cached("INSERT INTO rows VALUES (?1,?2,?3,?4)")
            .map_err(rows_error)?
            .execute(rusqlite::params![section, member, ordinal, bytes])
            .map_err(rows_error)?;
        Ok(())
    }

    fn visit(&self, section: &str, mut visit: impl FnMut(&[u8]) -> io::Result<()>) -> io::Result<()> {
        let mut statement = self
            .connection
            .prepare_cached("SELECT payload FROM rows WHERE section=?1 ORDER BY member,ordinal")
            .map_err(io::Error::other)?;
        let mut rows = statement.query([section]).map_err(io::Error::other)?;
        while let Some(row) = rows.next().map_err(io::Error::other)? {
            visit(row.get_ref(0).map_err(io::Error::other)?.as_blob().map_err(io::Error::other)?)?;
        }
        Ok(())
    }

    fn array(&self, writer: &mut dyn Write, section: &str) -> io::Result<()> {
        writer.write_all(b"[")?;
        let mut first = true;
        self.visit(section, |bytes| {
            separator(writer, &mut first)?;
            writer.write_all(bytes)
        })?;
        writer.write_all(b"]")
    }

    fn row(&self, writer: &mut dyn Write, section: &str, member: usize, ordinal: usize) -> io::Result<()> {
        let member = i64::try_from(member).map_err(io::Error::other)?;
        let ordinal = i64::try_from(ordinal).map_err(io::Error::other)?;
        self.connection
            .prepare_cached("SELECT payload FROM rows WHERE section=?1 AND member=?2 AND ordinal=?3")
            .map_err(io::Error::other)?
            .query_row(rusqlite::params![section, member, ordinal], |row| {
                let bytes = row
                    .get_ref(0)?
                    .as_blob()
                    .map_err(|error| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Blob, Box::new(error)))?;
                Ok(writer.write_all(bytes))
            })
            .map_err(io::Error::other)?
    }
}

impl fmt::Display for DocumentWarningText<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.0, self.1)
    }
}

impl Serialize for DocumentWarningText<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl Serialize for Report<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_map(Some(12))?;
        fields.serialize_entry("appearance", self.appearance)?;
        fields.serialize_entry("report_defaults", self.report_defaults)?;
        fields.serialize_entry("case_runs", &Rows(self, Section::Cases))?;
        fields.serialize_entry("documents", &self.header.documents)?;
        fields.serialize_entry("generated_at", &self.header.generated_at)?;
        fields.serialize_entry(
            "hub",
            &Hub {
                generator: "dokime-runtime",
                name: "Dokime",
                version: env!("CARGO_PKG_VERSION"),
            },
        )?;
        fields.serialize_entry("plots", &self.header.plots.unwrap_or(&BTreeMap::new()))?;
        fields.serialize_entry("schema_version", &1u8)?;
        fields.serialize_entry("snapshot", &SnapshotRows(self))?;
        fields.serialize_entry("test_cycle_plan", &PlanRows(self))?;
        fields.serialize_entry("test_executions", &Rows(self, Section::Executions))?;
        fields.serialize_entry("warnings", &Rows(self, Section::Warnings))?;
        fields.end()
    }
}

impl Serialize for Rows<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let report = self.0;
        let mut rows = serializer.serialize_seq(None)?;
        if matches!(self.1, Section::Warnings) {
            // The Header Has No Joined Executions; Only Truly Empty Cycles Get This Warning.
            if report.index.members == 0 {
                rows.serialize_element(
                    "report.no_executions: No executions are available yet; this report reflects the current cycle plan.",
                )?;
            }
            if report.header.cycle.result.is_none() {
                rows.serialize_element("report.no_result_summary: No result summary has been produced for this cycle.")?;
            }
        }
        if matches!(self.1, Section::Templates | Section::PlanCases | Section::Queue) {
            for (ordinal, case) in report.index.cases.iter().enumerate() {
                if matches!(self.1, Section::Templates) && !case.first_template {
                    continue;
                }
                match self.1 {
                    Section::Templates => rows.serialize_element(&plan::Template {
                        template_id: case.template.as_str(),
                        name: &case.metadata.label,
                        description: None,
                        source_file: None,
                        parameters: plan::Parameters(&case.definitions),
                        files: &[],
                        steps: &[],
                        measurements: &[],
                        criteria: &[],
                        initialize_hooks: &[],
                        finalize_hooks: &[],
                        interrupt_hooks: &[],
                        cycle_summary: None,
                        metadata: BTreeMap::new(),
                    })?,
                    Section::PlanCases => rows.serialize_element(&plan::Case {
                        case_id: case.id.as_str(),
                        template_id: case.template.as_str(),
                        parameters: ReportValues(&case.parameters),
                        generation_index: ordinal,
                        label: &case.metadata.label,
                        metadata: &case.metadata,
                        latest_test_execution_id: Some(case.latest_id.as_str()),
                        status: member_status(&case.status, case.planned),
                    })?,
                    Section::Queue => rows.serialize_element(&plan::QueueItem {
                        queue_item_id: case.id.as_str(),
                        test_cycle_id: report.header.cycle.test_cycle_id.as_str(),
                        ordinal,
                        template_id: case.template.as_str(),
                        case_id: case.id.as_str(),
                        repeat_index: 0,
                        status: if case.planned { "queued" } else { queue_status(&case.status) },
                        current_test_execution_id: Some(case.latest_id.as_str()),
                        attempt_count: case.count,
                        blocked_reason: None,
                    })?,
                    _ => unreachable!(),
                }
            }
            return rows.end();
        }
        if matches!(self.1, Section::Executions) {
            for (ordinal, member) in report.index.executions.iter().enumerate() {
                rows.serialize_element(&ExecutionRow {
                    test_execution_id: member.id.as_str(),
                    test_cycle_id: report.header.cycle.test_cycle_id.as_str(),
                    queue_item_id: member.case.as_str(),
                    case_id: member.case.as_str(),
                    template_id: member.template.as_str(),
                    attempt_index: ordinal,
                    status: member_status(&member.status, member.planned),
                    started_at: member.started_at.as_ref(),
                    finished_at: member.finished_at.as_ref(),
                    parameters: ReportValues(&member.parameters),
                    metadata: ExecutionMetadata {
                        result: member.result.as_ref(),
                        parameters: ReportValues(&member.parameters),
                    },
                    steps: Steps(report, member),
                })?;
            }
            return rows.end();
        }
        for member in 0..report.index.members {
            let counts = report.index.counts[member];
            if matches!(self.1, Section::Artifacts) && counts[0] == 0
                || matches!(self.1, Section::Records) && counts[1] == 0
                || matches!(self.1, Section::Warnings) && counts[2] == 0
            {
                continue;
            }
            let captured = (report.read)(member).map_err(serde::ser::Error::custom)?;
            let input = &*captured;
            match self.1 {
                Section::Cases => {
                    let execution = &input.cycle.test_executions[0];
                    let row = case_session(input, execution.snapshot.case());
                    rows.serialize_element(&row)?;
                }
                Section::Artifacts => {
                    for link in &input.artifact_links {
                        rows.serialize_element(&ArtifactRow { link, download_href: &link.href })?;
                    }
                }
                Section::Records => {
                    for record in &input.records {
                        rows.serialize_element(&record_row(report.header, &record))?;
                    }
                }
                Section::Warnings => {
                    for (code, message) in crate::report::document::document_warnings(input) {
                        rows.serialize_element(&DocumentWarningText(code, message))?;
                    }
                }
                _ => unreachable!(),
            }
        }
        rows.end()
    }
}

impl Serialize for SnapshotRows<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let input = self.0.header;
        let snapshot = snapshot(input);
        let mut fields = serializer.serialize_map(Some(12))?;
        fields.serialize_entry("run", &snapshot.run)?;
        fields.serialize_entry("session_id", &snapshot.session_id)?;
        fields.serialize_entry("title", &snapshot.title)?;
        fields.serialize_entry("revision", &snapshot.revision)?;
        fields.serialize_entry("records", &Rows(self.0, Section::Records))?;
        fields.serialize_entry("telemetry", &Telemetry(self.0))?;
        fields.serialize_entry("pending_prompts", &snapshot.pending_prompts)?;
        fields.serialize_entry("notes", &Annotations(self.0, AnnotationKind::Note))?;
        fields.serialize_entry("health", &Annotations(self.0, AnnotationKind::Health))?;
        fields.serialize_entry("result_summary", &snapshot.result_summary)?;
        fields.serialize_entry("traceability", &snapshot.traceability)?;
        fields.serialize_entry("artifacts", &Rows(self.0, Section::Artifacts))?;
        fields.serialize_entry("updated_at", &snapshot.updated_at)?;
        fields.end()
    }
}

impl Serialize for PlanRows<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let report = self.0;
        let input = report.header;
        let cycle = plan::Cycle {
            test_cycle_id: input.cycle.test_cycle_id.as_str(),
            title: &input.cycle.metadata.label,
            suite_id: input.cycle.metadata.identifier.as_str(),
            suite_name: &input.cycle.metadata.label,
            status: cycle_status(input),
            case_count: report.index.cases.len(),
            queue_count: report.index.cases.len(),
            current_test_execution_id: None,
            current_queue_item_id: None,
            started_at: input.cycle.created_at,
            finished_at: is_finished_cycle(input).then_some(input.generated_at),
            duration_s: None,
            duration_delta_s: None,
            metadata: input.cycle.metadata,
            generated_at: input.cycle.created_at,
        };
        let mut fields = serializer.serialize_map(Some(5))?;
        fields.serialize_entry("test_cycle", &cycle)?;
        fields.serialize_entry("templates", &Rows(report, Section::Templates))?;
        fields.serialize_entry("cases", &Rows(report, Section::PlanCases))?;
        fields.serialize_entry("queue", &Rows(report, Section::Queue))?;
        fields.serialize_entry("generated_at", input.cycle.created_at)?;
        fields.end()
    }
}

impl Serialize for Telemetry<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_map(None)?;
        let mut remaining = self.0.index.samples.as_slice();
        while let Some(first) = remaining.first() {
            let count = remaining.iter().take_while(|sample| sample.stream == first.stream).count();
            let (samples, rest) = remaining.split_at(count);
            remaining = rest;
            let returned = selection(count, self.0.report_defaults.plot_points as usize).count();
            #[derive(Serialize)]
            struct Stream<'a, 'r> {
                stream: &'a str,
                spec: Option<()>,
                summary: BTreeMap<(), ()>,
                samples: Samples<'a, 'r>,
                max_samples: usize,
                total_samples: usize,
                dropped_samples: usize,
                downsampled: bool,
                updated_at: &'a Timestamp,
            }
            fields.serialize_entry(
                first.stream.as_str(),
                &Stream {
                    stream: first.stream.as_str(),
                    spec: None,
                    summary: BTreeMap::new(),
                    samples: Samples(self.0, samples),
                    max_samples: self.0.report_defaults.plot_points as usize,
                    total_samples: count,
                    dropped_samples: count - returned,
                    downsampled: returned < count,
                    updated_at: &samples.last().unwrap().timestamp,
                },
            )?;
        }
        fields.end()
    }
}

impl Serialize for Steps<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let Self(report, member) = *self;
        let owner = || StepOwner {
            id: member.id.as_str(),
            case: member.case.as_str(),
            template: member.template.as_str(),
            scope: &member.scope,
            generated_at: &member.generated_at,
        };
        let mut rows = serializer.serialize_seq(None)?;
        for (order, step) in member.steps.iter().enumerate() {
            rows.serialize_element(&step_with_owner(report.header, owner(), step, order, None))?;
            // Criteria Order Within Their Step, As The Live Execution Numbers Them.
            for (position, criterion) in step.criteria.iter().enumerate() {
                let order = order.saturating_mul(100).saturating_add(position);
                rows.serialize_element(&step_with_owner(report.header, owner(), step, order, Some(criterion)))?;
            }
        }
        rows.end()
    }
}

impl Serialize for Annotations<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let Self(report, kind) = *self;
        let mut rows = serializer.serialize_seq(None)?;
        // Consecutive Annotations From One Member Share One Loaded Capture.
        let mut loaded = None;
        for annotation in report.index.annotations.iter().filter(|annotation| annotation.kind == kind) {
            if loaded.as_ref().is_none_or(|(member, _)| *member != annotation.member) {
                drop(loaded.take());
                loaded = Some((annotation.member, (report.read)(annotation.member).map_err(serde::ser::Error::custom)?));
            }
            let crate::report::html::Member::Captured(captured) = &loaded.as_ref().unwrap().1;
            let context = &captured.context;
            match kind {
                AnnotationKind::Note => {
                    let note = &context.notes[annotation.position];
                    rows.serialize_element(&NoteRow {
                        note_id: note.note_id.as_str(),
                        author: note.author.as_deref(),
                        text: &note.message,
                        timestamp: &note.created_at,
                        scope: &note.scope,
                    })?;
                }
                AnnotationKind::Health => {
                    let item = &context.health[annotation.position];
                    rows.serialize_element(&HealthRow {
                        health_id: item.health_item_id.as_str(),
                        level: &item.severity,
                        title: &item.message,
                        message: item.detail.as_deref(),
                        details: BTreeMap::new(),
                        created_at: &item.observed_at,
                        scope: &item.scope,
                    })?;
                }
            }
        }
        rows.end()
    }
}

impl Serialize for Samples<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let returned = selection(self.1.len(), self.0.report_defaults.plot_points as usize).count();
        let mut rows = serializer.serialize_seq(Some(returned))?;
        // Consecutive Samples From One Member Share One Original Decoded Loan.
        let mut loaded = None;
        for location in selected_samples(self.1, self.0.report_defaults.plot_points as usize) {
            if loaded.as_ref().is_none_or(|(member, _)| *member != location.member) {
                drop(loaded.take());
                loaded = Some((location.member, (self.0.read)(location.member).map_err(serde::ser::Error::custom)?));
            }
            let sample = loaded
                .as_ref()
                .unwrap()
                .1
                .telemetry
                .get(location.sample)
                .ok_or_else(|| serde::ser::Error::custom("report sample location is missing"))?;
            rows.serialize_element(&telemetry::SampleValues(sample))?;
        }
        rows.end()
    }
}

/* === Public Functions === */

pub(crate) fn selection(length: usize, maximum: usize) -> impl Iterator<Item = usize> + Clone {
    let count = if length <= maximum { length } else { maximum.saturating_sub(1) };
    let indices = harmos::signal::decimate_indices(length, count);
    let endpoint = length.checked_sub(1).filter(|last| indices.clone().next_back() != Some(*last));
    indices.chain(endpoint)
}

/* === Private Functions === */

fn separator(writer: &mut dyn Write, first: &mut bool) -> io::Result<()> {
    if !*first {
        writer.write_all(b",")?;
    }
    *first = false;
    Ok(())
}

fn rows_error(error: rusqlite::Error) -> Error {
    Error::InvalidReportInputs(error.to_string())
}

// An Undispatched Attempt Reads As The Live Cycle Spells It: A Skipped One Is Planned.
fn member_status(status: &ExecutionStatus, planned: bool) -> &'static str {
    if planned && *status == ExecutionStatus::Skipped { "planned" } else { execution_status(status) }
}

fn selected_samples(samples: &[crate::report::cycle::Sample], maximum: usize) -> impl Iterator<Item = &crate::report::cycle::Sample> {
    selection(samples.len(), maximum).map(move |index| &samples[index])
}

/* === Tests === */

#[test]
fn preview_selection_preserves_order_and_terminal_endpoint() {
    for (length, maximum, expected) in [
        (0, 0, vec![]),
        (1, 0, vec![0]),
        (5, 0, vec![4]),
        (5, 1, vec![4]),
        (5, 5, vec![0, 1, 2, 3, 4]),
        (8, 4, vec![0, 3, 6, 7]),
        (9, 4, vec![0, 3, 6, 8]),
        (10, 4, vec![0, 4, 8, 9]),
    ] {
        let indices = selection(length, maximum);
        assert_eq!(indices.clone().count(), expected.len());
        assert_eq!(indices.collect::<Vec<_>>(), expected);
    }
}

#[test]
fn fixture_report_save_measurement() {
    use crate::report::{CapturedExecutionReportInput, cycle::CycleMember};
    use std::{
        io::{BufWriter, Read},
        sync::atomic::Ordering,
        time::Instant,
    };
    let captured: CapturedExecutionReportInput = serde_json::from_str(include_str!("../fixtures/execution-report-input.json")).unwrap();
    let input = harmos::storage::ReadBatch::new(captured);
    let CycleMember::Captured(member) = CycleMember::capture(input.clone()).unwrap() else { unreachable!() };
    let mut index = Index::new();
    index.push_capture(&input).unwrap();
    index.sort();
    let read = |_| member.read().map(crate::report::html::Member::Captured);
    let report = Report {
        header: (&input.input).into(),
        index: &index,
        appearance: &input.context.appearance,
        report_defaults: &input.report_defaults,
        read: &read,
    };
    let mut baseline_file = None;
    let started = Instant::now();
    for _ in 0..20 {
        let mut writer = BufWriter::with_capacity(1024 * 1024, tempfile::tempfile().unwrap());
        drop(member.read().unwrap());
        report.write(&mut writer).unwrap();
        writer.flush().unwrap();
        baseline_file = Some(writer.into_inner().unwrap());
    }
    let before = started.elapsed() / 20;
    let mut baseline_bytes = Vec::new();
    let mut baseline_file = baseline_file.unwrap();
    std::io::Seek::rewind(&mut baseline_file).unwrap();
    baseline_file.read_to_end(&mut baseline_bytes).unwrap();
    let baseline_decodes = member.decodes.load(Ordering::Relaxed);
    let mut candidate_file = None;
    let started = Instant::now();
    for _ in 0..20 {
        let mut writer = BufWriter::with_capacity(1024 * 1024, tempfile::tempfile().unwrap());
        let captured = member.read().unwrap();
        let rows = MemberRows::new().unwrap();
        rows.capture(0, &captured, report.header).unwrap();
        drop(captured);
        report.write_prepared(&mut writer, &rows).unwrap();
        writer.flush().unwrap();
        candidate_file = Some(writer.into_inner().unwrap());
    }
    let after = started.elapsed() / 20;
    let mut candidate_bytes = Vec::new();
    let mut candidate_file = candidate_file.unwrap();
    std::io::Seek::rewind(&mut candidate_file).unwrap();
    candidate_file.read_to_end(&mut candidate_bytes).unwrap();
    assert_eq!(member.decodes.load(Ordering::Relaxed) - baseline_decodes, 20);
    assert!(baseline_decodes > 20);
    assert_eq!(candidate_bytes, baseline_bytes);
    eprintln!("Fixture Report Save: before={before:?}, after={after:?}, bytes={}", candidate_bytes.len());
}
