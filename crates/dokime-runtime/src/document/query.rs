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
use std::io::Write;

// Third Party Imports
use arrow_array::{Array, StringArray, StructArray, UInt64Array};
use sha2::{Digest, Sha256};
use tokio::sync::OnceCell;

// Local Imports
use super::{DocumentCardBody, DocumentCardDefinition, DocumentSelection, TelemetrySummaryGroupPayload};
use super::{
    binding::{self, Evidence},
    capture::{CardBinding, CardBindingResult},
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

pub(crate) struct Context<'a> {
    runtime: &'a RuntimeHandle,
    snapshot: &'a RuntimeStateSnapshot,
    selection: &'a DocumentSelection,
    pub(crate) at: &'a harmos::storage::StorageCursor,
    artifacts: OnceCell<ArtifactInput>,
}

struct ArtifactInput {
    rows: harmos::storage::ReadBatch<Vec<crate::artifacts::ArtifactRecord>>,
    sha256: String,
}

struct HashWriter(Sha256);

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

impl Context<'_> {
    pub(crate) fn new<'a>(
        runtime: &'a RuntimeHandle, snapshot: &'a RuntimeStateSnapshot, selection: &'a DocumentSelection,
        at: &'a harmos::storage::StorageCursor,
    ) -> Context<'a> {
        Context {
            runtime,
            snapshot,
            selection,
            at,
            artifacts: OnceCell::new(),
        }
    }

    pub(crate) fn artifact_digest(&self) -> Option<&str> {
        self.artifacts.get().map(|input| input.sha256.as_str())
    }

    pub(crate) fn validate_membership(&self) -> crate::runtime::Result<()> {
        if self.selection.scope.test_execution_id.is_none()
            && let Some(id) = &self.selection.scope.test_cycle_id
        {
            let captured = self.snapshot.state().cycles.get(id);
            let current = self.runtime.watch_state().latest();
            if captured
                .is_none_or(|captured| current.state().cycles.get(id).is_none_or(|current| current.execution_ids != captured.execution_ids))
            {
                return Err(crate::runtime::Error::conflict("document_capture_incomplete", "cycle members changed"));
            }
        }
        Ok(())
    }

    pub(crate) async fn artifacts(&self) -> crate::runtime::Result<&[crate::artifacts::ArtifactRecord]> {
        let input = self
            .artifacts
            .get_or_try_init(|| async {
                self.validate_membership()?;
                let executions = if let Some(id) = &self.selection.scope.test_execution_id {
                    vec![id.clone()]
                } else {
                    let id = self
                        .selection
                        .scope
                        .test_cycle_id
                        .as_ref()
                        .ok_or_else(|| crate::runtime::Error::Operation("artifact query requires an owner".into()))?;
                    let cycle = self
                        .snapshot
                        .state()
                        .cycles
                        .get(id)
                        .ok_or_else(|| crate::runtime::Error::not_found("document_not_found", "document cycle was not found"))?;
                    cycle
                        .execution_ids
                        .iter()
                        .filter(|id| {
                            self.selection.scope.test_case_id.as_ref().is_none_or(|case| {
                                self.snapshot.state().executions.get(*id).is_some_and(|execution| &execution.test_case_id == case)
                            })
                        })
                        .cloned()
                        .collect()
                };
                let rows = self.runtime.execution_storage.selected_artifact_records(self.runtime.storage(), self.at, &executions).await?;
                validate_cycle_artifacts(self.selection, self.snapshot, rows.as_slice())
                    .map_err(|_| crate::runtime::Error::conflict("document_capture_incomplete", "artifact capture scope differs"))?;
                if rows.iter().any(|record| self.selection.foreign_execution_scope(Some(&record.scope))) {
                    return Err(crate::runtime::Error::conflict("document_capture_incomplete", "artifact capture scope differs"));
                }
                let mut digest = HashWriter(Sha256::new());
                for record in rows.iter().filter(|record| self.selection.matches_scope(Some(&record.scope))) {
                    serde_json::to_writer(&mut digest, record).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
                    digest.write_all(b"\n").map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
                }
                let sha256 = digest.0.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
                Ok::<_, crate::runtime::Error>(ArtifactInput { rows, sha256 })
            })
            .await?;
        Ok(input.rows.as_slice())
    }

    pub(crate) async fn card(
        &self, card: DocumentCardDefinition, mut binding: CardBinding, data: BorrowedData<'_>,
    ) -> crate::runtime::Result<CardBinding> {
        let (runtime, selection, at) = (self.runtime, self.selection, self.at);
        let limits = super::bounded::DocumentLimits::default();
        let DocumentCardBody::TelemetrySummary { summary } = &card.body else { return Ok(binding) };
        let artifacts_needed = card
            .filters
            .iter()
            .map(|filter| &filter.expression)
            .chain(
                summary
                    .groups
                    .iter()
                    .flat_map(|group| &group.tiles)
                    .flat_map(|tile| std::iter::once(&tile.value).chain(tile.filters.iter().map(|filter| &filter.expression))),
            )
            .any(|expression| matches!(expression.source, DataSource::Artifact { .. }));
        let data = BorrowedData {
            artifacts: if artifacts_needed { super::input::Artifacts::Native(self.artifacts().await?) } else { data.artifacts },
            ..data
        };
        let evidence = Evidence {
            records: data.records,
            artifacts: data.artifacts,
            telemetry: data.telemetry,
            context: data.context,
            selection: Some(selection),
        };
        let mut output_bytes = 0usize;
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
                        records(runtime, data.context, selection, at, tile.value.clone(), filters).await
                    } else {
                        telemetry(runtime, data.context, selection, at, tile.value.clone(), filters).await
                    };
                    match result {
                        Ok(value) => {
                            payload.value = value;
                            payload.unavailable = None;
                            payload.state =
                                payload.value.as_ref().and_then(|value| super::resolve::matching_summary_state(value, &tile.conditions));
                        }
                        Err(crate::runtime::Error::Query(mut error)) => {
                            error.message = format!("tile {}: {}", tile.metadata.identifier, error.message);
                            return Err(error.into());
                        }
                        Err(error) => return Err(error),
                    }
                }
                if let Some(reason) = &payload.unavailable {
                    return Err(crate::runtime::Error::Operation(reason.clone()));
                }
                output_bytes = output_bytes.saturating_add(
                    super::bounded::document_size(&payload, usize::MAX, limits.depth)
                        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?,
                );
                if output_bytes > limits.materialized_bytes {
                    return Err(crate::Error::Document(super::DocumentFailure::Budget {
                        budget: "materialized_bytes",
                        limit: limits.materialized_bytes,
                        reached: output_bytes,
                        card: card.metadata.identifier.to_string(),
                    })
                    .into());
                }
                tiles.push(payload);
            }
            groups.push(TelemetrySummaryGroupPayload { metadata: group.metadata.clone(), tiles });
        }
        binding.result = CardBindingResult::Summary { groups };
        super::bounded::card_size(&binding, limits.materialized_bytes, limits.depth, card.metadata.identifier.as_str())?;
        Ok(binding)
    }
}

impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

pub(crate) fn validate_cycle_artifacts(
    selection: &DocumentSelection, snapshot: &crate::queries::RuntimeStateSnapshot, records: &[crate::artifacts::ArtifactRecord],
) -> harmos::Result<()> {
    if selection.scope.test_execution_id.is_some() {
        return Ok(());
    }
    let Some(id) = &selection.scope.test_cycle_id else {
        return Ok(());
    };
    let cycle = snapshot.state().cycles.get(id).ok_or_else(cycle_artifact_scope)?;
    let members = cycle.execution_ids.iter().collect::<std::collections::BTreeSet<_>>();
    for record in records {
        let Some(execution_id) = &record.scope.test_execution_id else {
            continue;
        };
        let owner = snapshot.state().executions.get(execution_id);
        if record.scope.test_cycle_id.as_ref() != Some(id) && owner.is_none_or(|owner| &owner.test_cycle_id != id) {
            continue;
        }
        let owner = owner.ok_or_else(cycle_artifact_scope)?;
        if &owner.test_cycle_id != id
            || !members.contains(execution_id)
            || !crate::evidence_runtime::scopes_correlate(Some(&record.scope), Some(&owner.scope))
        {
            return Err(cycle_artifact_scope());
        }
    }
    Ok(())
}

/* === Private Functions === */

fn cycle_artifact_scope() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "document artifact scope",
        reason: "artifact descriptor differs from the captured cycle members",
    }
}

async fn finish(fold: super::frame::Fold, config: crate::config::QueryConfig) -> crate::runtime::Result<Option<Value>> {
    let mut work = QueryWork::document(config, "dokime.documents.summary");
    let _cancel = work.cancellation();
    tokio::task::spawn_blocking(move || {
        work.check()?;
        let result = fold.finish_checked(&mut || {
            work.check().map_err(|_| harmos::storage::Error::ResourceExhausted {
                resource: "document query cancelled or expired",
            })
        });
        work.check()?;
        Ok(result?)
    })
    .await
    .map_err(|_| crate::runtime::Error::Operation("document fold worker failed".into()))?
}

async fn telemetry(
    runtime: &RuntimeHandle, execution_context: super::input::Context<'_>, selection: &DocumentSelection,
    at: &harmos::storage::StorageCursor, expression: crate::expressions::DataExpression, filters: Vec<super::DocumentFilter>,
) -> crate::runtime::Result<Option<Value>> {
    let operation = "dokime.documents.summary";
    let execution = selection
        .scope
        .test_execution_id
        .as_ref()
        .ok_or_else(|| crate::runtime::Error::Operation("summary query requires an execution".into()))?;
    let scope = Scope::Execution(execution.to_string());
    let contracts = runtime
        .local_storage
        .query_telemetry_contracts(scope.clone(), at.sequence, runtime.query_config().limits().schema_bytes as usize, operation)
        .await
        .map_err(query_error)?;
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
        && let Some(value_type) = contracts
            .iter()
            .filter(|contract| contract.stream.as_str() == stream)
            .flat_map(|contract| &contract.columns)
            .find(|column| column.metadata.identifier.as_str() == field)
            .map(|column| &column.value_type)
        && contracts.iter().filter(|contract| contract.stream.as_str() == stream).all(|contract| {
            contract.columns.iter().any(|column| {
                column.metadata.identifier.as_str() == field
                    && &column.value_type == value_type
                    && (matches!(
                        value_type,
                        dokime_domain::model::values::ValueType::Number | dokime_domain::model::values::ValueType::Integer
                    ) || *aggregate == harmos::Aggregate::Count)
            })
        })
        && let Some(value) = runtime
            .local_storage
            .document_statistic(
                execution.to_string(),
                stream.clone(),
                field.clone(),
                at.clone(),
                *aggregate,
                QueryWork::document(runtime.query_config().clone(), operation),
            )
            .await?
    {
        return Ok(value);
    }
    let expires = dokime_domain::model::identity::wire::Timestamp::from(
        time::OffsetDateTime::now_utc()
            + time::Duration::try_from(runtime.query_config().limits().snapshot_lifetime)
                .map_err(|_| crate::runtime::Error::Operation("invalid query lifetime".into()))?,
    );
    let bytes = serde_json::to_vec(&contracts).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
    let cut = runtime
        .local_storage
        .snapshot_capture(scope, Some(at.clone()), None, expires, bytes, None)
        .await?
        .ok_or_else(|| crate::runtime::Error::Operation("document query cut unavailable".into()))?;
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
        let captured = execution_context.capture();
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
                    let context = super::input::Context::captured(&captured, &selected.scope);
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
    finish(state.0, runtime.query_config().clone()).await
}

async fn records(
    runtime: &RuntimeHandle, context: super::input::Context<'_>, selection: &DocumentSelection, at: &harmos::storage::StorageCursor,
    expression: crate::expressions::DataExpression, filters: Vec<super::DocumentFilter>,
) -> crate::runtime::Result<Option<Value>> {
    let execution = selection
        .scope
        .test_execution_id
        .as_ref()
        .ok_or_else(|| crate::runtime::Error::Operation("record summary requires an execution".into()))?;
    let mut fold = super::frame::Fold::new(&expression)?;
    let needed = std::iter::once(&expression)
        .chain(filters.iter().map(|filter| &filter.expression))
        .filter_map(|expression| match expression.transforms.first() {
            Some(DataTransform::TimeBetween { previous_record_type }) => Some(previous_record_type.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut previous = std::collections::BTreeMap::<String, i128>::new();
    let mut partition = String::new();
    let mut after = (String::new(), 0);
    let mut work = QueryWork::document(runtime.query_config().clone(), "dokime.documents.records");
    let _cancel = work.cancellation();
    loop {
        work.check()?;
        let (rows, next) =
            runtime.local_storage.document_records(at.clone(), execution.to_string(), after).await.map_err(|error| match error {
                crate::storage::Error::Runtime(harmos::Error::InvalidInput { field: "document record scope", .. }) => {
                    crate::runtime::Error::conflict("document_capture_incomplete", "record capture scope differs")
                }
                error => error.into(),
            })?;
        let Some(next) = next else { break };
        after = next;
        for row in &rows {
            let record = &row.record;
            let scope = record.captured_scope(execution).or(record.scope.as_ref());
            if selection.foreign_execution_scope(scope) {
                return Err(crate::runtime::Error::conflict("document_capture_incomplete", "record capture scope differs"));
            }
            if !selection.matches_scope(scope) {
                continue;
            }
            work.check()?;
            let attempt = match scope.map(|scope| &scope.origin) {
                Some(dokime_domain::model::execution::EvidenceOrigin::Step { attempt_id }) => attempt_id.as_str(),
                _ => "",
            };
            if partition != attempt {
                previous.clear();
                partition = attempt.to_owned();
            }
            let view = super::input::RecordView { entry: record, scope };
            let evidence = Evidence {
                context,
                selection: Some(selection),
                ..Default::default()
            };
            let value = |expression: &crate::expressions::DataExpression| {
                if let Some(DataTransform::TimeBetween { previous_record_type }) = expression.transforms.first() {
                    let start = previous.get(previous_record_type)?;
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
            if needed.contains(record.record_type.as_str())
                && let Ok(time) = record.timestamp.to_datetime()
            {
                previous.insert(record.record_type.to_string(), time.unix_timestamp_nanos());
            }
        }
    }
    finish(fold, runtime.query_config().clone()).await
}

fn query_error(error: Box<dokime_domain::model::query::error::Failure>) -> crate::runtime::Error {
    crate::runtime::Error::Query(error)
}

/* === Tests === */
