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

// Third Party Imports
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyDict, PyDictMethods},
};
// Local Imports
use super::convert::*;
use crate::{pyserde, to_value_error};
use dokime_app::core::{Identifier, Metadata, ValueFormat};
use dokime_app::document::{
    DocumentCard, DocumentCardBody, DocumentCardLayout, DocumentFilter, PlotDefinition,
    ReferenceBand, ReferenceLine, validate_plot_definition,
};
use dokime_app::expressions::DataExpression;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub(crate) fn document_plot_contract(
    py: Python<'_>,
    label: String,
    plot_type: String,
    identifier: Option<String>,
    stream: Option<String>,
    x: Option<String>,
    y: Vec<String>,
    z_stream: Option<String>,
    z_field: Option<String>,
    series_by: Option<String>,
    x_expression: Bound<'_, PyAny>,
    y_expressions: Bound<'_, PyAny>,
    z_expression: Bound<'_, PyAny>,
    series_by_expression: Bound<'_, PyAny>,
    filters: Bound<'_, PyAny>,
    group: Option<String>,
    unit: Option<String>,
    style: Option<String>,
    x_scale: Option<String>,
    y_scale: Option<String>,
    fft: Bound<'_, PyAny>,
    interval: Bound<'_, PyAny>,
    histogram: Bound<'_, PyAny>,
    marker_record_types: Vec<String>,
    reference_lines: Bound<'_, PyAny>,
    reference_bands: Bound<'_, PyAny>,
    live_window_s: Option<f64>,
    x_format: Bound<'_, PyAny>,
    y_format: Bound<'_, PyAny>,
    analysis: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let identifier = identifier.unwrap_or_else(|| identifier_from_label("plot", &label));
    let plot = PlotDefinition {
        automatic_id: false,
        metadata: Metadata::new(
            Identifier::new(identifier).map_err(to_value_error)?,
            label,
            None,
        ),
        plot_type: plot_type_from_str(&plot_type)?,
        stream,
        x,
        y,
        z_stream,
        z_field,
        series_by,
        x_expression: optional_py_contract(&x_expression)?,
        y_expressions: pyserde::from_py::<Vec<DataExpression>>(&y_expressions)?,
        z_expression: optional_py_contract(&z_expression)?,
        series_by_expression: optional_py_contract(&series_by_expression)?,
        filters: authored_filters(&filters)?,
        group: group.unwrap_or_else(|| "Other".to_owned()),
        unit,
        style: plot_style_from_str(style.as_deref().unwrap_or("line"))?,
        x_scale: axis_scale_from_str(x_scale.as_deref().unwrap_or("linear"))?,
        y_scale: axis_scale_from_str(y_scale.as_deref().unwrap_or("linear"))?,
        fft: optional_py_contract(&fft)?.unwrap_or_default(),
        analysis: authored_analysis(&analysis)?,
        interval: optional_py_contract(&interval)?.unwrap_or_default(),
        histogram: optional_py_contract(&histogram)?.unwrap_or_default(),
        marker_record_types,
        reference_lines: pyserde::from_py::<Vec<ReferenceLine>>(&reference_lines)?,
        reference_bands: pyserde::from_py::<Vec<ReferenceBand>>(&reference_bands)?,
        live_window_s: live_window_s.or(Some(120.0)),
        x_format: optional_py_contract::<ValueFormat>(&x_format)?,
        y_format: optional_py_contract::<ValueFormat>(&y_format)?,
        // A Python-authored plot declares no axis of its own; the suite it is
        // installed with resolves one from the column the x axis reads.
        x_axis: None,
    };

    validate_plot_definition(&plot).map_err(PyValueError::new_err)?;
    pyserde::to_py(
        py,
        &DocumentCard {
            metadata: plot.metadata.clone(),
            layout: DocumentCardLayout::default(),
            filters: Vec::new(),
            body: DocumentCardBody::Plot {
                plot: Box::new(plot),
            },
        },
    )
}

/* === Private Functions === */

fn authored_filters(filters: &Bound<'_, PyAny>) -> PyResult<Vec<DocumentFilter>> {
    filters
        .try_iter()?
        .map(|filter| {
            let filter = filter?;
            let fields = filter.cast::<PyDict>()?;
            let expression = fields
                .get_item("expression")?
                .ok_or_else(|| PyValueError::new_err("plot filter requires an expression"))?;
            let operator = fields
                .get_item("operator")?
                .ok_or_else(|| PyValueError::new_err("plot filter requires an operator"))?;
            let value = fields
                .get_item("value")?
                .filter(|value| !value.is_none())
                .as_ref()
                .map(pyserde::value_from_py)
                .transpose()?;

            Ok(DocumentFilter {
                expression: pyserde::from_py(&expression)?,
                operator: pyserde::from_py(&operator)?,
                value,
            })
        })
        .collect()
}

fn authored_analysis(
    value: &Bound<'_, PyAny>,
) -> PyResult<Option<dokime_app::analysis::Selection>> {
    if value.is_none() {
        return Ok(None);
    }
    let fields = value.cast::<PyDict>()?;
    for (key, _) in fields.iter() {
        let key = key.extract::<String>()?;
        if !matches!(key.as_str(), "id" | "version" | "parameters") {
            return Err(PyValueError::new_err(format!(
                "unknown analysis field: {key}"
            )));
        }
    }
    let id = fields
        .get_item("id")?
        .ok_or_else(|| PyValueError::new_err("analysis requires an id"))?
        .extract()?;
    let version = fields
        .get_item("version")?
        .ok_or_else(|| PyValueError::new_err("analysis requires a version"))?
        .extract()?;
    let parameters = fields
        .get_item("parameters")?
        .map(|parameters| {
            parameters
                .cast::<PyDict>()?
                .iter()
                .map(|(key, value)| Ok((key.extract::<String>()?, pyserde::value_from_py(&value)?)))
                .collect::<PyResult<_>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(Some(dokime_app::analysis::Selection {
        id,
        version,
        parameters,
    }))
}

/* === Tests === */
