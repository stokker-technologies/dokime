/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Plot Preview Checkpoints At Evidence Commits

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

// Third Party Imports
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Local Imports
use super::{Limits, Preview};
use crate::plot::{PlotPayload, PlotPayloadContext, PlotPayloadRequest};
use crate::storage::Error;
use crate::storage::sqlite::State;
use crate::storage::sqlite::data::telemetry::previews as checkpoint_state;
use crate::telemetry::TelemetryContract;

/* === Constants === */

pub(crate) const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS report_plot_previews (
    execution TEXT NOT NULL, plot TEXT NOT NULL, through TEXT NOT NULL, state BLOB NOT NULL,
    definition BLOB NOT NULL, shared INTEGER NOT NULL,
    PRIMARY KEY(execution,plot)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS report_plot_bindings (
    execution TEXT NOT NULL, plot TEXT NOT NULL, projection TEXT NOT NULL,
    PRIMARY KEY(execution,plot)
) WITHOUT ROWID;";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Serialize, Deserialize)]
struct Plan {
    request: PlotPayloadRequest,
    contracts: Vec<TelemetryContract>,
    limits: Limits,
    context: BTreeMap<String, dokime_domain::model::values::Value>,
    #[serde(skip)]
    preview: Preview,
}

// A Plot Preview Read In The Same Snapshot As The Rest Of A Report's Evidence.
#[derive(Clone)]
pub(crate) struct Checkpoint(Arc<Plan>);

#[derive(Default)]
pub(crate) struct Batch {
    loaded: BTreeSet<String>,
    plans: BTreeMap<(String, String), (u64, Plan)>,
    dirty: BTreeSet<(String, String)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Batch {
    fn load(&mut self, connection: &Connection, execution: &str) -> Result<(), Error> {
        if self.loaded.contains(execution) {
            return Ok(());
        }
        self.loaded.insert(execution.to_owned());
        let mut query = connection
            .prepare("SELECT execution,plot,through,state,definition FROM report_plot_previews WHERE execution=?1 AND shared=0")?;
        let rows = query.query_map([execution], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        for row in rows {
            let (execution, plot, through, bytes, definition) = row?;
            if let std::collections::btree_map::Entry::Vacant(entry) = self.plans.entry((execution, plot)) {
                entry.insert((u64::from_str_radix(&through, 16).map_err(|_| Error::Corrupt("invalid plot preview frontier"))?, {
                    let mut plan = decode_plan(&definition)?;
                    plan.preview = checkpoint_state::decode(&bytes)?;
                    plan
                }));
            }
        }
        Ok(())
    }

    // Telemetry Arrives Already Decoded And Normalized To Its Arrow Contract, Matching What Replay Reads Back.
    #[tracing::instrument(level = "debug", name = "ingest.plot", skip_all)]
    pub(crate) fn sample(
        &mut self, connection: &Connection, row: &harmos::storage::StoredRow, sample: &crate::telemetry::TelemetrySample,
    ) -> Result<(), Error> {
        if !row.record.schema.id.as_str().starts_with("dokime.telemetry.") {
            return Ok(());
        }
        let owners: BTreeSet<_> =
            sample.scope.iter().chain(&sample.captured_scopes).filter_map(|scope| scope.test_execution_id.as_ref()).collect();
        for owner in owners {
            self.load(connection, owner.as_str())?;
        }
        for ((owner, projection), (through, plan)) in &mut self.plans {
            if row.record.position.sequence <= *through {
                continue;
            }
            let Some(scope) = sample
                .scope
                .iter()
                .chain(&sample.captured_scopes)
                .find(|scope| scope.test_execution_id.as_ref().is_some_and(|id| id.as_str() == owner))
            else {
                continue;
            };
            if !crate::plot::plot_streams(&plan.request).iter().any(|stream| crate::plot::stream_matches_alias(row.stream.as_str(), stream))
            {
                continue;
            }
            let mut sample = sample.clone();
            sample.scope = Some(scope.clone());
            for (key, value) in &plan.context {
                sample.values.entry(key.clone()).or_insert_with(|| value.clone());
            }
            plan.preview.sample(sample, &plan.request, plan.limits).map_err(|error| Error::Catalog(error.to_string()))?;
            *through = row.record.position.sequence;
            self.dirty.insert((owner.clone(), projection.clone()));
        }
        Ok(())
    }

    pub(crate) fn record(&mut self, connection: &Connection, row: &harmos::storage::StoredRecord) -> Result<(), Error> {
        if row.schema != crate::storage::records::schema() {
            return Ok(());
        }
        let record = crate::storage::records::RecordRow::decode_entry(&row.payload)?;
        let owners: BTreeSet<_> =
            record.scope.iter().chain(&record.captured_scopes).filter_map(|scope| scope.test_execution_id.as_ref()).collect();
        for owner in owners {
            self.load(connection, owner.as_str())?;
        }
        for ((owner, projection), (through, plan)) in &mut self.plans {
            if row.position.sequence <= *through {
                continue;
            }
            let mut selected = record.clone();
            if let Some(scope) =
                record.captured_scopes.iter().find(|scope| scope.test_execution_id.as_ref().is_some_and(|id| id.as_str() == owner))
            {
                selected.scope = Some(scope.clone());
            } else if !record.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()).is_some_and(|id| id.as_str() == owner) {
                continue;
            }
            if plan
                .request
                .scope
                .as_ref()
                .is_some_and(|scope| !dokime_domain::model::execution::scope_visible_in(selected.scope.as_ref(), scope))
            {
                continue;
            }
            plan.preview.record(selected, &plan.request, plan.limits).map_err(|error| Error::Catalog(error.to_string()))?;
            *through = row.position.sequence;
            self.dirty.insert((owner.clone(), projection.clone()));
        }
        Ok(())
    }

    pub(crate) fn save(self, connection: &Connection) -> Result<(), Error> {
        let mut query = connection.prepare_cached("UPDATE report_plot_previews SET through=?3,state=?4 WHERE execution=?1 AND plot=?2")?;
        for ((execution, plot), (through, plan)) in self.plans {
            if self.dirty.contains(&(execution.clone(), plot.clone())) {
                query.execute(params![execution, plot, format!("{through:016x}"), checkpoint_state::encode(&plan.preview)?])?;
            }
        }
        Ok(())
    }
}

impl State {
    pub(crate) async fn live_plot_previews(
        &self, executions: Vec<String>, request: PlotPayloadRequest, at: harmos::storage::StorageCursor, current: bool,
    ) -> Result<Option<(PlotPayload, harmos::storage::StorageCursor)>, Error> {
        self.read(move |connection| {
            Ok((|| {
                let head = crate::storage::sqlite::data::saved(connection)?;
                head.history.check(&at.history)?;
                let through = if current {
                    harmos::storage::StorageCursor {
                        history: head.history,
                        sequence: head.sequence,
                    }
                } else {
                    at
                };
                let mut combined: Option<Plan> = None;
                let Some(plot) = &request.plot_id else { return Ok(None) };
                let mut query = connection.prepare_cached(
                    "SELECT p.definition,p.state,p.shared FROM report_plot_bindings v JOIN report_plot_previews p
                     ON p.execution=v.execution AND p.plot=v.projection WHERE v.execution=?1 AND v.plot=?2 AND p.through<=?3",
                )?;
                for execution in executions {
                    let bytes: Option<(Vec<u8>, Vec<u8>, bool)> = query
                        .query_row(params![execution, plot, format!("{:016x}", through.sequence)], |row| {
                            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                        })
                        .optional()?;
                    let Some((definition, state, shared)) = bytes else { return Ok(None) };
                    let Some(plan) = load_plan(connection, &execution, through.sequence, &definition, &state, shared)? else {
                        return Ok(None);
                    };
                    let checkpoint = Checkpoint(Arc::new(plan));
                    if !checkpoint.matches(&request) {
                        return Ok(None);
                    }
                    if let Some(combined) = &mut combined {
                        combined
                            .preview
                            .merge(Arc::try_unwrap(checkpoint.0).ok().expect("private live checkpoint").preview, &request, combined.limits)
                            .map_err(|error| Error::Catalog(error.to_string()))?;
                    } else {
                        combined = Some(Arc::try_unwrap(checkpoint.0).ok().expect("private live checkpoint"));
                    }
                }
                combined
                    .map(|checkpoint| {
                        Checkpoint(Arc::new(checkpoint))
                            .payload(&request, crate::model::time::now_utc().as_str(), (None, None))
                            .map(|payload| (payload, through))
                            .map_err(|error| Error::Catalog(error.to_string()))
                    })
                    .transpose()
            })())
        })
        .await?
    }

    pub(crate) async fn register_plot_previews(
        &self, execution: String, requests: Vec<PlotPayloadRequest>, contracts: Vec<TelemetryContract>, limits: Limits,
        context: BTreeMap<String, dokime_domain::model::values::Value>,
    ) -> Result<(), Error> {
        self.call(move |connection| {
            Ok((|| -> Result<(), Error> {
                let transaction = connection.transaction()?;
                for mut request in requests {
                    let Some(plot) = request.plot_id.clone() else { continue };
                    request.plot_id = None;
                    let digest = Sha256::digest(serde_json::to_vec(&request)?);
                    let projection = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                    let streams = crate::plot::plot_streams(&request);
                    let selected: Vec<_> = contracts
                        .iter()
                        .filter(|contract| streams.iter().any(|stream| crate::plot::stream_matches_alias(contract.stream.as_str(), stream)))
                        .cloned()
                        .collect();
                    let shared = uses_shared_fields(&request, &selected);
                    let plan = Plan {
                        request,
                        contracts: selected,
                        limits,
                        context: context.clone(),
                        preview: Preview::default(),
                    };
                    transaction.execute(
                        "INSERT OR IGNORE INTO report_plot_previews VALUES (?1,?2,'0000000000000000',?3,?4,?5)",
                        params![execution, projection, checkpoint_state::encode(&plan.preview)?, serde_json::to_vec(&plan)?, shared],
                    )?;
                    transaction
                        .execute("INSERT OR IGNORE INTO report_plot_bindings VALUES (?1,?2,?3)", params![execution, plot, projection])?;
                }
                transaction.commit()?;
                Ok(())
            })())
        })
        .await?
    }
}

impl Checkpoint {
    fn matches(&self, request: &PlotPayloadRequest) -> bool {
        if !request.scope.as_ref().is_some_and(|scope| {
            self.0.request.scope.as_ref().is_some_and(|saved| dokime_domain::model::execution::scope_visible_in(Some(saved), scope))
        }) {
            return false;
        }
        let shape = |request: &PlotPayloadRequest| {
            // Both Inputs Have Already Undergone The One Query-Time Mapping.
            let mut request = request.clone();
            request.plot_id = None;
            request.scope = None;
            request.max_points = None;
            request.summary_window = None;
            request.technique = Default::default();
            request.x_format = None;
            request.y_format = None;
            serde_json::to_value(request).ok()
        };
        shape(&self.0.request) == shape(request)
    }

    pub(crate) fn payload(
        &self, request: &PlotPayloadRequest, generated_at: &str, epochs: (Option<&str>, Option<&str>),
    ) -> crate::runtime::Result<PlotPayload> {
        let mut saved = self.0.request.clone();
        saved.max_points = request.max_points;
        saved.scope = request.scope.clone();
        saved.summary_window = request.summary_window;
        saved.technique = request.technique;
        saved.x_format = request.x_format.clone().or(saved.x_format);
        saved.y_format = request.y_format.clone().or(saved.y_format);
        let context = PlotPayloadContext {
            default_max_points: saved.max_points.unwrap_or(self.0.limits.exact_points),
            x_format: saved.x_format.as_ref(),
            series_format: saved.y_format.as_ref(),
            run_started_at: epochs.0,
            cycle_started_at: epochs.1,
        };
        let mut payload = self.0.preview.payload(&saved, generated_at, &context, self.0.limits)?;
        decorate(&mut payload, &self.0.contracts, &saved);
        Ok(payload)
    }
}

/* === Public Functions === */

fn uses_shared_fields(request: &PlotPayloadRequest, contracts: &[TelemetryContract]) -> bool {
    let numeric = |field: &str| {
        !field.contains('.')
            && contracts.iter().any(|contract| {
                crate::plot::stream_matches_alias(contract.stream.as_str(), &request.stream)
                    && contract.normalized().columns.iter().any(|column| {
                        column.metadata.identifier.as_str() == field
                            && matches!(
                                column.value_type,
                                dokime_domain::model::values::ValueType::Integer | dokime_domain::model::values::ValueType::Number
                            )
                    })
            })
    };
    request.analysis.is_none()
        && request.filters.is_empty()
        && request.series_by.is_none()
        && request.series_by_expression.is_none()
        && request.z_expression.is_none()
        && request.z_stream.is_none()
        && request.z_field.is_none()
        && request.x_expression.is_none()
        && request.x_key.as_deref().is_none_or(|key| key == "timestamp")
        && request.summary_window.is_none()
        && request.marker_record_types.is_empty()
        && request.scope.as_ref().is_none_or(|scope| scope.step_id.is_none() && scope.criterion_id.is_none())
        && matches!(request.plot_type, None | Some(crate::document::PlotType::TimeSeries))
        && request.y_expressions.iter().all(|expression| {
            matches!(&expression.source, crate::expressions::DataSource::Telemetry { stream, .. }
                if crate::plot::stream_matches_alias(&request.stream, stream))
                && expression.transforms.is_empty()
        })
        && (!request.y_keys.is_empty() || !request.y_expressions.is_empty())
        && request.y_keys.iter().all(|field| numeric(field))
        && request
            .y_expressions
            .iter()
            .all(|expression| matches!(&expression.source,crate::expressions::DataSource::Telemetry { field,.. } if numeric(field)))
}

pub(crate) fn uses_numeric_mean(request: &PlotPayloadRequest) -> bool {
    request
        .analysis
        .as_ref()
        .is_some_and(|selection| selection.id == "dokime.analysis.mean" && selection.version == 1 && selection.parameters.is_empty())
        && request.filters.is_empty()
        && request.series_by.is_none()
        && request.series_by_expression.is_none()
        && request.z_expression.is_none()
        && request.z_stream.is_none()
        && request.z_field.is_none()
        && request.x_expression.is_none()
        && request.x_key.as_deref().is_none_or(|key| key == "timestamp")
        && request.summary_window.is_none()
        && request.scope.as_ref().is_none_or(|scope| scope.step_id.is_none() && scope.criterion_id.is_none())
        && matches!(request.plot_type, None | Some(crate::document::PlotType::TimeSeries))
        && request.y_expressions.iter().all(|expression| {
            matches!(&expression.source, crate::expressions::DataSource::Telemetry { stream, .. }
                if crate::plot::stream_matches_alias(&request.stream, stream))
                && expression.transforms.is_empty()
        })
}

pub(crate) fn decorate(payload: &mut PlotPayload, contracts: &[TelemetryContract], request: &PlotPayloadRequest) {
    let x_field = request
        .x_expression
        .as_ref()
        .and_then(|expression| match &expression.source {
            crate::expressions::DataSource::Telemetry { field, .. } => Some(field.as_str()),
            _ => None,
        })
        .or(request.x_key.as_deref())
        .unwrap_or("timestamp");
    let column = contracts
        .iter()
        .filter(|contract| crate::plot::stream_matches_alias(contract.stream.as_str(), &request.stream))
        .flat_map(|contract| &contract.columns)
        .find(|column| column.metadata.identifier.as_str() == x_field);
    payload.x_format = crate::plot::resolved_format(request.x_format.as_ref(), column).cloned();
    for series in &mut payload.series {
        let field = request
            .y_expressions
            .iter()
            .find_map(|expression| {
                let crate::expressions::DataSource::Telemetry { field, .. } = &expression.source else { return None };
                (request.y_expressions.len() == 1 || field == &series.key || expression.label.as_ref() == Some(&series.key))
                    .then_some(field.as_str())
            })
            .or_else(|| (request.y_keys.len() == 1).then(|| request.y_keys[0].as_str()))
            .unwrap_or_else(|| series.key.split(':').next().unwrap_or(&series.key));
        let column = contracts
            .iter()
            .filter(|contract| crate::plot::stream_matches_alias(contract.stream.as_str(), &request.stream))
            .flat_map(|contract| &contract.columns)
            .find(|column| column.metadata.identifier.as_str() == field);
        series.unit = request
            .unit
            .clone()
            .or_else(|| column.and_then(|column| column.unit.as_ref()).map(|unit| unit.symbol.clone()))
            .or(series.unit.take());
        series.format = crate::plot::resolved_format(request.y_format.as_ref(), column).cloned();
        if request.series_by.is_none()
            && request.series_by_expression.is_none()
            && let Some(expression) = request.y_expressions.iter().find(
                |expression| matches!(&expression.source, crate::expressions::DataSource::Telemetry { field: key, .. } if key == field),
            )
            && let Some(label) = &expression.label
        {
            series.label = label.clone();
        }
    }
}

/* === Private Functions === */

fn decode_plan(bytes: &[u8]) -> Result<Plan, Error> {
    serde_json::from_slice(bytes).map_err(|error| {
        Error::Incompatible(format!(
            "Plot preview layout is incompatible; use a fresh storage root and preserve the existing evidence: {error}"
        ))
    })
}

// Checkpoints That Advanced Past The Cut Are Omitted So Their Plots Rebuild Through It.
pub(crate) fn checkpoints(connection: &Connection, execution: &str, at: u64) -> Result<BTreeMap<String, Checkpoint>, Error> {
    let mut query = connection.prepare(
        "SELECT v.plot,p.through,p.state,p.definition,p.shared,p.plot
        FROM report_plot_bindings v JOIN report_plot_previews p ON p.execution=v.execution AND p.plot=v.projection WHERE v.execution=?1",
    )?;
    let mut rows = query.query([execution])?;
    let mut checkpoints = BTreeMap::new();
    let mut projections = BTreeMap::<String, Checkpoint>::new();
    while let Some(row) = rows.next()? {
        let through: String = row.get(1)?;
        let through = u64::from_str_radix(&through, 16).map_err(|_| Error::Corrupt("invalid plot preview frontier"))?;
        if through > at {
            continue;
        }
        let bytes: Vec<u8> = row.get(2)?;
        let projection: String = row.get(5)?;
        let checkpoint = if let Some(checkpoint) = projections.get(&projection) {
            checkpoint.clone()
        } else {
            let definition: Vec<u8> = row.get(3)?;
            let Some(plan) = load_plan(connection, execution, at, &definition, &bytes, row.get(4)?)? else { continue };
            let checkpoint = Checkpoint(Arc::new(plan));
            projections.insert(projection, checkpoint.clone());
            checkpoint
        };
        checkpoints.insert(row.get(0)?, checkpoint);
    }
    Ok(checkpoints)
}

fn load_plan(
    connection: &Connection, execution: &str, at: u64, definition: &[u8], state: &[u8], shared: bool,
) -> Result<Option<Plan>, Error> {
    let mut plan = decode_plan(definition)?;
    if shared {
        let advanced: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM storage_report_previews WHERE execution=?1 AND through>?2)",
            params![execution, format!("{at:016x}")],
            |row| row.get(0),
        )?;
        if advanced {
            return Ok(None);
        }
        let head = crate::storage::sqlite::data::saved(connection)?;
        let cut = harmos::storage::StorageCursor { history: head.history, sequence: at };
        let fields = checkpoint_state::report_previews(connection, execution.into(), cut)?;
        let selected = |field: &str| {
            plan.request.y_keys.iter().any(|key| key == field)
                || plan.request.y_expressions.iter().any(
                    |expression| matches!(&expression.source,crate::expressions::DataSource::Telemetry { field:key,.. } if key == field),
                )
        };
        if fields.iter().any(|field| {
            crate::plot::stream_matches_alias(&field.stream, &plan.request.stream)
                && selected(&field.field)
                && field.total <= plan.limits.exact_points as u64
                && field.exact.len() as u64 != field.total
        }) {
            return Ok(None);
        }
        let summaries = crate::storage::sqlite::data::telemetry::aggregates::numeric_summaries(connection, execution.into())?;
        plan.preview =
            Preview::shared(&fields, &summaries, &plan.request, plan.limits).map_err(|error| Error::Catalog(error.to_string()))?;
        plan.preview.context.extend(checkpoint_state::report_context(connection, execution, &plan.request.stream, at)?);
        for (key, value) in &plan.context {
            plan.preview.context.entry(key.clone()).or_insert_with(|| value.clone());
        }
    } else {
        plan.preview = checkpoint_state::decode(state)?;
    }
    Ok(Some(plan))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn duplicate_views_share_static_definition_and_clean_state() {
        let root = tempfile::tempdir().unwrap();
        let state = State::open(root.path()).await.unwrap();
        state.call(|connection| connection.execute_batch(SCHEMA)).await.unwrap();
        for (name, copies) in [("single", 1), ("many", 64)] {
            let requests: Vec<_> = (0..copies)
                .map(|index| PlotPayloadRequest {
                    plot_id: Some(format!("plot.{index}")),
                    stream: "stream".into(),
                    x_key: Some("value".into()),
                    y_keys: vec!["value".into()],
                    plot_type: Some(crate::document::PlotType::XyScatter),
                    ..Default::default()
                })
                .collect();
            state
                .register_plot_previews(
                    name.into(),
                    requests,
                    Vec::new(),
                    crate::config::local_storage_defaults().previews.into(),
                    Default::default(),
                )
                .await
                .unwrap();
            let execution = name.to_owned();
            let (plans, views, changed) = state
                .call(move |connection| {
                    Ok((|| -> Result<_, Error> {
                        let plans: i64 =
                            connection.query_row("SELECT count(*) FROM report_plot_previews WHERE execution=?1", [&execution], |row| {
                                row.get(0)
                            })?;
                        let views: i64 =
                            connection.query_row("SELECT count(*) FROM report_plot_bindings WHERE execution=?1", [&execution], |row| {
                                row.get(0)
                            })?;
                        let before = connection.total_changes();
                        let mut batch = Batch::default();
                        batch.load(connection, &execution)?;
                        batch.save(connection)?;
                        Ok((plans, views, connection.total_changes() - before))
                    })())
                })
                .await
                .unwrap()
                .unwrap();
            assert_eq!(views, copies);
            assert_eq!(plans, views.min(1));
            assert_eq!(changed, 0);
        }
        state.close().await.unwrap();
    }

    #[test]
    fn checkpoint_selection() {
        let scope = dokime_domain::model::execution::RecordScope {
            test_execution_id: Some(dokime_domain::model::identity::TestExecutionId::from_seed("selection")),
            ..Default::default()
        };
        let request = PlotPayloadRequest {
            scope: Some(scope),
            y_keys: vec!["value".into()],
            ..Default::default()
        };
        let checkpoint = Checkpoint(Arc::new(Plan {
            request: request.clone(),
            contracts: Vec::new(),
            limits: crate::config::local_storage_defaults().previews.into(),
            context: BTreeMap::new(),
            preview: Preview::default(),
        }));
        let mut changed = request.clone();
        changed.max_points = Some(17);
        changed.summary_window = Some(crate::plot::PlotWindow { from: 1.0, to: 2.0 });
        assert!(checkpoint.matches(&changed));
        changed.y_keys = vec!["other".into()];
        assert!(!checkpoint.matches(&changed));
        changed = request;
        changed.scope.as_mut().unwrap().test_execution_id = Some(dokime_domain::model::identity::TestExecutionId::from_seed("other"));
        assert!(!checkpoint.matches(&changed));
    }

    #[test]
    fn frozen_checkpoint() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        let mut plan = Plan {
            request: PlotPayloadRequest::default(),
            contracts: Vec::new(),
            limits: crate::config::local_storage_defaults().previews.into(),
            context: BTreeMap::new(),
            preview: Preview::default(),
        };
        plan.preview.context.insert("wifi_channel".into(), dokime_domain::model::values::Value::Integer(6));
        connection
            .execute(
                "INSERT INTO report_plot_previews VALUES ('execution','projection','000000000000000a',?1,?2,0)",
                params![checkpoint_state::encode(&plan.preview).unwrap(), serde_json::to_vec(&plan).unwrap()],
            )
            .unwrap();
        connection.execute("INSERT INTO report_plot_bindings VALUES ('execution','plot','projection')", []).unwrap();
        assert!(checkpoints(&connection, "execution", 9).unwrap().is_empty());
        let current = checkpoints(&connection, "execution", 10).unwrap().remove("plot").unwrap();
        assert_eq!(current.0.preview.context, plan.preview.context);
        assert!(checkpoints(&connection, "other", 10).unwrap().is_empty());
        let mut legacy = serde_json::json!({"preview":&plan.preview});
        legacy["preview"].as_object_mut().unwrap().remove("marker_records");
        assert!(matches!(
            checkpoint_state::decode::<Preview>(&checkpoint_state::encode(&legacy["preview"]).unwrap()),
            Err(Error::Incompatible(_))
        ));
    }
}
