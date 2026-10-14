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
use std::{collections::BTreeMap, io::Write, num::NonZeroUsize};
// Third Party Imports
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};
// Local Imports
use super::*;
use crate::report::cycle::Index;
use dokime::storage::Manager;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type ReadMember<'a> = dyn Fn(usize) -> Result<crate::report::html::Member> + 'a;

/* === Structs === */
pub(in crate::report) struct Report<'a> {
    pub header: Header<'a>,
    pub index: &'a Index,
    pub appearance: &'a crate::installed_plugins::RuntimeAppearanceFrame,
    pub read: &'a ReadMember<'a>,
    pub manager: &'a Manager,
    pub computation: &'a dyn Fn() -> Result<crate::report::html::Computation>,
}
struct Rows<'a, 'r>(&'a Report<'r>, Section);
struct DocumentWarningText<'a>(&'a str, &'a str);
struct SnapshotRows<'a, 'r>(&'a Report<'r>);
struct PlanRows<'a, 'r>(&'a Report<'r>);
struct Telemetry<'a, 'r>(&'a Report<'r>);
struct Samples<'a, 'r>(&'a Report<'r>, &'a [crate::report::cycle::Sample]);

/* === Enums === */
#[derive(Clone, Copy)]
enum Section {
    Artifacts,
    Notes,
    Health,
    Cases,
    Records,
    Executions,
    Steps,
    Warnings,
    Templates,
    PlanCases,
    Queue,
}

/* === Traits === */

/* === Implementations === */
impl Report<'_> {
    pub(crate) fn write(&self, writer: &mut dyn Write) -> Result<()> {
        output::write_html(writer, self, REPORT_APP_TEMPLATE)
            .map_err(|source| Error::InvalidReportInputs(source.to_string()))
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
        let mut fields = serializer.serialize_map(Some(15))?;
        fields.serialize_entry("appearance", self.appearance)?;
        fields.serialize_entry("artifact_links", &Rows(self, Section::Artifacts))?;
        fields.serialize_entry("box_summaries", &BTreeMap::<String, JsonValue>::new())?;
        fields.serialize_entry("case_runs", &Rows(self, Section::Cases))?;
        fields.serialize_entry("documents", &self.header.documents)?;
        fields.serialize_entry("generated_at", &self.header.generated_at)?;
        fields.serialize_entry("histograms", &BTreeMap::<String, JsonValue>::new())?;
        fields.serialize_entry(
            "hub",
            &Hub {
                generator: "dokime-app",
                name: "Dokime",
                version: env!("CARGO_PKG_VERSION"),
            },
        )?;
        fields.serialize_entry("plots", &self.header.plots.unwrap_or(&BTreeMap::new()))?;
        fields.serialize_entry("schema_version", &1u8)?;
        fields.serialize_entry("snapshot", &SnapshotRows(self))?;
        fields.serialize_entry("test_cycle_plan", &PlanRows(self))?;
        fields.serialize_entry("test_executions", &Rows(self, Section::Executions))?;
        fields.serialize_entry("test_steps", &Rows(self, Section::Steps))?;
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
                rows.serialize_element("report.no_executions: No executions are available yet; this report reflects the current cycle plan.")?;
            }
            if report.header.cycle.result.is_none() {
                rows.serialize_element(
                    "report.no_result_summary: No result summary has been produced for this cycle.",
                )?;
            }
        }
        if matches!(
            self.1,
            Section::Templates | Section::PlanCases | Section::Queue
        ) {
            for (ordinal, case) in report.index.cases.iter().enumerate() {
                if matches!(self.1, Section::Templates) && !case.first_template {
                    continue;
                }
                let computation = (report.computation)().map_err(serde::ser::Error::custom)?;
                match self.1 {
                    Section::Templates => rows.serialize_element(&plan::Template {
                        template_id: case.template.as_str(),
                        name: &case.metadata.label,
                        description: None,
                        source_file: None,
                        parameters: &[],
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
                        status: execution_status(&case.status),
                    })?,
                    Section::Queue => rows.serialize_element(&plan::QueueItem {
                        queue_item_id: case.id.as_str(),
                        test_cycle_id: report.header.cycle.test_cycle_id.as_str(),
                        ordinal,
                        template_id: case.template.as_str(),
                        case_id: case.id.as_str(),
                        repeat_index: 0,
                        status: queue_status(&case.status),
                        current_test_execution_id: Some(case.latest_id.as_str()),
                        attempt_count: case.count,
                        blocked_reason: None,
                    })?,
                    _ => unreachable!(),
                }
                computation.check().map_err(serde::ser::Error::custom)?;
            }
            return rows.end();
        }
        if matches!(self.1, Section::Executions | Section::Steps) {
            let computation = (report.computation)().map_err(serde::ser::Error::custom)?;
            for (ordinal, member) in report.index.executions.iter().enumerate() {
                if matches!(self.1, Section::Executions) {
                    rows.serialize_element(&ExecutionRow {
                        test_execution_id: member.id.as_str(),
                        test_cycle_id: report.header.cycle.test_cycle_id.as_str(),
                        queue_item_id: member.case.as_str(),
                        case_id: member.case.as_str(),
                        template_id: member.template.as_str(),
                        attempt_index: ordinal,
                        status: execution_status(&member.status),
                        started_at: member.started_at.as_ref(),
                        finished_at: member.finished_at.as_ref(),
                        parameters: ReportValues(&member.parameters),
                        metadata: ExecutionMetadata {
                            result: member.result.as_ref(),
                            parameters: ReportValues(&member.parameters),
                        },
                    })?;
                } else {
                    for (order, step) in member.steps.iter().enumerate() {
                        let owner = || StepOwner {
                            id: member.id.as_str(),
                            case: member.case.as_str(),
                            template: member.template.as_str(),
                            scope: &member.scope,
                        };
                        rows.serialize_element(&step_with_owner(
                            report.header,
                            owner(),
                            step,
                            order,
                            None,
                        ))?;
                        for (order, criterion) in step.criteria.iter().enumerate() {
                            rows.serialize_element(&step_with_owner(
                                report.header,
                                owner(),
                                step,
                                order,
                                Some(criterion),
                            ))?;
                        }
                    }
                }
            }
            computation.check().map_err(serde::ser::Error::custom)?;
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
            let computation = (report.computation)().map_err(serde::ser::Error::custom)?;
            let input = &*captured;
            match self.1 {
                Section::Cases => {
                    let execution = &input.cycle.test_executions[0];
                    let _scratch = report
                        .manager
                        .prepare_production(NonZeroUsize::new(rendering_memory(input)).unwrap())
                        .map_err(serde::ser::Error::custom)?;
                    let row = case_session(input, execution.snapshot.case());
                    rows.serialize_element(&row)?;
                }
                Section::Notes | Section::Health => match &captured {
                    crate::report::html::Member::Captured(captured) => match self.1 {
                        Section::Notes => {
                            for note in &captured.context.notes {
                                rows.serialize_element(&NoteRow {
                                    note_id: note.note_id.as_str(),
                                    text: &note.message,
                                    timestamp: &note.created_at,
                                    scope: &note.scope,
                                })?;
                            }
                        }
                        Section::Health => {
                            for item in &captured.context.health {
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
                        _ => unreachable!(),
                    },
                },
                Section::Artifacts => {
                    for link in &input.artifact_links {
                        rows.serialize_element(link)?;
                    }
                }
                Section::Records => {
                    for record in &input.records {
                        rows.serialize_element(&record_row(report.header, record))?;
                    }
                }
                Section::Warnings => {
                    for (code, message) in crate::report::document::document_warnings(input) {
                        rows.serialize_element(&DocumentWarningText(code, message))?;
                    }
                }
                _ => unreachable!(),
            }
            computation.check().map_err(serde::ser::Error::custom)?;
        }
        rows.end()
    }
}
impl Serialize for SnapshotRows<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let input = self.0.header;
        let snapshot = snapshot(input);
        let mut fields = serializer.serialize_map(Some(13))?;
        fields.serialize_entry("run", &snapshot.run)?;
        fields.serialize_entry("session_id", &snapshot.session_id)?;
        fields.serialize_entry("title", &snapshot.title)?;
        fields.serialize_entry("revision", &snapshot.revision)?;
        fields.serialize_entry("records", &Rows(self.0, Section::Records))?;
        fields.serialize_entry("telemetry", &Telemetry(self.0))?;
        fields.serialize_entry("pending_prompts", &snapshot.pending_prompts)?;
        fields.serialize_entry("notes", &Rows(self.0, Section::Notes))?;
        fields.serialize_entry("health", &Rows(self.0, Section::Health))?;
        fields.serialize_entry("result_summary", &snapshot.result_summary)?;
        fields.serialize_entry("traceability", &snapshot.traceability)?;
        fields.serialize_entry("artifacts", &snapshot.artifacts)?;
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
            continue_on_failure: input.cycle.continue_on_failure,
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
            let count = remaining
                .iter()
                .take_while(|sample| sample.stream == first.stream)
                .count();
            let (samples, rest) = remaining.split_at(count);
            remaining = rest;
            let returned = selection(count).len();
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
                    max_samples: dokime::document::DEFAULT_PLOT_MAX_POINTS,
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
impl Serialize for Samples<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let returned = selection(self.1.len()).len();
        let mut rows = serializer.serialize_seq(Some(returned))?;
        // Consecutive Samples From One Member Share One Original Decoded Loan.
        let mut loaded = None;
        for location in selected_samples(self.1) {
            if loaded
                .as_ref()
                .is_none_or(|(member, _)| *member != location.member)
            {
                drop(loaded.take());
                loaded = Some((
                    location.member,
                    (self.0.read)(location.member).map_err(serde::ser::Error::custom)?,
                ));
            }
            let computation = (self.0.computation)().map_err(serde::ser::Error::custom)?;
            let sample = &loaded.as_ref().unwrap().1.telemetry[location.sample];
            rows.serialize_element(&telemetry::SampleValues(sample))?;
            computation.check().map_err(serde::ser::Error::custom)?;
        }
        rows.end()
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn selected_samples(
    samples: &[crate::report::cycle::Sample],
) -> impl ExactSizeIterator<Item = &crate::report::cycle::Sample> {
    selection(samples.len())
        .into_iter()
        .map(move |index| &samples[index])
}

pub(crate) fn selection(length: usize) -> Vec<usize> {
    let mut indices = if length <= dokime::document::DEFAULT_PLOT_MAX_POINTS {
        (0..length).collect::<Vec<_>>()
    } else {
        harmos::signal::decimate(length, 1_999)
    };
    if let Some(last) = length.checked_sub(1)
        && indices.last() != Some(&last)
    {
        indices.push(last);
    }
    indices
}

/* === Tests === */
