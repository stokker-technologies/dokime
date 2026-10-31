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

// Third Party Imports
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

// Local Imports
use super::{Limits, Preview};
use crate::plot::{PlotPayload, PlotPayloadContext, PlotPayloadRequest};
use crate::storage::Error;
use crate::storage::sqlite::State;
use crate::telemetry::TelemetryContract;

/* === Constants === */

pub(crate) const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS report_plot_previews (
    execution TEXT NOT NULL, plot TEXT NOT NULL, through TEXT NOT NULL, state BLOB NOT NULL,
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
    preview: Preview,
}

#[derive(Default)]
pub(crate) struct Batch {
    loaded: BTreeSet<String>,
    plans: BTreeMap<(String, String), (u64, Plan)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Batch {
    fn load(&mut self, connection: &Connection, execution: &str) -> Result<(), Error> {
        if !self.loaded.insert(execution.to_owned()) {
            return Ok(());
        }
        let mut query = connection.prepare("SELECT execution,plot,through,state FROM report_plot_previews WHERE execution=?1")?;
        let rows = query.query_map([execution], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, Vec<u8>>(3)?))
        })?;
        for row in rows {
            let (execution, plot, through, bytes) = row?;
            if let std::collections::btree_map::Entry::Vacant(entry) = self.plans.entry((execution, plot)) {
                entry.insert((
                    u64::from_str_radix(&through, 16).map_err(|_| Error::Corrupt("invalid plot preview frontier"))?,
                    serde_json::from_slice(&bytes)?,
                ));
            }
        }
        Ok(())
    }

    // Telemetry Arrives Already Decoded And Normalized To Its Arrow Contract, Matching What Replay Reads Back.
    pub(crate) fn sample(
        &mut self, connection: &Connection, row: &harmos::storage::StoredRow, sample: &crate::telemetry::TelemetrySample,
    ) -> Result<(), Error> {
        let Some(execution) = row.record.scope.as_ref() else { return Ok(()) };
        if !row.record.schema.id.as_str().starts_with("dokime.telemetry.") {
            return Ok(());
        }
        self.load(connection, execution.as_str())?;
        for ((owner, _), (through, plan)) in &mut self.plans {
            if owner != execution.as_str() || row.record.position.sequence <= *through {
                continue;
            }
            if !crate::plot::plot_streams(&plan.request).iter().any(|stream| crate::plot::stream_matches_alias(row.stream.as_str(), stream))
            {
                continue;
            }
            let mut sample = sample.clone();
            for (key, value) in &plan.context {
                sample.values.entry(key.clone()).or_insert_with(|| value.clone());
            }
            plan.preview.sample(sample, &plan.request, plan.limits).map_err(|error| Error::Catalog(error.to_string()))?;
            *through = row.record.position.sequence;
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
        for ((owner, _), (through, plan)) in &mut self.plans {
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
            if !crate::plot::sample_matches_scope(selected.scope.as_ref(), plan.request.scope.as_ref()) {
                continue;
            }
            plan.preview.record(selected, &plan.request, plan.limits).map_err(|error| Error::Catalog(error.to_string()))?;
            *through = row.position.sequence;
        }
        Ok(())
    }

    pub(crate) fn save(self, connection: &Connection) -> Result<(), Error> {
        let mut query = connection.prepare_cached("UPDATE report_plot_previews SET through=?3,state=?4 WHERE execution=?1 AND plot=?2")?;
        for ((execution, plot), (through, plan)) in self.plans {
            query.execute(params![execution, plot, format!("{through:016x}"), serde_json::to_vec(&plan)?])?;
        }
        Ok(())
    }
}

impl State {
    pub(crate) async fn register_plot_previews(
        &self, execution: String, requests: Vec<PlotPayloadRequest>, contracts: Vec<TelemetryContract>, limits: Limits,
        context: BTreeMap<String, dokime_domain::model::values::Value>,
    ) -> Result<(), Error> {
        self.call(move |connection| {
            Ok((|| -> Result<(), Error> {
                let transaction = connection.transaction()?;
                for request in requests {
                    let Some(plot) = request.plot_id.clone() else { continue };
                    let streams = crate::plot::plot_streams(&request);
                    let selected = contracts
                        .iter()
                        .filter(|contract| streams.iter().any(|stream| crate::plot::stream_matches_alias(contract.stream.as_str(), stream)))
                        .cloned()
                        .collect();
                    let plan = Plan {
                        request,
                        contracts: selected,
                        limits,
                        context: context.clone(),
                        preview: Preview::default(),
                    };
                    transaction.execute(
                        "INSERT OR IGNORE INTO report_plot_previews VALUES (?1,?2,'0000000000000000',?3)",
                        params![execution, plot, serde_json::to_vec(&plan)?],
                    )?;
                }
                transaction.commit()?;
                Ok(())
            })())
        })
        .await?
    }

    pub(crate) async fn plot_preview(
        &self, execution: String, plot: String, at: harmos::storage::StorageCursor, request: PlotPayloadRequest, generated_at: String,
        epochs: (Option<String>, Option<String>),
    ) -> crate::runtime::Result<Option<PlotPayload>> {
        if uses_numeric_mean(&request) {
            return Ok(None);
        }
        let head = self.data_saved().await?;
        head.history.check(&at.history)?;
        if at.sequence > head.sequence {
            return Err(Error::Data("plot preview exceeds the saved evidence frontier").into());
        }
        let state = self.read(move |connection| Ok(checkpoint(connection, &execution, &plot, at.sequence))).await??;
        let Some(state) = state else { return Ok(None) };
        let mut saved_request = state.request.clone();
        saved_request.max_points = request.max_points;
        let request = saved_request;
        let context = PlotPayloadContext {
            default_max_points: request.max_points.unwrap_or(state.limits.exact_points),
            x_format: request.x_format.as_ref(),
            series_format: request.y_format.as_ref(),
            run_started_at: epochs.0.as_deref(),
            cycle_started_at: epochs.1.as_deref(),
        };
        let mut payload = state.preview.payload(&request, &generated_at, &context, state.limits)?;
        decorate(&mut payload, &state.contracts, &request);
        Ok(Some(payload))
    }
}

/* === Public Functions === */

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

fn checkpoint(connection: &Connection, execution: &str, plot: &str, at: u64) -> Result<Option<Plan>, Error> {
    let mut query = connection.prepare("SELECT through,state FROM report_plot_previews WHERE execution=?1 AND plot=?2")?;
    let mut rows = query.query(params![execution, plot])?;
    let Some(row) = rows.next()? else { return Ok(None) };
    let through: String = row.get(0)?;
    let through = u64::from_str_radix(&through, 16).map_err(|_| Error::Corrupt("invalid plot preview frontier"))?;
    if through > at {
        return Ok(None);
    }
    let bytes: Vec<u8> = row.get(1)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

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
                "INSERT INTO report_plot_previews VALUES ('execution','plot','000000000000000a',?1)",
                [serde_json::to_vec(&plan).unwrap()],
            )
            .unwrap();
        assert!(checkpoint(&connection, "execution", "plot", 9).unwrap().is_none());
        let current = checkpoint(&connection, "execution", "plot", 10).unwrap().unwrap();
        assert_eq!(current.preview.context, plan.preview.context);
        assert!(checkpoint(&connection, "other", "plot", 10).unwrap().is_none());
    }
}
