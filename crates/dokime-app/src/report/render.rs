/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod cycle;
pub(super) mod header;
mod plan;
mod telemetry;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt,
};
// Third Party Imports
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};
use serde_json::Value as JsonValue;
// Local Imports
use super::output;
use crate::definition::TestCase;
use crate::document::DocumentDefinition;
use crate::document::capture::Capture;
use crate::execution::{
    CriterionResult, CriterionStatus, ExecutionStatus, ResultStatus, ResultSummary, StepRecord,
    StepStatus, TestExecution,
};
use crate::report::{
    asset::REPORT_APP_TEMPLATE,
    input::{CycleReportInput, ReportArtifactLink, ReportCaseDocument},
};
use crate::traceability::{TraceabilityEntry, TraceabilitySnapshot, derived_traceability_group};
use crate::{Error, Result};
use dokime::core::{CaseId, DocumentId, Metadata, RecordScope, Timestamp, Value};
use header::Header;

/* === Constants === */
const DEFAULT_DOCUMENT_ID: &str = "default";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct CaseRun<'a> {
    test_case: &'a TestCase,
    execution: Option<&'a TestExecution>,
    document: Option<&'a ReportCaseDocument>,
    traceability: Traceability<'a>,
}

#[derive(Serialize)]
struct ExecutionRow<'a, S> {
    test_execution_id: &'a str,
    test_cycle_id: &'a str,
    queue_item_id: &'a str,
    case_id: &'a str,
    template_id: &'a str,
    attempt_index: usize,
    status: &'static str,
    started_at: Option<&'a Timestamp>,
    finished_at: Option<&'a Timestamp>,
    parameters: ReportValues<'a>,
    metadata: ExecutionMetadata<'a>,
    /// The execution's own step and criterion rows, in recorded order.
    steps: S,
}

/// One reachable artifact: its link already carries the record's description
/// and grouping, so the row is complete once the download target is named.
#[derive(Serialize)]
struct ArtifactRow<'a> {
    #[serde(flatten)]
    link: &'a ReportArtifactLink,
    #[serde(rename = "downloadHref")]
    download_href: &'a str,
}

#[derive(Serialize)]
struct ExecutionMetadata<'a> {
    result: Option<&'a ResultSummary>,
    parameters: ReportValues<'a>,
}

#[derive(Serialize)]
struct NoteRow<'a> {
    note_id: &'a str,
    author: Option<&'a str>,
    text: &'a str,
    timestamp: &'a Timestamp,
    scope: &'a RecordScope,
}

#[derive(Serialize)]
struct HealthRow<'a> {
    health_id: &'a str,
    level: &'a dokime::execution::HealthSeverity,
    title: &'a str,
    message: Option<&'a str>,
    details: BTreeMap<String, JsonValue>,
    created_at: &'a Timestamp,
    scope: &'a RecordScope,
}

struct Snapshot<'a> {
    run: RunRow<'a>,
    session_id: &'a str,
    title: ReportTitle<'a>,
    revision: u8,
    pending_prompts: &'static [()],
    result_summary: Option<Summary<'a>>,
    traceability: Option<()>,
    updated_at: &'a Timestamp,
}

#[derive(Serialize)]
struct RunRow<'a> {
    session_id: &'a str,
    title: &'a str,
    template_id: &'a str,
    template_slug: &'a str,
    label: &'a str,
    metadata: &'a Metadata,
    status: &'static str,
    started_at: &'a Timestamp,
    updated_at: &'a Timestamp,
    run_dir: Option<()>,
    local_url: Option<()>,
    lan_url: Option<()>,
    rerun_command: Option<()>,
}

struct ReportValues<'a>(&'a BTreeMap<String, Value>);

#[derive(Serialize)]
struct RecordRow<'a> {
    record_id: &'a str,
    record_type: &'a str,
    timestamp: &'a Timestamp,
    session_id: &'a str,
    sequence: u64,
    level: &'static str,
    source: &'a str,
    display: RecordDisplay,
    message: Option<&'a str>,
    error: Option<&'a dokime::records::RecordErrorDetail>,
    details: ReportValues<'a>,
    related_sample_id: Option<&'a str>,
    related_stream: Option<&'a str>,
    related_artifact_id: Option<&'a str>,
    scope: Option<ReportScope<'a>>,
}

#[derive(Serialize)]
struct RecordDisplay {
    icon: &'static str,
    tone: &'static str,
}

#[derive(Serialize)]
struct ReportScope<'a> {
    test_cycle_id: &'a str,
    test_execution_id: Option<&'a str>,
    test_case_id: Option<&'a str>,
    template_id: Option<&'a str>,
    step_id: Option<&'a str>,
    criterion_id: Option<&'a str>,
}

struct StepFailure<'a>(&'a StepRecord);

#[derive(Serialize)]
struct StepRow<'a> {
    record_type: &'static str,
    test_execution_id: &'a str,
    test_cycle_id: &'a str,
    queue_item_id: &'a str,
    case_id: &'a str,
    template_id: &'a str,
    step_id: &'a str,
    method_name: &'a str,
    order: usize,
    status: &'static str,
    result: StepResult<'a>,
    started_at: &'a Timestamp,
    finished_at: &'a Timestamp,
    duration_s: f64,
    metadata: &'a Metadata,
    scope: StepScope<'a>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum StepResult<'a> {
    Step {
        status: &'static str,
        details: StepFailure<'a>,
    },
    Criterion {
        status: &'static str,
        reason: &'a Option<String>,
        details: CriterionDetails<'a>,
    },
}

#[derive(Serialize)]
struct CriterionDetails<'a> {
    observed: Option<ReportValue<'a>>,
}

#[derive(Serialize)]
struct StepScope<'a> {
    test_cycle_id: &'a str,
    test_execution_id: Option<&'a str>,
    test_case_id: Option<&'a str>,
    template_id: &'a str,
    step_id: &'a str,
    criterion_id: Option<&'a str>,
}

struct StepOwner<'a> {
    id: &'a str,
    case: &'a str,
    template: &'a str,
    scope: &'a RecordScope,
}
struct ReportValue<'a>(&'a Value);

#[derive(Serialize)]
struct Hub {
    generator: &'static str,
    name: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct Summary<'a> {
    outcome: &'static str,
    headline: &'static str,
    warnings: &'static [()],
    evidence: [SummaryEvidence; 1],
    updated_at: &'a Timestamp,
    scope: ReportScope<'a>,
}

#[derive(Serialize)]
struct SummaryEvidence {
    passed: u32,
    failed: u32,
    blocked: u32,
    skipped: u32,
}

struct Traceability<'a> {
    input: &'a CycleReportInput,
    scope: Option<RecordScope>,
}

struct TraceabilityGroups<'a>(BTreeMap<(TraceabilityGroupRank, &'a str), TraceabilityGroup<'a>>);

struct TraceabilityGroup<'a> {
    group_id: TraceabilityKey<'a>,
    title: &'a str,
    sources: BTreeSet<&'a str>,
    items: BTreeMap<TraceabilityKey<'a>, TraceabilityItem<'a>>,
    updated_at: &'a Timestamp,
}

#[derive(Serialize)]
struct TraceabilityItem<'a> {
    label: &'a str,
    value: ReportValue<'a>,
    unit: Option<&'a str>,
    level: &'static str,
    details: TraceabilityDetails<'a>,
    scope: ReportScope<'a>,
}

#[derive(Serialize)]
struct TraceabilityDetails<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<&'a str>,
    identifier: &'a str,
}

struct TraceabilityItems<'g, 'a>(&'g BTreeMap<TraceabilityKey<'a>, TraceabilityItem<'a>>);
struct TraceabilitySources<'g, 'a>(&'g BTreeSet<&'a str>);

struct ReportTitle<'a> {
    label: &'a str,
    cycle: bool,
}

struct TraceabilityKey<'a> {
    identifier: &'a str,
    label: &'a str,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TraceabilityGroupRank {
    Derived,
    Stated,
}

/* === Traits === */

/* === Implementations === */

impl Serialize for ReportValues<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            fields.serialize_entry(key, &ReportValue(value))?;
        }
        fields.end()
    }
}

impl Serialize for StepFailure<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut fields =
            serializer.serialize_map(Some(if self.0.detail.is_some() { 3 } else { 0 }))?;
        if let Some(detail) = &self.0.detail {
            fields.serialize_entry("message", &detail.message)?;
            fields.serialize_entry("causes", &detail.chain)?;
            fields.serialize_entry("location", &detail.location)?;
        }
        fields.end()
    }
}

impl Serialize for CaseRun<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_map(Some(6))?;
        fields.serialize_entry("queue_item_id", &self.test_case.test_case_id)?;
        fields.serialize_entry("case_id", &self.test_case.test_case_id)?;
        fields.serialize_entry(
            "test_execution_id",
            &self.execution.map(|execution| &execution.test_execution_id),
        )?;
        fields.serialize_entry("traceability", &self.traceability)?;
        match self.document {
            Some(document) => {
                fields.serialize_entry("documents", &document.documents)?;
                fields.serialize_entry("plots", &document.plots)?;
            }
            None => {
                fields.serialize_entry("documents", &BTreeMap::<DocumentId, Capture>::new())?;
                fields.serialize_entry("plots", &BTreeMap::<String, JsonValue>::new())?;
            }
        }
        fields.end()
    }
}

impl Serialize for ReportValue<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self.0 {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(value) => serializer.serialize_bool(*value),
            Value::Integer(value) if value.unsigned_abs() > 9_007_199_254_740_991 => {
                serializer.collect_str(value)
            }
            Value::Integer(value) => serializer.serialize_i64(*value),
            Value::Unsigned(value) if *value > 9_007_199_254_740_991 => {
                serializer.collect_str(value)
            }
            Value::Unsigned(value) => serializer.serialize_u64(*value),
            Value::Number(value) if !value.is_finite() => serializer.collect_str(value),
            Value::Number(value) => serializer.serialize_f64(*value),
            Value::Text(value) => serializer.serialize_str(value),
            Value::Datetime(value) => {
                // UTC RFC3339 Uses At Most Thirty ASCII Bytes Including Nanoseconds.
                let mut bytes = [0u8; 30];
                let mut output = std::io::Cursor::new(bytes.as_mut_slice());
                match value
                    .to_offset(time::UtcOffset::UTC)
                    .format_into(&mut output, &time::format_description::well_known::Rfc3339)
                {
                    Ok(_) => {
                        let length = output.position() as usize;
                        let text = std::str::from_utf8(&bytes[..length])
                            .map_err(serde::ser::Error::custom)?;
                        serializer.serialize_str(text)
                    }
                    Err(_) => serializer.serialize_unit(),
                }
            }
            Value::Array(values) => {
                let mut sequence = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    sequence.serialize_element(&ReportValue(value))?;
                }
                sequence.end()
            }
            Value::Object(values) => {
                let mut fields = serializer.serialize_map(Some(values.len()))?;
                for (key, value) in values {
                    fields.serialize_entry(key, &ReportValue(value))?;
                }
                fields.end()
            }
        }
    }
}

/* === Public Functions === */
/* === Private Functions === */
fn snapshot(input: Header<'_>) -> Snapshot<'_> {
    let template_id = input.cycle.metadata.identifier.as_str();
    Snapshot {
        run: RunRow {
            session_id: input.session_id.as_str(),
            title: &input.cycle.metadata.label,
            template_id,
            template_slug: template_id,
            label: &input.cycle.metadata.label,
            metadata: input.cycle.metadata,
            status: run_status(input),
            started_at: input.cycle.created_at,
            updated_at: input.generated_at,
            run_dir: None,
            local_url: None,
            lan_url: None,
            rerun_command: None,
        },
        session_id: input.session_id.as_str(),
        title: report_document_title(
            input,
            cycle_default_document_materialization(input).map(|document| &document.definition),
        ),
        revision: 0,
        pending_prompts: &[],
        result_summary: input.cycle.result.map(|result| Summary {
            outcome: match result.status {
                ResultStatus::Passed => "completed",
                ResultStatus::Failed | ResultStatus::Blocked | ResultStatus::Cancelled => "failed",
                ResultStatus::Pending => "unknown",
            },
            headline: result_status(&result.status),
            warnings: &[],
            evidence: [SummaryEvidence {
                passed: result.passed,
                failed: result.failed,
                blocked: result.blocked,
                skipped: result.skipped,
            }],
            updated_at: input.generated_at,
            scope: empty_report_scope(input),
        }),
        // Cycle-Level Traceability Is Read Per Case.
        traceability: None,
        updated_at: input.generated_at,
    }
}

fn report_scope<'a>(input: Header<'a>, scope: &'a RecordScope) -> ReportScope<'a> {
    ReportScope {
        test_cycle_id: scope
            .test_cycle_id
            .as_ref()
            .map_or(input.cycle.test_cycle_id.as_str(), |id| id.as_str()),
        test_execution_id: scope.test_execution_id.as_ref().map(|id| id.as_str()),
        test_case_id: scope.test_case_id.as_ref().map(|id| id.as_str()),
        template_id: scope.template_id.as_ref().map(|id| id.as_str()),
        step_id: scope.step_id.as_ref().map(|id| id.as_str()),
        criterion_id: scope.criterion_id.as_ref().map(|id| id.as_str()),
    }
}

fn record_tone(severity: &str) -> &'static str {
    match severity {
        "error" => "danger",
        "warning" | "warn" => "warning",
        "debug" => "muted",
        _ => "neutral",
    }
}

fn record_severity(severity: crate::records::RecordSeverity) -> &'static str {
    match severity {
        crate::records::RecordSeverity::Debug => "debug",
        crate::records::RecordSeverity::Info => "info",
        crate::records::RecordSeverity::Warning => "warning",
        crate::records::RecordSeverity::Error => "error",
    }
}

fn record_row<'a>(input: Header<'a>, record: &'a crate::records::RecordEntry) -> RecordRow<'a> {
    let severity = record_severity(record.severity);
    RecordRow {
        record_id: record.record_id.as_str(),
        record_type: record.record_type.as_str(),
        timestamp: &record.timestamp,
        session_id: input.session_id.as_str(),
        sequence: record.sequence,
        level: severity,
        source: &record.source,
        display: RecordDisplay {
            icon: "clock",
            tone: record_tone(severity),
        },
        message: record.message.as_deref(),
        error: record.error.as_ref(),
        details: ReportValues(&record.details),
        related_sample_id: record.related_sample_id.as_ref().map(|id| id.as_str()),
        related_stream: record.related_stream_id.as_ref().map(|id| id.as_str()),
        related_artifact_id: record.related_artifact_id.as_ref().map(|id| id.as_str()),
        scope: record
            .scope
            .as_ref()
            .map(|scope| report_scope(input, scope)),
    }
}

fn step_with_owner<'a>(
    input: Header<'a>,
    owner: StepOwner<'a>,
    step: &'a StepRecord,
    order: usize,
    criterion: Option<&'a CriterionResult>,
) -> StepRow<'a> {
    let status = criterion.map_or_else(
        || step_status(&step.status),
        |criterion| criterion_status(&criterion.status),
    );
    let result = criterion.map_or_else(
        || StepResult::Step {
            status,
            details: StepFailure(step),
        },
        |criterion| StepResult::Criterion {
            status,
            reason: &criterion.message,
            details: CriterionDetails {
                observed: criterion.observed.as_ref().map(ReportValue),
            },
        },
    );
    let scope = owner.scope;
    StepRow {
        record_type: if criterion.is_some() {
            "criterion"
        } else {
            "step"
        },
        test_execution_id: owner.id,
        test_cycle_id: input.cycle.test_cycle_id.as_str(),
        queue_item_id: owner.case,
        case_id: owner.case,
        template_id: owner.template,
        step_id: step.step_id.as_str(),
        method_name: criterion
            .map_or(&step.metadata, |criterion| &criterion.metadata)
            .identifier
            .as_str(),
        order,
        status,
        result,
        started_at: step.started_at.as_ref().unwrap_or(input.generated_at),
        finished_at: step.finished_at.as_ref().unwrap_or(input.generated_at),
        duration_s: if criterion.is_some() {
            0.0
        } else {
            duration_seconds(step.started_at.as_ref(), step.finished_at.as_ref())
        },
        metadata: criterion.map_or(&step.metadata, |criterion| &criterion.metadata),
        scope: StepScope {
            test_cycle_id: scope
                .test_cycle_id
                .as_ref()
                .map_or(input.cycle.test_cycle_id.as_str(), |id| id.as_str()),
            test_execution_id: scope.test_execution_id.as_ref().map(|id| id.as_str()),
            test_case_id: scope.test_case_id.as_ref().map(|id| id.as_str()),
            template_id: owner.template,
            step_id: step.step_id.as_str(),
            criterion_id: criterion
                .map(|criterion| criterion.criterion_id.as_str())
                .or_else(|| scope.criterion_id.as_ref().map(|id| id.as_str())),
        },
    }
}

fn case_session<'a>(input: &'a CycleReportInput, test_case: &'a TestCase) -> CaseRun<'a> {
    let execution = latest_execution_for_case(input, &test_case.test_case_id);
    let scope = execution.map_or_else(
        || {
            RecordScope::default()
                .with_cycle(input.cycle.test_cycle_id.clone())
                .with_test_case(test_case.test_case_id.clone())
        },
        |execution| execution.scope.clone(),
    );
    let document = case_document_for_case(input, &test_case.test_case_id, execution);
    let traceability = Traceability {
        input,
        scope: Some(scope),
    };
    CaseRun {
        test_case,
        execution,
        document,
        traceability,
    }
}

fn report_document_title<'a>(
    input: Header<'a>,
    document: Option<&'a DocumentDefinition>,
) -> ReportTitle<'a> {
    match document.filter(|document| document.metadata.identifier.as_str() != "cycle.default") {
        Some(document) => ReportTitle {
            label: &document.metadata.label,
            cycle: false,
        },
        None => ReportTitle {
            label: &input.cycle.metadata.label,
            cycle: true,
        },
    }
}

fn cycle_default_document_materialization(input: Header<'_>) -> Option<&Capture> {
    input
        .documents
        .iter()
        .find_map(|(id, document)| (id.as_str() == DEFAULT_DOCUMENT_ID).then_some(document))
}

fn case_document_for_case<'a>(
    input: &'a CycleReportInput,
    test_case_id: &CaseId,
    execution: Option<&TestExecution>,
) -> Option<&'a ReportCaseDocument> {
    input.case_documents.iter().find(|document| {
        &document.test_case_id == test_case_id
            && match (&document.test_execution_id, execution) {
                (Some(expected), Some(execution)) => expected == &execution.test_execution_id,
                (Some(_), None) => false,
                (None, _) => true,
            }
    })
}

impl Serialize for Traceability<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let groups = traceability_groups(self.input, self.scope.as_ref());
        if groups.0.is_empty() {
            return serializer.serialize_unit();
        }
        let last = self.input.traceability.last();
        let scope = last.map_or_else(
            || empty_report_scope(self.input.into()),
            |item| report_scope(self.input.into(), &item.scope),
        );
        let updated_at = last.map_or(&self.input.generated_at, |item| &item.created_at);
        let mut fields = serializer.serialize_map(Some(5))?;
        fields.serialize_entry("groups", &groups)?;
        fields.serialize_entry("warnings", &[] as &[()])?;
        fields.serialize_entry("artifacts", &[] as &[()])?;
        fields.serialize_entry("updated_at", updated_at)?;
        fields.serialize_entry("scope", &scope)?;
        fields.end()
    }
}

impl fmt::Display for ReportTitle<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label)?;
        if self.cycle {
            formatter.write_str(" Report")?;
        }
        Ok(())
    }
}

impl Serialize for ReportTitle<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl TraceabilityKey<'_> {
    fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.identifier
            .bytes()
            .chain(std::iter::once(b':'))
            .chain(self.label.bytes())
    }
}

impl Ord for TraceabilityKey<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes().cmp(other.bytes())
    }
}

impl PartialOrd for TraceabilityKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TraceabilityKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for TraceabilityKey<'_> {}

impl fmt::Display for TraceabilityKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.identifier)?;
        formatter.write_str(":")?;
        formatter.write_str(self.label)
    }
}

impl Serialize for TraceabilityKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl Serialize for TraceabilityGroups<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut groups = serializer.serialize_seq(Some(self.0.len()))?;
        for group in self.0.values() {
            groups.serialize_element(group)?;
        }
        groups.end()
    }
}

impl Serialize for TraceabilityGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_map(Some(7))?;
        fields.serialize_entry("group_id", &self.group_id)?;
        fields.serialize_entry("title", self.title)?;
        fields.serialize_entry("summary", "")?;
        fields.serialize_entry("level", "info")?;
        fields.serialize_entry("items", &TraceabilityItems(&self.items))?;
        fields.serialize_entry("details", &TraceabilitySources(&self.sources))?;
        fields.serialize_entry("updated_at", self.updated_at)?;
        fields.end()
    }
}

impl Serialize for TraceabilityItems<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut items = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0.values() {
            items.serialize_element(item)?;
        }
        items.end()
    }
}

impl Serialize for TraceabilitySources<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut fields = serializer.serialize_map(Some(usize::from(!self.0.is_empty())))?;
        if !self.0.is_empty() {
            fields.serialize_entry("sources", self.0)?;
        }
        fields.end()
    }
}

fn traceability_groups<'a>(
    input: &'a CycleReportInput,
    scope: Option<&RecordScope>,
) -> TraceabilityGroups<'a> {
    // Rank Separates Derived Groups From Equally Titled Authored Sections.
    let mut groups = BTreeMap::<(TraceabilityGroupRank, &str), TraceabilityGroup<'a>>::new();
    for item in &input.traceability {
        if scope
            .and_then(|scope| scope.test_execution_id.as_ref())
            .is_some()
            && item.scope.test_execution_id.is_none()
        {
            continue;
        }
        if scope.is_some_and(|scope| !dokime::core::scope_visible_in(Some(&item.scope), scope)) {
            continue;
        }
        let rank = traceability_group_rank(item);
        for entry in item
            .entries
            .iter()
            .filter(|entry| entry.source.as_deref() != Some("traceability.environment"))
        {
            let entries = std::slice::from_ref(&entry);
            let title = traceability_group_title(item, entries);
            let group = groups
                .entry((rank, title))
                .or_insert_with(|| TraceabilityGroup {
                    group_id: TraceabilityKey {
                        identifier: item.traceability_id.as_str(),
                        label: title,
                    },
                    title,
                    sources: BTreeSet::new(),
                    items: BTreeMap::new(),
                    updated_at: &item.created_at,
                });
            if let Some(source) = entry.source.as_deref() {
                group.sources.insert(source);
            }
            group.updated_at = &item.created_at;
            let key = TraceabilityKey {
                identifier: entry.metadata.identifier.as_str(),
                label: &entry.metadata.label,
            };
            group.items.insert(
                key,
                TraceabilityItem {
                    label: &entry.metadata.label,
                    value: ReportValue(&entry.value),
                    unit: entry.unit.as_deref(),
                    level: "info",
                    details: TraceabilityDetails {
                        source: entry.source.as_deref(),
                        identifier: entry.metadata.identifier.as_str(),
                    },
                    scope: report_scope(input.into(), &item.scope),
                },
            );
        }
    }
    TraceabilityGroups(groups)
}

fn empty_report_scope(input: Header<'_>) -> ReportScope<'_> {
    ReportScope {
        test_cycle_id: input.cycle.test_cycle_id.as_str(),
        test_execution_id: None,
        test_case_id: None,
        template_id: None,
        step_id: None,
        criterion_id: None,
    }
}

fn traceability_group_title<'a>(
    item: &'a TraceabilitySnapshot,
    entries: &[&TraceabilityEntry],
) -> &'a str {
    if let Some(metadata) = &item.metadata {
        return &metadata.label;
    }
    derived_group_title(item, entries)
}

fn traceability_group_rank(item: &TraceabilitySnapshot) -> TraceabilityGroupRank {
    match item.metadata {
        Some(_) => TraceabilityGroupRank::Stated,
        None => TraceabilityGroupRank::Derived,
    }
}

fn derived_group_title(
    item: &TraceabilitySnapshot,
    entries: &[&TraceabilityEntry],
) -> &'static str {
    // Host-owned groups are titled from their provenance rather than the
    // words in their rows. A row
    // whose label is rewritten stays in the group it was filed under. Everything
    // below is the older ladder, which still names the one-per-resource groups.
    if let Some(group) = entries
        .iter()
        .filter_map(|entry| entry.source.as_deref())
        .find_map(derived_traceability_group)
    {
        return group.label;
    }

    // Every Keyword Is ASCII And Contains No Separating Space.
    // Unicode Lowercase Expansions Are Matched Without Copying The Input.
    let contains = |needle: &str| {
        entries.iter().any(|entry| {
            [
                entry.metadata.identifier.as_str(),
                entry.metadata.label.as_str(),
                entry.source.as_deref().unwrap_or_default(),
            ]
            .into_iter()
            .any(|text| lowercase_contains(text, needle))
        })
    };

    if contains("traceability.python_interfaces") {
        "Python Interfaces"
    } else if contains("traceability.head_unit") {
        "Head Unit Traceability"
    } else if contains("traceability.wifi") || contains("wifi") {
        "WiFi Traceability"
    } else if contains("bluetooth") {
        "Bluetooth Traceability"
    } else if contains("traceability.network") || contains("network") {
        "Network Traceability"
    } else if contains("host") {
        "Host Traceability"
    } else if contains("power") || contains("battery") {
        "Power Traceability"
    } else if contains("environment") || contains("weather") || contains("location") {
        "Environment Traceability"
    } else if contains("execution") || item.scope.test_execution_id.is_some() {
        "Execution Traceability"
    } else if contains("cycle") || contains("suite") {
        "Cycle Traceability"
    } else if item.scope.test_case_id.is_some() {
        "Case Traceability"
    } else if item.scope.test_cycle_id.is_some() {
        "Cycle Traceability"
    } else {
        "Runtime Traceability"
    }
}

fn lowercase_contains(text: &str, needle: &str) -> bool {
    let mut characters = text.chars().flat_map(char::to_lowercase);
    while let Some(character) = characters.next() {
        let mut expected = needle.chars();
        if expected.next() != Some(character) {
            continue;
        }
        let mut tail = characters.clone();
        if expected.all(|character| tail.next() == Some(character)) {
            return true;
        }
    }
    needle.is_empty()
}

fn latest_execution_for_case<'a>(
    input: &'a CycleReportInput,
    test_case_id: &CaseId,
) -> Option<&'a TestExecution> {
    input
        .cycle
        .test_executions
        .iter()
        .rev()
        .find(|execution| &execution.test_case_id == test_case_id)
}

fn execution_status(status: &ExecutionStatus) -> &'static str {
    match status {
        ExecutionStatus::Skipped => "skipped",
        ExecutionStatus::Pending => "created",
        ExecutionStatus::Running => "running",
        ExecutionStatus::Passed => "passed",
        ExecutionStatus::Failed | ExecutionStatus::Blocked => "failed",
        ExecutionStatus::Cancelled => "stopped",
        ExecutionStatus::Interrupted => "interrupted",
        ExecutionStatus::Invalidated => "invalidated",
    }
}

fn queue_status(status: &ExecutionStatus) -> &'static str {
    match status {
        ExecutionStatus::Skipped => "skipped",
        ExecutionStatus::Pending => "queued",
        ExecutionStatus::Running => "running",
        ExecutionStatus::Passed => "passed",
        ExecutionStatus::Failed => "failed",
        ExecutionStatus::Blocked => "blocked",
        ExecutionStatus::Cancelled => "stopped",
        ExecutionStatus::Interrupted => "interrupted",
        ExecutionStatus::Invalidated => "invalidated",
    }
}

fn step_status(status: &StepStatus) -> &'static str {
    match status {
        StepStatus::Passed => "passed",
        StepStatus::Failed => "failed",
        // As the live document reads them: a step still to run or running is
        // not one that was passed over.
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Skipped => "skipped",
        StepStatus::Cancelled => "stopped",
    }
}

fn criterion_status(status: &CriterionStatus) -> &'static str {
    match status {
        CriterionStatus::Passed => "passed",
        CriterionStatus::Failed => "failed",
        // Kept apart from both neighbours. Reporting a criterion that could
        // not be evaluated as skipped would hide that the run tried; reporting
        // it as failed would invent a verdict it never reached.
        CriterionStatus::Indeterminate => "indeterminate",
        CriterionStatus::Skipped => "skipped",
        CriterionStatus::Pending => "pending",
    }
}

fn result_status(status: &ResultStatus) -> &'static str {
    match status {
        ResultStatus::Pending => "pending",
        ResultStatus::Passed => "passed",
        ResultStatus::Failed => "failed",
        ResultStatus::Blocked => "blocked",
        ResultStatus::Cancelled => "cancelled",
    }
}

fn cycle_status(input: Header<'_>) -> &'static str {
    match input.cycle.lifecycle {
        crate::execution::CycleStatus::Running
        | crate::execution::CycleStatus::Paused
        | crate::execution::CycleStatus::Stopping => "running",
        crate::execution::CycleStatus::Completed => "completed",
        crate::execution::CycleStatus::Failed | crate::execution::CycleStatus::Blocked => "failed",
        crate::execution::CycleStatus::Stopped => "stopped",
        _ => "planned",
    }
}

fn run_status(input: Header<'_>) -> &'static str {
    match cycle_status(input) {
        "running" => "running",
        "completed" => "completed",
        "failed" | "stopped" => "failed",
        _ => "idle",
    }
}

fn is_finished_cycle(input: Header<'_>) -> bool {
    matches!(cycle_status(input), "completed" | "failed" | "stopped")
}

fn duration_seconds(started_at: Option<&Timestamp>, finished_at: Option<&Timestamp>) -> f64 {
    started_at
        .zip(finished_at)
        .and_then(|(started_at, finished_at)| started_at.millis_until(finished_at))
        .map(|duration| duration.0 as f64 / 1000.0)
        .unwrap_or(0.0)
}

/* === Tests === */
