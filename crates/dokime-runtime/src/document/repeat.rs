/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Paged Repeated Sections

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::{DocumentSectionRepeat, binding, input::BorrowedData};
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::storage::sqlite::{
    QueryWork,
    data::distinct::{self, Field, Predicate, Query, Selected},
};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct Cursor {
    sources: Vec<SourceCursor>,
    emitted: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct SourceCursor {
    after: Option<[u8; 32]>,
    done: bool,
}

struct Target {
    query: Option<Query>,
    constant: Option<Value>,
    order: Option<Value>,
    ordered: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Cursor {
    pub(crate) fn saw_values(&self) -> bool {
        self.emitted
    }
}

/* === Public Functions === */

pub(crate) fn fields<'a>(
    documents: impl IntoIterator<Item = &'a super::DocumentDefinition>, contracts: &[dokime_domain::model::telemetry::TelemetryContract],
    context: &BTreeMap<String, Value>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut selected = BTreeMap::<String, BTreeSet<String>>::new();
    for document in documents {
        let mut repeats = document.sections.iter().filter_map(|section| section.repeat.as_ref()).collect::<Vec<_>>();
        let mut nodes = document.layout.nodes.iter().collect::<Vec<_>>();
        while let Some(node) = nodes.pop() {
            if let super::DocumentNode::Section(section) = node {
                repeats.extend(section.repeat.as_ref());
                nodes.extend(&section.children);
            }
        }
        for repeat in repeats {
            let contextual = |expression: &DataExpression| match &expression.source {
                DataSource::Parameter { .. } => true,
                DataSource::Dimension { name } => context.contains_key(name),
                _ => false,
            };
            let constant = contextual(&repeat.value);
            let sampled = repeat.filters.iter().any(|filter| {
                !contextual(&filter.expression)
                    && matches!(filter.expression.source, DataSource::Telemetry { .. } | DataSource::Dimension { .. })
            });
            if constant && !sampled {
                continue;
            }
            for contract in contracts {
                if let DataSource::Telemetry { stream, .. } = &repeat.value.source
                    && !super::plot_inputs::stream_matches_alias(contract.stream.as_str(), stream)
                {
                    continue;
                }
                if matches!(repeat.value.source, DataSource::Record { .. } | DataSource::Artifact { .. }) {
                    continue;
                }
                let expressions = std::iter::once(&repeat.value)
                    .chain(repeat.order_by.iter().filter(|_| !constant))
                    .chain(repeat.filters.iter().map(|filter| &filter.expression));
                for expression in expressions {
                    if contextual(expression) {
                        continue;
                    }
                    let name = match &expression.source {
                        DataSource::Telemetry { stream, field }
                            if super::plot_inputs::stream_matches_alias(contract.stream.as_str(), stream) =>
                        {
                            field
                        }
                        DataSource::Dimension { name } => name,
                        _ => continue,
                    };
                    let fields = selected.entry(contract.stream.to_string()).or_default();
                    fields.insert(name.clone());
                    if let Some((_, alias)) = name.rsplit_once('.') {
                        fields.insert(alias.to_owned());
                    }
                }
                // Context Repeats Use a Telemetry Row to Establish Filtered Presence.
                if constant {
                    selected.entry(contract.stream.to_string()).or_default().insert("timestamp".into());
                }
            }
        }
    }
    selected
}

pub(crate) async fn next(
    environment: super::query::Environment<'_>, selection: &super::DocumentSelection, at: &harmos::storage::StorageCursor,
    repeat: &DocumentSectionRepeat, data: BorrowedData<'_>, cursor: &mut Cursor,
) -> crate::runtime::Result<Option<Value>> {
    let targets = targets(environment, selection, at, repeat, data).await?;
    if cursor.sources.is_empty() {
        cursor.sources.resize(targets.len(), SourceCursor::default());
    }
    if cursor.sources.len() != targets.len() {
        return Err(crate::runtime::Error::conflict("document_continuation", "repeat sources differ from the captured cut"));
    }
    let mut heads = Vec::with_capacity(targets.len());
    for (target, source) in targets.iter().zip(&cursor.sources) {
        heads.push(head(environment, at, target, source, None).await?);
    }
    loop {
        let candidate = heads.iter().enumerate().filter_map(|(index, value)| value.as_ref().map(|value| (index, value))).min_by(
            |(left, a), (right, b)| a.order.cmp(&b.order).then(a.first.cmp(&b.first)).then(left.cmp(right)).then(a.key.cmp(&b.key)),
        );
        let Some((index, selected)) = candidate else { return Ok(None) };
        let key = distinct::value_key(&selected.value)?;
        let mut first = (selected.first, index);
        for (other, target) in targets.iter().enumerate() {
            if other == index {
                continue;
            }
            if let Some(value) = head(environment, at, target, &SourceCursor::default(), Some(key)).await? {
                first = first.min((value.first, other));
            }
        }
        let selected = heads[index].take().expect("selected repeat head exists");
        cursor.sources[index].after = Some(selected.key);
        cursor.sources[index].done = targets[index].constant.is_some() || selected.last;
        if first == (selected.first, index) {
            cursor.emitted = true;
            return Ok(Some(selected.value));
        }
        heads[index] = head(environment, at, &targets[index], &cursor.sources[index], None).await?;
    }
}

/* === Private Functions === */

async fn head(
    environment: super::query::Environment<'_>, at: &harmos::storage::StorageCursor, target: &Target, cursor: &SourceCursor,
    key: Option<[u8; 32]>,
) -> crate::runtime::Result<Option<Selected>> {
    if cursor.done {
        return Ok(None);
    }
    if let Some(constant) = &target.constant
        && key.is_some_and(|key| distinct::value_key(constant).ok() != Some(key))
    {
        return Ok(None);
    }
    let mut value = if let Some(query) = &target.query {
        let mut query = query.clone();
        query.key = if target.constant.is_some() { None } else { key };
        environment
            .storage
            .distinct_page(at.clone(), query, cursor.after, 1, QueryWork::document(environment.config.clone(), "dokime.documents.repeat"))
            .await?
            .values
            .into_iter()
            .next()
    } else {
        let value = target.constant.clone().expect("static repeat target has a value");
        Some(Selected {
            key: distinct::value_key(&value)?,
            order: match &target.order {
                Some(order) => distinct::sort_key(order)?,
                None if target.ordered => vec![255],
                None => distinct::sort_key(&value)?,
            },
            value,
            first: 0,
            last: true,
        })
    };
    if let Some(value) = &mut value
        && let Some(constant) = &target.constant
    {
        value.value = constant.clone();
        value.first = 0;
        value.order = match &target.order {
            Some(order) => distinct::sort_key(order)?,
            None if target.ordered => vec![255],
            None => distinct::sort_key(constant)?,
        };
    }
    Ok(value)
}

async fn targets(
    environment: super::query::Environment<'_>, selection: &super::DocumentSelection, at: &harmos::storage::StorageCursor,
    repeat: &DocumentSectionRepeat, data: BorrowedData<'_>,
) -> crate::runtime::Result<Vec<Target>> {
    let mut targets = Vec::new();
    if matches!(repeat.value.source, DataSource::Artifact { .. }) {
        let evidence = binding::Evidence {
            artifacts: data.artifacts,
            context: data.context,
            selection: Some(selection),
            ..Default::default()
        };
        for row in evidence.rows(&repeat.value) {
            if !super::resolve::row_matches_filters(row, repeat.filters.iter(), evidence) {
                continue;
            }
            let Some(value) = binding::row(row, &repeat.value, evidence) else { continue };
            if !super::resolve::value_matches_operator(Some(&value), super::FilterOperator::Exists, None) {
                continue;
            }
            targets.push(Target {
                query: None,
                constant: Some(value.into_owned()),
                order: repeat.order_by.as_ref().and_then(|order| binding::row(row, order, evidence)).map(std::borrow::Cow::into_owned),
                ordered: repeat.order_by.is_some(),
            });
        }
        return Ok(targets);
    }
    let mut scopes = data.context.entries().map(|entry| (entry.scope.clone(), Some(entry))).collect::<Vec<_>>();
    if scopes.is_empty() && selection.scope.test_execution_id.is_some() {
        scopes.push((selection.scope.clone(), None));
    }
    let contextual_value = matches!(repeat.value.source, DataSource::Parameter { .. })
        || matches!(repeat.value.source, DataSource::Dimension { .. })
            && data.context.entries().any(|entry| entry.value(&repeat.value.source).is_some());
    for (scope, context) in scopes {
        let constant = context.and_then(|context| contextual(&repeat.value, context));
        if contextual_value && constant.is_none() {
            continue;
        }
        if constant.as_ref().is_some_and(|value| !super::resolve::value_matches_operator(Some(value), super::FilterOperator::Exists, None))
        {
            continue;
        }
        let sampled = repeat.filters.iter().map(|filter| &filter.expression).any(|expression| {
            matches!(field(expression, context, None), Field::Path { .. }) && !matches!(expression.source, DataSource::Record { .. })
        });
        if constant.is_some() && !sampled {
            if repeat.filters.iter().all(|filter| {
                let Field::Constant(actual) = field(&filter.expression, context, None) else { return false };
                super::resolve::value_matches_operator(actual.as_ref(), filter.operator.clone(), filter.value.as_ref())
            }) {
                targets.push(Target {
                    query: None,
                    constant,
                    order: repeat.order_by.as_ref().and_then(|expression| context.and_then(|context| contextual(expression, context))),
                    ordered: repeat.order_by.is_some(),
                });
            }
            continue;
        }
        let Some(execution) = &scope.test_execution_id else { continue };
        let sources = if let DataSource::Record { record_type } = &repeat.value.source {
            vec![("record", record_type.clone())]
        } else {
            let contracts = environment
                .storage
                .query_telemetry_contracts(
                    crate::storage::sqlite::snapshots::Scope::Execution(execution.to_string()),
                    at.sequence,
                    environment.config.limits().schema_bytes as usize,
                    "dokime.documents.repeat",
                )
                .await?;
            let streams = contracts
                .into_iter()
                .filter(|contract| match &repeat.value.source {
                    DataSource::Telemetry { stream, .. } => super::plot_inputs::stream_matches_alias(contract.stream.as_str(), stream),
                    _ => true,
                })
                .map(|contract| (contract.stream.to_string(), ()))
                .collect::<BTreeMap<_, _>>();
            streams.into_keys().map(|stream| ("telemetry", stream)).collect()
        };
        for (kind, source) in sources {
            let row_field = |expression: &DataExpression| {
                if kind == "record"
                    && matches!(expression.source, DataSource::Dimension { .. })
                    && context.is_none_or(|entry| entry.value(&expression.source).is_none())
                {
                    Field::Constant(None)
                } else {
                    field(expression, context, Some(&source))
                }
            };
            let selected = if constant.is_some() {
                Field::Path {
                    path: vec!["timestamp".into()],
                    fallback: None,
                }
            } else {
                row_field(&repeat.value)
            };
            let Field::Path { path, fallback } = selected else { continue };
            let filters = repeat
                .filters
                .iter()
                .map(|filter| Predicate {
                    field: row_field(&filter.expression),
                    operator: filter.operator.clone(),
                    expected: filter.value.clone(),
                })
                .collect();
            let order = if constant.is_some() { None } else { repeat.order_by.as_ref().map(row_field) };
            targets.push(Target {
                query: Some(Query {
                    execution: execution.to_string(),
                    kind: kind.into(),
                    source,
                    path,
                    fallback,
                    scope: scope.clone(),
                    filters,
                    order,
                    key: None,
                    journal: (kind == "record")
                        .then_some(environment.journal)
                        .flatten()
                        .map(|(epoch, revision)| (epoch.to_owned(), revision)),
                }),
                constant: constant.clone(),
                ordered: repeat.order_by.is_some(),
                order: repeat
                    .order_by
                    .as_ref()
                    .and_then(|expression| context.and_then(|context| contextual(expression, context)))
                    .or_else(|| repeat.order_by.is_none().then(|| constant.clone()).flatten()),
            });
        }
    }
    Ok(targets)
}

fn contextual(expression: &DataExpression, context: super::context::Entry<'_>) -> Option<Value> {
    let mut value = context.value(&expression.source)?.clone();
    for transform in &expression.transforms {
        let DataTransform::Detail { name } = transform else { break };
        let Value::Object(mut fields) = value else { return None };
        value = fields.remove(name)?;
    }
    Some(value)
}

fn field(expression: &DataExpression, context: Option<super::context::Entry<'_>>, stream: Option<&str>) -> Field {
    if context.is_some_and(|entry| entry.value(&expression.source).is_some()) {
        return Field::Constant(context.and_then(|context| contextual(expression, context)));
    }
    let mut transforms = expression.transforms.iter();
    let (first, aliases) = match &expression.source {
        DataSource::Record { .. } => match transforms.next() {
            Some(DataTransform::Detail { name }) => (name.clone(), false),
            _ => return Field::Constant(None),
        },
        DataSource::Telemetry { stream: declared, field } => {
            if stream.is_some_and(|stream| !super::plot_inputs::stream_matches_alias(stream, declared)) {
                return Field::Constant(None);
            }
            (field.clone(), true)
        }
        DataSource::Dimension { name } => (name.clone(), true),
        _ => return Field::Constant(None),
    };
    let mut path = vec![first.clone()];
    path.extend(transforms.filter_map(|transform| match transform {
        DataTransform::Detail { name } => Some(name.clone()),
        _ => None,
    }));
    let fallback = aliases
        .then(|| {
            first.rsplit_once('.').map(|(_, name)| {
                let mut fallback = path.clone();
                fallback[0] = name.to_owned();
                fallback
            })
        })
        .flatten();
    Field::Path { path, fallback }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_repeat_indexes_selectors_and_aliases_without_plot_fields() {
        let mut document = super::super::DocumentDefinition::new("fixture", "Fixture");
        let mut section = super::super::DocumentSection::new("Values");
        let repeat = section.each(crate::expressions::telemetry("sensor", "schema.mode"), "{value}");
        repeat.order_by = Some(crate::expressions::telemetry("sensor", "order"));
        repeat.filters.push(super::super::DocumentFilter {
            expression: crate::expressions::telemetry("sensor", "enabled"),
            operator: super::super::FilterOperator::Exists,
            value: None,
        });
        let mut plot = super::super::Plot::time_series("Continuous");
        plot.y(crate::expressions::telemetry("sensor", "continuous"));
        section.plot(plot);
        document.layout.nodes.push(super::super::DocumentNode::Section(Box::new(super::super::DocumentSectionNode {
            node_id: section.metadata.identifier.clone(),
            metadata: section.metadata.clone(),
            repeat: section.repeat.take(),
            children: vec![],
        })));
        document.sections.push(section);
        let contracts = [dokime_domain::model::telemetry::TelemetryContract::declared("sensor", "Sensor")];
        let selected = fields([&document], &contracts, &BTreeMap::new());
        assert_eq!(
            selected,
            BTreeMap::from([("sensor".into(), BTreeSet::from(["schema.mode".into(), "mode".into(), "order".into(), "enabled".into(),]))])
        );
    }
}
