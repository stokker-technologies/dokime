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
use std::collections::BTreeMap;
// Third Party Imports
use harmos::{
    MemorySize,
    operations::Context,
    storage::{
        QueryValue,
        query::transform::{
            FieldPath, Transform,
            downsample::{Algorithm, Downsample},
            sort::{Sort, SortKey},
        },
    },
};
// Local Imports
use super::{invalid, nonzero, statistics, telemetry};
use crate::{
    Result,
    document::PlotType,
    execution::TestExecution,
    execution::context::source::CapturedSource,
    plot::{
        DownsampleTechnique, PlotAnalysis, PlotPayload, PlotPayloadContext, PlotPayloadRequest,
    },
};
use dokime::{
    core::Identifier, expressions::DataSource, records::RecordEntry, storage::Manager,
    telemetry::TelemetryContract,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(in crate::report) async fn materialize(
    ctx: &Context<'_>,
    storage: &Manager,
    settings: &crate::config::StorageSettings,
    source: &CapturedSource,
    execution: &TestExecution,
    records: &[RecordEntry],
    request: PlotPayloadRequest,
) -> Result<QueryValue<crate::report::value::Value>> {
    let context = PlotPayloadContext {
        x_format: request.x_format.clone(),
        series_format: request.y_format.clone(),
        run_started_at: execution.started_at.as_ref().map(ToString::to_string),
        cycle_started_at: source
            .context
            .cycle
            .cycle
            .started_at
            .as_ref()
            .map(ToString::to_string),
    };
    let points = request
        .max_points
        .unwrap_or(dokime::document::DEFAULT_PLOT_MAX_POINTS);
    let series = request.y_keys.len().max(request.y_expressions.len()).max(1);
    let request_bytes = crate::report::value::Value::serialization_memory(&request)?;
    let record_bytes = records
        .iter()
        .fold(std::mem::size_of::<Vec<RecordEntry>>(), |bytes, record| {
            bytes.saturating_add(record.memory_size())
        });
    let construction = points
        .checked_mul(series)
        .and_then(|points| {
            points.checked_mul(std::mem::size_of::<crate::plot::PlotPoint>() * 4 + 128)
        })
        .and_then(|bytes| {
            bytes.checked_add(
                request
                    .histogram
                    .bins
                    .saturating_mul(std::mem::size_of::<crate::plot::HistogramBin>() * 4),
            )
        })
        .and_then(|bytes| bytes.checked_add(record_bytes.saturating_mul(2)))
        .and_then(|bytes| bytes.checked_add(request_bytes.saturating_mul(4)))
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or_else(super::exhausted)?;
    let _construction = storage.prepare_production(nonzero(construction)?)?;
    let mut raw_working = None;
    let payload = match direct(&request, &source.telemetry_contracts) {
        Some((request, contract)) => {
            optimized(
                super::ReadContext {
                    ctx,
                    storage,
                    settings,
                },
                source,
                execution,
                records,
                &request,
                contract,
                &context,
            )
            .await?
        }
        None => {
            let streams = crate::plot::plot_streams(&request);
            let samples = telemetry::streams(
                ctx,
                storage,
                settings,
                &source.evidence_through,
                execution.test_execution_id.as_str(),
                &source.telemetry_contracts,
                &streams,
            )
            .await?;
            let working = samples
                .memory_size()
                .checked_mul(4)
                .and_then(|bytes| bytes.checked_add(record_bytes.saturating_mul(2)))
                .and_then(|bytes| bytes.checked_add(4096))
                .ok_or_else(super::exhausted)?;
            raw_working = Some(storage.prepare_production(nonzero(working)?)?);
            if request.analysis.is_some() {
                let projected = crate::plot::project_plot_payload(
                    &samples,
                    records,
                    &request,
                    source.captured_at.as_str(),
                    &context,
                );
                let contract = source
                    .telemetry_contracts
                    .get(
                        &Identifier::new(&request.stream)
                            .map_err(|_| invalid("invalid plot stream"))?,
                    )
                    .and_then(|versions| versions.values().next());
                crate::report::analysis::materialize(ctx, &request, &samples, projected, contract)
                    .await?
            } else {
                crate::plot::build_plot_payload(
                    &samples,
                    records,
                    request,
                    source.captured_at.as_str(),
                    &context,
                )
            }
        }
    };
    let _raw_working = raw_working;
    let bound = crate::report::value::Value::serialization_memory(&payload)?;
    Ok(storage
        .produce(nonzero(bound)?, |allowance| {
            crate::report::value::Value::from_serializable(&payload, allowance)
                .map_err(super::super::materialize::codec)
        })?
        .try_into_value()?)
}

/* === Private Functions === */
fn direct<'a>(
    request: &PlotPayloadRequest,
    contracts: &'a BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>,
) -> Option<(PlotPayloadRequest, &'a TelemetryContract)> {
    if request.analysis.is_some()
        || request.series_by.is_some()
        || request.series_by_expression.is_some()
        || request.z_expression.is_some()
        || request.z_stream.is_some()
        || request.z_field.is_some()
        || !request.filters.is_empty()
        || request.window_s.is_some()
        || request.summary_window.is_some()
        || request
            .scope
            .as_ref()
            .is_some_and(|scope| scope.step_id.is_some() || scope.criterion_id.is_some())
        || !matches!(
            request.plot_type,
            Some(
                PlotType::TimeSeries | PlotType::XyScatter | PlotType::XyLine | PlotType::Histogram
            )
        )
    {
        return None;
    }
    let versions = contracts
        .iter()
        .find(|(stream, _)| stream.as_str() == request.stream)?
        .1;
    if versions.len() != 1 {
        return None;
    }
    let contract = versions.values().next()?;
    let mut canonical = request.clone();
    let field = |expression: &dokime::expressions::DataExpression| match &expression.source {
        DataSource::Telemetry { stream, field }
            if stream == &request.stream && expression.transforms.is_empty() =>
        {
            Some(field.clone())
        }
        _ => None,
    };
    if canonical
        .y_expressions
        .iter()
        .any(|expression| expression.label.is_some())
    {
        return None;
    }
    if !canonical.y_expressions.is_empty() {
        canonical.y_keys = canonical
            .y_expressions
            .iter()
            .map(field)
            .collect::<Option<Vec<_>>>()?;
        canonical.y_expressions.clear();
    }
    if let Some(expression) = canonical.x_expression.as_ref() {
        canonical.x_key = Some(field(expression)?);
        canonical.x_expression = None;
    }
    if canonical.y_keys.is_empty() {
        return None;
    }
    Some((canonical, contract))
}

async fn optimized(
    read: super::ReadContext<'_, '_>,
    source: &CapturedSource,
    execution: &TestExecution,
    records: &[RecordEntry],
    request: &PlotPayloadRequest,
    contract: &TelemetryContract,
    context: &PlotPayloadContext,
) -> Result<PlotPayload> {
    let super::ReadContext {
        ctx,
        storage,
        settings,
    } = read;
    let at = &source.evidence_through;
    let scope = execution.test_execution_id.as_str();
    let time_axis = request
        .x_key
        .as_deref()
        .is_none_or(|key| key == "timestamp");
    let x = if time_axis {
        FieldPath {
            fields: vec!["_storage".into(), "source_time".into()],
        }
    } else {
        statistics::field(request.x_key.as_deref().unwrap())
    };
    let mut payload = crate::plot::build_plot_payload(
        &[],
        records,
        request.clone(),
        source.captured_at.as_str(),
        context,
    );
    payload.series.clear();
    payload.summaries.clear();
    let mut summaries = Vec::new();
    let mut histograms = Vec::new();
    for key in &request.y_keys {
        let statistics = statistics::collect(read, at, scope, contract, key, x.clone()).await?;
        // Missing Values Preserve Existing Gap Semantics Through The Raw Path.
        if statistics.count != statistics.rows && request.plot_type != Some(PlotType::Histogram) {
            let samples =
                telemetry::read(ctx, storage, settings, at, scope, contract, vec![]).await?;
            return Ok(crate::plot::build_plot_payload(
                &samples,
                records,
                request.clone(),
                source.captured_at.as_str(),
                context,
            ));
        }
        let maximum = request
            .max_points
            .unwrap_or(dokime::document::DEFAULT_PLOT_MAX_POINTS);
        let mut transforms = vec![Transform::Sort(Sort {
            keys: vec![SortKey {
                field: x.clone(),
                descending: false,
                nulls_first: false,
            }],
        })];
        if maximum > 0 {
            let algorithm = match request.technique {
                DownsampleTechnique::Raw => Algorithm::Decimate,
                DownsampleTechnique::MinMax if maximum >= 2 => Algorithm::Extrema,
                DownsampleTechnique::Lttb if maximum >= 3 => Algorithm::Lttb,
                DownsampleTechnique::Auto | DownsampleTechnique::Unspecified
                    if maximum >= 3 && statistics.count <= maximum.saturating_mul(4) =>
                {
                    Algorithm::Lttb
                }
                DownsampleTechnique::Auto | DownsampleTechnique::Unspecified if maximum >= 2 => {
                    Algorithm::Extrema
                }
                _ => Algorithm::Decimate,
            };
            // Histogram Bins And Statistics Are Independent Full-Data Queries.
            // Its Display Trace May Use Decimation Even When Values Are Missing.
            let algorithm = if request.plot_type == Some(PlotType::Histogram) {
                Algorithm::Decimate
            } else {
                algorithm
            };
            transforms.push(Transform::Downsample(Downsample {
                max_rows: maximum,
                algorithm,
                value: Some(statistics::field(key)),
                axis: Some(x.clone()),
            }));
        }
        let samples =
            telemetry::read(ctx, storage, settings, at, scope, contract, transforms).await?;
        let mut single = request.clone();
        single.y_keys = vec![key.clone()];
        let one = crate::plot::build_plot_payload(
            &samples,
            records,
            single,
            source.captured_at.as_str(),
            context,
        );
        for mut series in one.series {
            series.raw_points = statistics.count;
            series.downsampled = series.points.len() < statistics.count;
            payload.series.push(series);
        }
        if matches!(
            request.plot_type,
            Some(PlotType::TimeSeries | PlotType::XyScatter)
        ) {
            summaries.push(statistics.summary(key.clone()));
        }
        if request.plot_type == Some(PlotType::Histogram) {
            histograms.push(
                statistics::histogram(
                    read,
                    at,
                    scope,
                    contract,
                    key,
                    request.histogram.bins,
                    &statistics,
                )
                .await?,
            );
        }
    }
    payload = crate::plot::finish_plot_payload(payload, request);
    payload.summaries = summaries;
    for series in &mut payload.series {
        series.downsampled = series.points.len() < series.raw_points;
    }
    if request.plot_type == Some(PlotType::Histogram) {
        payload.analysis = Some(PlotAnalysis::Histogram { series: histograms });
    }
    Ok(payload)
}

/* === Tests === */
