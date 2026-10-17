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
use std::borrow::Cow;

// Third Party Imports
use harmos::Aggregate;

// Local Imports
use super::context::{Context, Entry};
use super::frame::Plan;
use super::input::{Artifacts, RecordView, Records};
use super::{DocumentCardBody, DocumentCardDefinition, DocumentDefinition, DocumentFailure, DocumentNode, Result};
use crate::artifacts::{ArtifactRecord, ArtifactStatus};
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy, Default)]
pub(super) struct Evidence<'a> {
    pub records: Records<'a>,
    pub artifacts: Artifacts<'a>,
    pub telemetry: &'a [TelemetrySample],
    pub context: Context<'a>,
    pub selection: Option<&'a super::DocumentSelection>,
}

/* === Enums === */
#[derive(Clone, Copy)]
pub(super) enum Row<'a> {
    Artifact { artifact: &'a ArtifactRecord, context: Option<Entry<'a>> },
    Record { record: RecordView<'a>, context: Option<Entry<'a>> },
    Sample { sample: &'a TelemetrySample, context: Option<Entry<'a>> },
    Context(Entry<'a>),
}

/* === Traits === */

/* === Implementations === */
impl<'a> Evidence<'a> {
    pub(super) fn upper_rows(self) -> usize {
        self.telemetry.len().saturating_add(self.context.len()).saturating_add(self.records.len()).saturating_add(self.artifacts.len())
    }

    pub(super) fn selected(self, sample: &TelemetrySample) -> bool {
        self.selection.is_none_or(|selection| selection.matches_scope(sample.scope.as_ref()))
    }

    pub(super) fn sample(self, sample: &'a TelemetrySample) -> Row<'a> {
        Row::Sample { sample, context: self.context.for_scope(sample.scope.as_ref()) }
    }

    pub(super) fn rows(self, expression: &DataExpression) -> impl DoubleEndedIterator<Item = Row<'a>> {
        self.rows_for_case(expression, None)
    }

    pub(super) fn rows_for_case(self, expression: &DataExpression, case: Option<Entry<'a>>) -> impl DoubleEndedIterator<Item = Row<'a>> {
        let contextual = matches!(expression.source, DataSource::Parameter { .. })
            || (matches!(expression.source, DataSource::Dimension { .. })
                && self
                    .context
                    .entries()
                    .filter(|entry| case.is_none_or(|case| case.covers(Some(entry.scope))))
                    .any(|entry| entry.value(&expression.source).is_some()));
        let uses_facts = matches!(expression.source, DataSource::Record { .. } | DataSource::Artifact { .. });
        let samples = self
            .telemetry
            .iter()
            .filter(move |sample| {
                !contextual && !uses_facts && self.selected(sample) && case.is_none_or(|case| case.covers(sample.scope.as_ref()))
            })
            .map(move |sample| Row::Sample { sample, context: case.or_else(|| self.context.for_scope(sample.scope.as_ref())) });
        let records = self
            .records
            .iter()
            .filter(move |record| {
                let DataSource::Record { record_type } = &expression.source else {
                    return false;
                };
                record.entry.record_type.as_str() == record_type
                    && self.selection.is_none_or(|selection| selection.matches_scope(record.scope))
                    && case.is_none_or(|case| case.covers(record.scope))
            })
            .map(move |record| Row::Record { record, context: case.or_else(|| self.context.for_scope(record.scope)) });
        let contexts = self
            .context
            .entries()
            .filter(move |context| contextual && case.is_none_or(|case| case.covers(Some(context.scope))))
            .map(Row::Context);
        let artifacts = self
            .artifacts
            .iter()
            .filter(move |record| {
                let DataSource::Artifact { artifact } = &expression.source else {
                    return false;
                };
                record.filename == *artifact
                    && self.selection.is_none_or(|selection| selection.matches_scope(Some(&record.scope)))
                    && case.is_none_or(|case| case.covers(Some(&record.scope)))
            })
            .map(move |artifact| Row::Artifact { artifact, context: case.or_else(|| self.context.for_scope(Some(&artifact.scope))) });
        samples.chain(records).chain(contexts).chain(artifacts)
    }
}

impl<'a> From<&'a [TelemetrySample]> for Evidence<'a> {
    fn from(telemetry: &'a [TelemetrySample]) -> Self {
        Self { records: Records::default(), artifacts: Artifacts::default(), telemetry, context: Context::default(), selection: None }
    }
}

impl<'a> From<&'a TelemetrySample> for Row<'a> {
    fn from(sample: &'a TelemetrySample) -> Self {
        Self::Sample { sample, context: None }
    }
}

impl Row<'_> {
    pub(super) fn needs_sample(self, expression: &DataExpression) -> bool {
        match expression.source {
            DataSource::Telemetry { .. } => true,
            DataSource::Dimension { .. } => match self {
                Self::Context(entry) => entry.value(&expression.source).is_none(),
                Self::Sample { .. } => false,
                Self::Record { .. } => false,
                Self::Artifact { .. } => false,
            },
            _ => false,
        }
    }
}

/* === Public Functions === */
pub(super) fn row_text<'a>(row: Row<'a>, expression: &DataExpression) -> Option<&'a str> {
    let Row::Artifact { artifact, .. } = row else {
        return None;
    };
    let DataSource::Artifact { artifact: filename } = &expression.source else {
        return None;
    };
    if &artifact.filename != filename {
        return None;
    }
    let Some(DataTransform::Detail { name }) = expression.transforms.first() else {
        return None;
    };
    artifact_text_field(artifact, name)
}

pub(crate) fn telemetry_needed(definition: &DocumentDefinition, context: Context<'_>) -> bool {
    bindings_need(definition, |expression| expression_needs_telemetry(expression, context))
}

pub(crate) fn records_needed(definition: &DocumentDefinition) -> bool {
    bindings_need(definition, |expression| matches!(expression.source, DataSource::Record { .. }))
}

pub(crate) fn artifacts_needed(definition: &DocumentDefinition) -> bool {
    bindings_need(definition, |expression| matches!(expression.source, DataSource::Artifact { .. }))
}

pub(crate) fn context_needed(definition: &DocumentDefinition) -> bool {
    bindings_need(definition, |expression| {
        matches!(expression.source, DataSource::Parameter { .. } | DataSource::Dimension { .. })
            || expression.transforms.contains(&DataTransform::PerCase)
    })
}

pub(super) fn validate(cards: &[DocumentCardDefinition], nodes: &[DocumentNode]) -> Result<()> {
    for card in cards {
        for filter in &card.filters {
            validate_expression(&filter.expression, false)?;
        }
        if let DocumentCardBody::TelemetrySummary { summary } = &card.body {
            for tile in summary.groups.iter().flat_map(|group| &group.tiles) {
                validate_expression(&tile.value, true)?;
                validate_predicates(&tile.value, card.filters.iter().chain(tile.filters.iter()))?;
                for filter in &tile.filters {
                    validate_expression(&filter.expression, false)?;
                }
            }
        }
    }
    for node in nodes {
        let DocumentNode::Section(section) = node else {
            continue;
        };
        if let Some(repeat) = &section.repeat {
            validate_repeat(repeat)?;
        }
        validate(&[], &section.children)?;
    }
    Ok(())
}

pub(super) fn validate_repeat(repeat: &super::DocumentSectionRepeat) -> Result<()> {
    validate_expression(&repeat.value, false)?;
    validate_predicates(&repeat.value, repeat.filters.iter())?;
    if let Some(order) = &repeat.order_by {
        validate_expression(order, false)?;
        validate_pair(&repeat.value, order)?;
    }
    for filter in &repeat.filters {
        validate_expression(&filter.expression, false)?;
    }
    Ok(())
}

pub(super) fn validate_filter(expression: &DataExpression) -> Result<()> {
    validate_expression(expression, false)
}

pub(super) fn validate_predicates<'a>(expression: &DataExpression, filters: impl Iterator<Item = &'a super::DocumentFilter>) -> Result<()> {
    for filter in filters {
        validate_pair(expression, &filter.expression)?;
    }
    Ok(())
}

pub(super) fn scratch_memory(definition: &DocumentDefinition, rows: usize) -> usize {
    summary_expressions(definition).map(|expression| plan(expression, rows).map_or(usize::MAX, |plan| plan.memory(rows))).max().unwrap_or(0)
}

pub(super) fn evaluation_work(definition: &DocumentDefinition, rows: usize, instances: usize) -> usize {
    summary_expressions(definition)
        .fold(0usize, |total, expression| total.saturating_add(plan(expression, rows).map_or(usize::MAX, |plan| plan.work(rows))))
        .saturating_mul(instances.max(1))
}

pub(super) fn correlation_work(definition: &DocumentDefinition, rows: usize, instances: usize) -> usize {
    fn correlated(expression: &DataExpression, rows: usize) -> usize {
        let lookup = expression
            .transforms
            .iter()
            .any(|transform| matches!(transform, DataTransform::TimeBetween { .. } | DataTransform::TelemetryAtOrBefore { .. }));
        let per_case = expression.transforms.contains(&DataTransform::PerCase);
        if !lookup && !per_case {
            return 0;
        }
        rows.saturating_mul(rows).saturating_mul(usize::from(lookup) + usize::from(per_case))
    }
    let mut expressions = 0usize;
    for (filters, body) in definition
        .cards
        .iter()
        .map(|card| (&card.filters, &card.body))
        .chain(definition.sections.iter().flat_map(|section| section.cards.iter().map(|card| (&card.filters, &card.body))))
    {
        if let DocumentCardBody::TelemetrySummary { summary } = body {
            expressions = filters.iter().fold(expressions, |count, filter| count.saturating_add(correlated(&filter.expression, rows)));
            for tile in summary.groups.iter().flat_map(|group| &group.tiles) {
                expressions = expressions.saturating_add(correlated(&tile.value, rows));
                expressions =
                    tile.filters.iter().fold(expressions, |count, filter| count.saturating_add(correlated(&filter.expression, rows)));
            }
        }
    }
    fn repeat_count(repeat: &super::DocumentSectionRepeat, rows: usize) -> usize {
        repeat.filters.iter().fold(
            correlated(&repeat.value, rows).saturating_add(repeat.order_by.as_ref().map_or(0, |order| correlated(order, rows))),
            |count, filter| count.saturating_add(correlated(&filter.expression, rows)),
        )
    }
    fn node_count(nodes: &[DocumentNode], rows: usize) -> usize {
        nodes.iter().fold(0usize, |count, node| match node {
            DocumentNode::Section(section) => count
                .saturating_add(section.repeat.as_ref().map_or(0, |repeat| repeat_count(repeat, rows)))
                .saturating_add(node_count(&section.children, rows)),
            _ => count,
        })
    }
    expressions = definition
        .sections
        .iter()
        .fold(expressions, |count, section| count.saturating_add(section.repeat.as_ref().map_or(0, |repeat| repeat_count(repeat, rows))))
        .saturating_add(node_count(&definition.layout.nodes, rows));
    expressions.saturating_mul(instances.max(1))
}

pub(super) fn row<'a>(input: Row<'a>, expression: &DataExpression, evidence: Evidence<'a>) -> Option<Cow<'a, Value>> {
    let (sample, record, artifact, context) = match input {
        Row::Sample { sample, context } => (Some(sample), None, None, context),
        Row::Record { record, context } => (None, Some(record), None, context),
        Row::Artifact { artifact, context } => (None, None, Some(artifact), context),
        Row::Context(context) => (None, None, None, Some(context)),
    };
    let contextual = context.and_then(|context| context.value(&expression.source));
    let mut transforms = expression.transforms.iter();
    let mut value = if let Some(value) = contextual {
        Cow::Borrowed(value)
    } else if let DataSource::Record { record_type } = &expression.source {
        let record = record?;
        if record.entry.record_type.as_str() != record_type {
            return None;
        }
        match transforms.next()? {
            DataTransform::Detail { name } => Cow::Borrowed(record.entry.details.get(name)?),
            DataTransform::TimeBetween { .. } | DataTransform::TelemetryAtOrBefore { .. } => correlate(record, expression, evidence)?,
            _ => return None,
        }
    } else if let DataSource::Artifact { artifact: filename } = &expression.source {
        let artifact = artifact?;
        if &artifact.filename != filename {
            return None;
        }
        let DataTransform::Detail { name } = transforms.next()? else {
            return None;
        };
        Cow::Owned(artifact_field(artifact, name)?)
    } else {
        let sample = sample?;
        let field = match &expression.source {
            DataSource::Telemetry { stream, field } if super::plot_inputs::stream_matches_alias(sample.stream.as_str(), stream) => field,
            DataSource::Dimension { name } => name,
            _ => return None,
        };
        if let Some(value) = sample.values.get(field) {
            Cow::Borrowed(value)
        } else if field == "timestamp" {
            Cow::Owned(Value::Datetime(sample.timestamp.to_datetime().ok()?))
        } else {
            Cow::Borrowed(sample.values.get(field.rsplit_once('.')?.1)?)
        }
    };
    for transform in transforms {
        let DataTransform::Detail { name } = transform else {
            break;
        };
        value = match value {
            Cow::Borrowed(Value::Object(fields)) => Cow::Borrowed(fields.get(name)?),
            Cow::Owned(Value::Object(mut fields)) => Cow::Owned(fields.remove(name)?),
            _ => return None,
        };
    }
    Some(value)
}

pub(super) fn evaluate(expression: &DataExpression, evidence: Evidence<'_>, selected: impl Fn(Row<'_>) -> bool) -> Result<Option<Value>> {
    validate_expression(expression, true)?;
    let per_case = expression.transforms.contains(&DataTransform::PerCase);
    if per_case && !evidence.context.available() {
        return Err(failure("per-case bindings require the original execution context"));
    }
    let cases = if per_case { evidence.context.cases().collect::<Vec<_>>() } else { Vec::new() };
    let plan = Plan::compile(expression, per_case.then_some(cases.len()))?;
    plan.evaluate(expression, evidence, &cases, selected)
}

/* === Private Functions === */
fn plan(expression: &DataExpression, rows: usize) -> Result<Plan> {
    let cases = expression.transforms.contains(&DataTransform::PerCase).then_some(rows.clamp(1, 4096));
    Plan::compile(expression, cases)
}

fn correlate<'a>(record: RecordView<'a>, expression: &DataExpression, evidence: Evidence<'a>) -> Option<Cow<'a, Value>> {
    let selector = DataExpression {
        label: None,
        source: expression.source.clone(),
        transforms: expression.transforms.first().cloned().into_iter().collect(),
    };
    let plan = Plan::compile(&selector, None).ok()?;
    plan.correlate(record, &selector, evidence).ok().flatten()
}

fn bindings_need(definition: &DocumentDefinition, needs: impl Fn(&DataExpression) -> bool + Copy) -> bool {
    let summaries = definition
        .cards
        .iter()
        .map(|card| (&card.filters, &card.body))
        .chain(definition.sections.iter().flat_map(|section| section.cards.iter().map(|card| (&card.filters, &card.body))))
        .any(|(filters, body)| {
            let DocumentCardBody::TelemetrySummary { summary } = body else {
                return false;
            };
            filters.iter().any(|filter| needs(&filter.expression))
                || summary
                    .groups
                    .iter()
                    .flat_map(|group| &group.tiles)
                    .any(|tile| needs(&tile.value) || tile.filters.iter().any(|filter| needs(&filter.expression)))
        });
    summaries
        || definition.sections.iter().filter_map(|section| section.repeat.as_ref()).any(|repeat| repeat_needs(repeat, needs))
        || nodes_need(&definition.layout.nodes, needs)
}

fn validate_pair(expression: &DataExpression, other: &DataExpression) -> Result<()> {
    let compatible = match (&expression.source, &other.source) {
        (DataSource::Record { record_type: selected }, DataSource::Record { record_type }) => selected == record_type,
        (DataSource::Artifact { artifact: selected }, DataSource::Artifact { artifact }) => selected == artifact,
        (_, DataSource::Record { .. })
        | (_, DataSource::Artifact { .. })
        | (DataSource::Record { .. } | DataSource::Artifact { .. }, DataSource::Telemetry { .. }) => false,
        _ => true,
    };
    if !compatible {
        return Err(failure("cross-source fact selectors require a bounded correlation adapter"));
    }
    Ok(())
}

fn expression_needs_telemetry(expression: &DataExpression, context: Context<'_>) -> bool {
    matches!(expression.source, DataSource::Telemetry { .. })
        || expression.transforms.iter().any(|transform| matches!(transform, DataTransform::TelemetryAtOrBefore { .. }))
        || (matches!(expression.source, DataSource::Dimension { .. }) && !context.supplies(&expression.source))
}

fn repeat_needs(repeat: &super::DocumentSectionRepeat, needs: impl Fn(&DataExpression) -> bool + Copy) -> bool {
    needs(&repeat.value) || repeat.order_by.as_ref().is_some_and(needs) || repeat.filters.iter().any(|filter| needs(&filter.expression))
}

fn nodes_need(nodes: &[DocumentNode], needs: impl Fn(&DataExpression) -> bool + Copy) -> bool {
    nodes.iter().any(|node| match node {
        DocumentNode::Section(section) => {
            section.repeat.as_ref().is_some_and(|repeat| repeat_needs(repeat, needs)) || nodes_need(&section.children, needs)
        }
        _ => false,
    })
}

fn summary_expressions(definition: &DocumentDefinition) -> impl Iterator<Item = &DataExpression> {
    definition
        .cards
        .iter()
        .map(|card| &card.body)
        .chain(definition.sections.iter().flat_map(|section| section.cards.iter().map(|card| &card.body)))
        .filter_map(|body| match body {
            DocumentCardBody::TelemetrySummary { summary } => Some(summary),
            _ => None,
        })
        .flat_map(|summary| summary.groups.iter().flat_map(|group| &group.tiles))
        .map(|tile| &tile.value)
}

fn validate_expression(expression: &DataExpression, aggregate: bool) -> Result<()> {
    if let Some(index) = expression.transforms.iter().position(|transform| matches!(transform, DataTransform::PerCase)) {
        let suffix = &expression.transforms[index + 1..];
        if !aggregate || suffix.len() != 1 || suffix.first().is_some_and(|transform| !matches!(transform, DataTransform::Aggregate(_))) {
            return Err(failure("per-case scalar summaries require one final outer fold"));
        }
        return validate_selector(&expression.source, &expression.transforms[..index], true);
    }
    validate_selector(&expression.source, &expression.transforms, aggregate)
}

fn validate_selector(source: &DataSource, transforms: &[DataTransform], aggregate: bool) -> Result<()> {
    if matches!(source, DataSource::Artifact { .. }) {
        if aggregate && transforms == [DataTransform::Aggregate(Aggregate::Count)] {
            return Ok(());
        }
        let Some(DataTransform::Detail { name }) = transforms.first() else {
            return Err(failure("artifact scalar bindings require a metadata selector or an occurrence count"));
        };
        if !matches!(
            name.as_str(),
            "artifact_id" | "upload_id" | "filename" | "content_type" | "size_bytes" | "sha256" | "status" | "created_at"
        ) || transforms[1..].iter().any(|transform| matches!(transform, DataTransform::Detail { .. }))
        {
            return Err(failure("artifact bindings require a declared scalar metadata field"));
        }
    }
    if matches!(source, DataSource::Record { .. })
        && !matches!(
            transforms.first(),
            Some(DataTransform::Detail { .. } | DataTransform::TimeBetween { .. } | DataTransform::TelemetryAtOrBefore { .. })
        )
        && !(aggregate && transforms == [DataTransform::Aggregate(Aggregate::Count)])
    {
        return Err(failure("record scalar bindings require a selector or an occurrence count"));
    }
    for (index, transform) in transforms.iter().enumerate() {
        if matches!(transform, DataTransform::Detail { .. }) {
            if transforms.iter().take(index).any(|transform| matches!(transform, DataTransform::TimeBetween { .. })) {
                return Err(failure("elapsed record seconds do not have detail fields"));
            }
            continue;
        }
        if index == 0
            && matches!(source, DataSource::Record { .. })
            && matches!(transform, DataTransform::TimeBetween { .. } | DataTransform::TelemetryAtOrBefore { .. })
        {
            continue;
        }
        if aggregate && index + 1 == transforms.len() && matches!(transform, DataTransform::Aggregate(_)) {
            continue;
        }
        return Err(failure("row selectors permit detail transforms; summaries permit one final scalar fold"));
    }
    Ok(())
}

fn failure(message: &'static str) -> crate::Error {
    DocumentFailure::Evaluation { message }.into()
}

fn artifact_field(artifact: &ArtifactRecord, name: &str) -> Option<Value> {
    Some(match name {
        "size_bytes" => Value::Unsigned(artifact.size_bytes),
        "created_at" => Value::Datetime(artifact.created_at.to_datetime().ok()?),
        "content_type" | "sha256" => artifact_text_field(artifact, name).map_or(Value::Null, |value| Value::Text(value.into())),
        "artifact_id" | "upload_id" | "filename" | "status" => Value::Text(artifact_text_field(artifact, name)?.into()),
        _ => return None,
    })
}

fn artifact_text_field<'a>(artifact: &'a ArtifactRecord, name: &str) -> Option<&'a str> {
    match name {
        "artifact_id" => Some(artifact.artifact_id.as_str()),
        "upload_id" => Some(artifact.upload_id.as_str()),
        "filename" => Some(&artifact.filename),
        "content_type" => artifact.content_type.as_deref(),
        "sha256" => artifact.sha256.as_deref(),
        "status" => Some(match artifact.status {
            ArtifactStatus::Staged => "staged",
            ArtifactStatus::Complete => "complete",
            ArtifactStatus::Cancelled => "cancelled",
            ArtifactStatus::Rejected => "rejected",
        }),
        _ => None,
    }
}

/* === Tests === */
