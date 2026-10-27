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
use std::collections::BTreeSet;

// Third Party Imports

// Local Imports
use crate::evidence_runtime::ScopeSelection;
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::plot::PlotPayloadRequest;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn canonical_plot_request(mut request: PlotPayloadRequest, telemetry_streams: &[Identifier]) -> PlotPayloadRequest {
    if !telemetry_streams.iter().any(|stream| stream.as_str() == request.stream)
        && telemetry_streams.len() == 1
        && let Some(stream) = telemetry_streams.iter().next()
    {
        request.stream = stream.as_str().to_owned();
    }

    crate::plot::canonical_plot_query(request)
}

pub(super) fn request_record_types(request: &PlotPayloadRequest) -> Vec<Identifier> {
    let mut record_types =
        request.marker_record_types.iter().filter_map(|record_type| Identifier::new(record_type.clone()).ok()).collect::<BTreeSet<_>>();

    for expression in request_expressions(request) {
        collect_record_expression_types(expression, &mut record_types);
    }

    record_types.into_iter().collect()
}

pub(crate) fn request_telemetry_streams(request: &PlotPayloadRequest) -> Vec<Identifier> {
    let mut streams = BTreeSet::new();
    if !request.y_keys.is_empty() || request.x_key.is_some() || request.series_by.is_some() || request.z_field.is_some() {
        insert_identifier(&mut streams, &request.stream);
    }
    if let Some(stream) = &request.z_stream {
        insert_identifier(&mut streams, stream);
    }
    for expression in request_expressions(request) {
        for stream in expression_telemetry_streams(expression) {
            insert_identifier(&mut streams, &stream);
        }
    }

    streams.into_iter().collect()
}

pub(super) fn request_needs_dimension_context(request: &PlotPayloadRequest) -> bool {
    request_expressions(request).any(expression_needs_dimension_context)
}

pub(super) fn scope_selection(request: &PlotPayloadRequest) -> ScopeSelection {
    request.scope.as_ref().map_or_else(ScopeSelection::default, |scope| ScopeSelection {
        test_cycle_id: scope.test_cycle_id.clone(),
        test_execution_id: scope.test_execution_id.clone(),
        test_case_id: scope.test_case_id.clone(),
        template_id: scope.template_id.clone(),
    })
}

/* === Private Functions === */

fn expression_telemetry_streams(expression: &DataExpression) -> Vec<String> {
    let mut streams = Vec::new();
    if let DataSource::Telemetry { stream, .. } = &expression.source {
        streams.push(stream.clone());
    }
    for transform in &expression.transforms {
        if let DataTransform::TelemetryAtOrBefore { stream, .. } = transform {
            streams.push(stream.clone());
        }
    }
    streams
}

fn insert_identifier(streams: &mut BTreeSet<Identifier>, stream: &str) {
    if let Ok(identifier) = Identifier::new(stream.to_owned()) {
        streams.insert(identifier);
    }
}

fn collect_record_expression_types(expression: &DataExpression, record_types: &mut BTreeSet<Identifier>) {
    let DataSource::Record { record_type } = &expression.source else {
        return;
    };
    if let Ok(identifier) = Identifier::new(record_type.clone()) {
        record_types.insert(identifier);
    }
    for transform in &expression.transforms {
        if let DataTransform::TimeBetween { previous_record_type } = transform
            && let Ok(identifier) = Identifier::new(previous_record_type.clone())
        {
            record_types.insert(identifier);
        }
    }
}

fn request_expressions(request: &PlotPayloadRequest) -> impl Iterator<Item = &DataExpression> {
    request
        .x_expression
        .iter()
        .chain(request.y_expressions.iter())
        .chain(request.z_expression.iter())
        .chain(request.series_by_expression.iter())
        .chain(request.filters.iter().map(|filter| &filter.expression))
}

fn expression_needs_dimension_context(expression: &DataExpression) -> bool {
    matches!(&expression.source, DataSource::Dimension { .. } | DataSource::Parameter { .. })
}

/* === Tests === */
