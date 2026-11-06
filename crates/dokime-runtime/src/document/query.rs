/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Query Document Summary Bindings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use arrow_array::{Array, StringArray, StructArray, UInt64Array};

// Local Imports
use super::{DocumentCardBody, DocumentCardDefinition, DocumentSelection, TelemetrySummaryGroupPayload};
use super::{
    binding::{self, Evidence},
    capture::{Capture, CardBindingResult},
    input::BorrowedData,
};
use crate::expressions::{DataSource, DataTransform};
use crate::storage::sqlite::{QueryProjection, QueryRead, QueryWork, snapshots::Scope};
use crate::{RuntimeHandle, queries::RuntimeStateSnapshot};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ProjectedRow<'a> {
    metadata: &'a StructArray,
    values: &'a StructArray,
    index: usize,
    stream: &'a str,
    context: Option<super::context::Entry<'a>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ProjectedRow<'_> {
    fn value(
        &self, expression: &crate::expressions::DataExpression,
    ) -> Result<Option<Value>, Box<dokime_domain::model::query::error::Failure>> {
        let operation = "dokime.documents.summary";
        let contextual = self.context.and_then(|context| context.value(&expression.source));
        let value = if let Some(value) = contextual {
            Some(value.clone())
        } else {
            let field = match &expression.source {
                DataSource::Telemetry { stream, field } if super::plot_inputs::stream_matches_alias(self.stream, stream) => field,
                DataSource::Dimension { name } => name,
                _ => return Ok(None),
            };
            let column =
                self.values.column_by_name(field).or_else(|| field.rsplit_once('.').and_then(|(_, name)| self.values.column_by_name(name)));
            if let Some(column) = column {
                Some(
                    crate::storage::ipc::cell(column, self.index)
                        .map_err(|error| crate::storage::sqlite::storage_error(error, operation))?,
                )
            } else if field == "timestamp" {
                let column = self
                    .metadata
                    .column_by_name("timestamp")
                    .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                    .filter(|column| !column.is_null(self.index))
                    .ok_or_else(|| {
                        dokime_domain::model::query::error::Failure::new(
                            dokime_domain::model::query::error::Reason::Corruption,
                            operation,
                            "Document timestamp projection is absent",
                        )
                    })?;
                let timestamp = dokime_domain::model::time::Timestamp::new(column.value(self.index)).to_datetime().map_err(|_| {
                    dokime_domain::model::query::error::Failure::new(
                        dokime_domain::model::query::error::Reason::Corruption,
                        operation,
                        "Document timestamp is invalid",
                    )
                })?;
                Some(Value::Datetime(timestamp))
            } else {
                None
            }
        };
        let Some(mut value) = value else { return Ok(None) };
        for transform in &expression.transforms {
            let DataTransform::Detail { name } = transform else { break };
            let Value::Object(mut fields) = value else { return Ok(None) };
            let Some(selected) = fields.remove(name) else { return Ok(None) };
            value = selected;
        }
        Ok(Some(value))
    }
}

/* === Public Functions === */

pub(crate) async fn summaries(
    runtime: &RuntimeHandle, snapshot: &RuntimeStateSnapshot, selection: &DocumentSelection, at: &harmos::storage::StorageCursor,
    cards: Vec<DocumentCardDefinition>, capture: &mut Capture, data: BorrowedData<'_>,
) -> crate::runtime::Result<()> {
    let evidence = Evidence {
        records: data.records,
        artifacts: data.artifacts,
        telemetry: data.telemetry,
        context: data.context,
        selection: Some(selection),
    };
    for card in cards {
        let DocumentCardBody::TelemetrySummary { summary } = &card.body else { continue };
        let mut groups = Vec::new();
        for group in &summary.groups {
            let mut tiles = Vec::new();
            for tile in &group.tiles {
                let query = binding::expression_needs_telemetry(&tile.value, data.context)
                    || matches!(tile.value.source, DataSource::Record { .. })
                    || card
                        .filters
                        .iter()
                        .chain(&tile.filters)
                        .any(|filter| binding::expression_needs_telemetry(&filter.expression, data.context));
                let mut payload = if query {
                    super::resolve::summary_tile_value(tile, None, None)
                } else {
                    super::resolve::telemetry_summary_tile_payload(
                        tile,
                        &card.filters,
                        evidence,
                        super::bounded::DocumentLimits::default().unavailable_reason_bytes,
                    )?
                };
                if query {
                    let filters = card.filters.iter().chain(&tile.filters).cloned().collect();
                    let validation = binding::validate_expression(&tile.value, true).and_then(|()| {
                        binding::validate_predicates(&tile.value, card.filters.iter().chain(&tile.filters))?;
                        for filter in card.filters.iter().chain(&tile.filters) {
                            binding::validate_filter(&filter.expression)?;
                        }
                        Ok(())
                    });
                    let result = if let Err(error) = validation {
                        Err(crate::runtime::Error::Operation(error.to_string()))
                    } else if matches!(tile.value.source, DataSource::Record { .. }) {
                        records(runtime, snapshot, selection, at, tile.value.clone(), filters).await
                    } else {
                        telemetry(runtime, snapshot, selection, at, tile.value.clone(), filters).await
                    };
                    match result {
                        Ok(value) => {
                            payload.value = value;
                            payload.unavailable = None;
                            payload.state =
                                payload.value.as_ref().and_then(|value| super::resolve::matching_summary_state(value, &tile.conditions));
                        }
                        Err(error) => {
                            use crate::runtime::Error;
                            use dokime_domain::model::query::error::Reason;
                            let source_failure = match &error {
                                Error::Storage(_) | Error::Admission(_) => true,
                                Error::Query(failure) => {
                                    matches!(failure.reason, Reason::Corruption | Reason::StorageUnavailable | Reason::SnapshotUnavailable)
                                }
                                _ => false,
                            };
                            if source_failure {
                                return Err(error);
                            }
                            payload.value = None;
                            payload.state = None;
                            payload.unavailable = Some(format!("card {}: {error}", card.metadata.identifier));
                        }
                    }
                }
                tiles.push(payload);
            }
            groups.push(TelemetrySummaryGroupPayload { metadata: group.metadata.clone(), tiles });
        }
        let binding = capture
            .cards
            .iter_mut()
            .find(|binding| binding.metadata.identifier == card.metadata.identifier)
            .expect("captured summary query has a binding");
        binding.result = CardBindingResult::Summary { groups };
        let limits = super::bounded::DocumentLimits::default();
        super::bounded::card_size(binding, limits.materialized_bytes, limits.depth, card.metadata.identifier.as_str())?;
    }
    Ok(())
}

/* === Private Functions === */

async fn telemetry(
    runtime: &RuntimeHandle, snapshot: &RuntimeStateSnapshot, selection: &DocumentSelection, at: &harmos::storage::StorageCursor,
    expression: crate::expressions::DataExpression, filters: Vec<super::DocumentFilter>,
) -> crate::runtime::Result<Option<Value>> {
    let operation = "dokime.documents.summary";
    let execution = selection
        .scope
        .test_execution_id
        .as_ref()
        .ok_or_else(|| crate::runtime::Error::Operation("summary query requires an execution".into()))?;
    let scope = Scope::Execution(execution.to_string());
    let mut contracts = runtime
        .local_storage
        .query_telemetry_contracts(scope.clone(), at.sequence, runtime.query_config().limits().schema_bytes as usize, operation)
        .await
        .map_err(query_error)?;
    for contract in snapshot.state().telemetry_definitions.values() {
        if !contracts.iter().any(|saved| saved.stream == contract.stream && saved.schema_version == contract.schema_version) {
            contracts.push(contract.clone());
        }
    }
    let declared = match &expression.source {
        DataSource::Telemetry { stream, .. } => Some(stream.as_str()),
        DataSource::Dimension { .. } => None,
        DataSource::Parameter { .. } => None,
        _ => unreachable!("telemetry bindings select telemetry or context"),
    };
    let streams = contracts
        .iter()
        .filter(|contract| declared.is_none_or(|declared| super::plot_inputs::stream_matches_alias(contract.stream.as_str(), declared)))
        .map(|contract| contract.stream.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    if filters.is_empty()
        && selection.scope.step_id.is_none()
        && selection.scope.criterion_id.is_none()
        && streams.len() == 1
        && let (DataSource::Telemetry { field, .. }, [DataTransform::Aggregate(aggregate)]) =
            (&expression.source, expression.transforms.as_slice())
        && let Some(stream) = streams.first()
        && contracts.iter().filter(|contract| contract.stream.as_str() == stream).all(|contract| {
            contract.columns.iter().any(|column| {
                column.metadata.identifier.as_str() == field
                    && column.required
                    && !column.nullable
                    && (column.value_type == dokime_domain::model::values::ValueType::Number || *aggregate == harmos::Aggregate::Count)
            })
        })
        && let Some(moments) =
            runtime.local_storage.document_moments(execution.to_string(), stream.clone(), field.clone(), at.clone()).await?
    {
        if *aggregate == harmos::Aggregate::Count {
            return Ok(Some(Value::Unsigned(moments.count)));
        }
        if let Some(value) = moments.value(*aggregate).filter(|value| value.is_finite()) {
            return Ok(Some(Value::Number(value)));
        }
    }
    let expires = dokime_domain::model::identity::wire::Timestamp::from(
        time::OffsetDateTime::now_utc()
            + time::Duration::try_from(runtime.query_config().limits().snapshot_lifetime)
                .map_err(|_| crate::runtime::Error::Operation("invalid query lifetime".into()))?,
    );
    let bytes = serde_json::to_vec(&contracts).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
    let cut = runtime
        .local_storage
        .snapshot_capture(scope, Some(at.clone()), None, expires, bytes, Some(snapshot.revision()))
        .await?
        .ok_or_else(|| crate::runtime::Error::Operation("document query cut unavailable".into()))?;
    let execution_context = snapshot.state().executions.get(execution).map_or_else(super::input::Context::default, |execution| {
        super::input::Context::new(std::slice::from_ref(execution), &selection.scope)
    });
    let contextual = execution_context.supplies(&expression.source);
    let mut state = (super::frame::Fold::new(&expression)?, false);
    for stream in streams {
        let versions =
            contracts.iter().filter(|contract| contract.stream.as_str() == stream).map(|contract| contract.schema_version).collect();
        let selectors = std::iter::once(&expression).chain(filters.iter().map(|filter| &filter.expression));
        let mut fields = std::collections::BTreeSet::new();
        let mut metadata = vec!["position"];
        for selector in selectors {
            if !binding::expression_needs_telemetry(selector, execution_context) {
                continue;
            }
            let field = match &selector.source {
                DataSource::Telemetry { stream: declared, field } if super::plot_inputs::stream_matches_alias(&stream, declared) => field,
                DataSource::Dimension { name } => name,
                _ => continue,
            };
            for contract in contracts.iter().filter(|contract| contract.stream.as_str() == stream) {
                let column = contract.columns.iter().find(|column| column.metadata.identifier.as_str() == field).or_else(|| {
                    field
                        .rsplit_once('.')
                        .and_then(|(_, name)| contract.columns.iter().find(|column| column.metadata.identifier.as_str() == name))
                });
                if let Some(column) = column {
                    fields.insert(column.metadata.identifier.to_string());
                } else if field == "timestamp" && !metadata.contains(&"timestamp") {
                    metadata.push("timestamp");
                }
            }
        }
        let read = QueryRead {
            cut: cut.clone(),
            execution: execution.to_string(),
            stream,
            versions,
            fields: fields.into_iter().collect(),
            between: None,
            after: 0,
            chunk: None,
        };
        let projection = QueryProjection {
            metadata,
            decoded_bytes: super::bounded::DocumentLimits::default().materialized_bytes,
        };
        let selected = selection.clone();
        let snapshot = snapshot.clone();
        let filters = filters.clone();
        let expression = expression.clone();
        let (result, _) = runtime
            .local_storage
            .query_native_batches(
                read,
                QueryWork::document(runtime.query_config().clone(), operation),
                state,
                Some(projection),
                move |(fold, seen), input, contract, work| {
                    let corrupt = || {
                        dokime_domain::model::query::error::Failure::new(
                            dokime_domain::model::query::error::Reason::Corruption,
                            operation,
                            "Document projection layout differs",
                        )
                    };
                    let metadata = input.column(0).as_any().downcast_ref::<StructArray>().ok_or_else(corrupt)?;
                    let values = input.column(1).as_any().downcast_ref::<StructArray>().ok_or_else(corrupt)?;
                    let positions = metadata
                        .column_by_name("position")
                        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
                        .ok_or_else(corrupt)?;
                    let execution = selected.scope.test_execution_id.as_ref().and_then(|id| snapshot.state().executions.get(id));
                    let context = execution.map_or_else(super::input::Context::default, |execution| {
                        super::input::Context::new(std::slice::from_ref(execution), &selected.scope)
                    });
                    let context = context.entries().next();
                    for index in 0..input.num_rows() {
                        work.check()?;
                        if contextual && *seen {
                            break;
                        }
                        let row = ProjectedRow {
                            metadata,
                            values,
                            index,
                            stream: contract.stream.as_str(),
                            context,
                        };
                        let mut matches = true;
                        for filter in &filters {
                            let actual = row.value(&filter.expression)?;
                            if !super::resolve::value_matches_operator(actual.as_ref(), filter.operator.clone(), filter.value.as_ref()) {
                                matches = false;
                                break;
                            }
                        }
                        if !matches {
                            continue;
                        }
                        let value = row.value(&expression)?;
                        fold.push_at(positions.value(index), value.as_ref()).map_err(|error| {
                            dokime_domain::model::query::error::Failure::new(
                                dokime_domain::model::query::error::Reason::InvalidSelection,
                                operation,
                                error.to_string(),
                            )
                        })?;
                        *seen = true;
                    }
                    Ok(())
                },
            )
            .await
            .map_err(query_error)?;
        state = result;
    }
    Ok(state.0.finish()?)
}

async fn records(
    runtime: &RuntimeHandle, snapshot: &RuntimeStateSnapshot, selection: &DocumentSelection, at: &harmos::storage::StorageCursor,
    expression: crate::expressions::DataExpression, filters: Vec<super::DocumentFilter>,
) -> crate::runtime::Result<Option<Value>> {
    let execution = selection
        .scope
        .test_execution_id
        .as_ref()
        .and_then(|id| snapshot.state().executions.get(id))
        .ok_or_else(|| crate::runtime::Error::Operation("record summary requires an execution".into()))?;
    let mut fold = super::frame::Fold::new(&expression)?;
    let mut previous = std::collections::BTreeMap::<(String, String, String), i128>::new();
    let mut after = 0;
    let context = super::input::Context::new(std::slice::from_ref(execution), &selection.scope);
    loop {
        let rows = runtime
            .local_storage
            .scoped_records(
                at.clone(),
                crate::evidence_runtime::ScopeSelection {
                    test_execution_id: Some(execution.test_execution_id.clone()),
                    ..Default::default()
                },
                None,
                after,
                0,
                u64::MAX,
            )
            .await?;
        let Some(last) = rows.last() else { break };
        after = last.position;
        for row in &rows {
            let record = &row.record;
            let scope = execution.records.captured_scope(record).or(record.scope.as_ref());
            if selection.foreign_execution_scope(scope) {
                return Err(crate::runtime::Error::conflict("document_capture_incomplete", "record capture scope differs"));
            }
            if !selection.matches_scope(scope) {
                continue;
            }
            let view = super::input::RecordView { entry: record, scope };
            let evidence = Evidence {
                context,
                selection: Some(selection),
                ..Default::default()
            };
            let value = |expression: &crate::expressions::DataExpression| {
                if let Some(DataTransform::TimeBetween { previous_record_type }) = expression.transforms.first() {
                    let key = record_key(previous_record_type, scope);
                    let start = previous.get(&key)?;
                    let end = record.timestamp.to_datetime().ok()?.unix_timestamp_nanos();
                    let elapsed = end.checked_sub(*start).filter(|elapsed| *elapsed >= 0)?;
                    return Some(if elapsed % 1_000_000_000 == 0 {
                        Value::Unsigned(u64::try_from(elapsed / 1_000_000_000).ok()?)
                    } else {
                        Value::Number(elapsed as f64 / 1_000_000_000.0)
                    });
                }
                binding::row(
                    binding::Row::Record {
                        record: view,
                        context: context.for_scope(scope),
                    },
                    expression,
                    evidence,
                )
                .map(std::borrow::Cow::into_owned)
            };
            let DataSource::Record { record_type } = &expression.source else { unreachable!() };
            if record.record_type.as_str() == record_type
                && filters.iter().all(|filter| {
                    super::resolve::value_matches_operator(
                        value(&filter.expression).as_ref(),
                        filter.operator.clone(),
                        filter.value.as_ref(),
                    )
                })
            {
                fold.push_at(row.position, value(&expression).as_ref())?;
            }
            if let Ok(time) = record.timestamp.to_datetime() {
                previous.insert(record_key(record.record_type.as_str(), scope), time.unix_timestamp_nanos());
            }
        }
    }
    Ok(fold.finish()?)
}

fn record_key(record_type: &str, scope: Option<&dokime_domain::model::execution::RecordScope>) -> (String, String, String) {
    use dokime_domain::model::execution::EvidenceOrigin;
    let execution = scope.and_then(|scope| scope.test_execution_id.as_ref()).map_or("", |id| id.as_str());
    let attempt = match scope.map(|scope| &scope.origin) {
        Some(EvidenceOrigin::Step { attempt_id }) => attempt_id.as_str(),
        _ => "",
    };
    (record_type.to_owned(), execution.to_owned(), attempt.to_owned())
}

fn query_error(error: Box<dokime_domain::model::query::error::Failure>) -> crate::runtime::Error {
    crate::runtime::Error::Query(error)
}

/* === Tests === */
