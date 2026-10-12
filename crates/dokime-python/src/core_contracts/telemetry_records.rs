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
use pyo3::{prelude::*, types::PyAny};
// Local Imports
use super::convert::*;
use crate::{pyserde, to_value_error};
use dokime_app::core::{Identifier, Metadata, TemplateId};
use dokime_app::records::{RecordContract, RecordField};
use dokime_app::telemetry::{TelemetryColumn, TelemetryContract};

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
pub(crate) fn record_field_contract(
    py: Python<'_>,
    name: String,
    label: Option<String>,
    description: Option<String>,
    value_type: Bound<'_, PyAny>,
    required: bool,
    unit: Option<String>,
) -> PyResult<Py<PyAny>> {
    let resolved_value_type = declaration_value_type_from_py(&value_type)?;
    let nullable = is_nullable_value_type(&value_type)?;
    let mut contract = RecordField::new(
        Metadata::new(
            Identifier::new(name.clone()).map_err(to_value_error)?,
            label.unwrap_or_else(|| title_from_identifier(&name)),
            description,
        ),
        resolved_value_type,
    );
    contract.required = required;
    contract.nullable = nullable;
    contract.unit = unit.map(measurement_unit_from_symbol).transpose()?;

    pyserde::to_py(py, &contract)
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_contract(
    py: Python<'_>,
    record_type: String,
    label: Option<String>,
    description: Option<String>,
    default_level: String,
    allow_extra_fields: bool,
    fields: Bound<'_, PyAny>,
    template_scope: Option<String>,
    icon: Option<String>,
    tone: Option<String>,
) -> PyResult<Py<PyAny>> {
    let resolved = record_type.clone();
    let contract = RecordContract {
        record_type: Identifier::new(resolved.clone()).map_err(to_value_error)?,
        schema_version: 1,
        metadata: Metadata::new(
            Identifier::new(resolved).map_err(to_value_error)?,
            label.unwrap_or_else(|| title_from_identifier(&record_type)),
            description,
        ),
        display: record_display(icon.as_deref(), tone.as_deref())?,
        template_id: template_scope
            .map(|id| TemplateId::from_uuid(id).map_err(to_value_error))
            .transpose()?,
        owner: None,
        default_severity: record_severity_from_str(&default_level)?,
        fields: pyserde::from_py::<Vec<RecordField>>(&fields)?,
        validation_mode: None,
        allow_extra_fields,
    };

    pyserde::to_py(py, &contract)
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub(crate) fn telemetry_column_contract(
    py: Python<'_>,
    name: String,
    label: Option<String>,
    description: Option<String>,
    value_type: Bound<'_, PyAny>,
    role: Option<String>,
    unit: Option<String>,
    required: bool,
) -> PyResult<Py<PyAny>> {
    let resolved_value_type = declaration_value_type_from_py(&value_type)?;
    let nullable = is_nullable_value_type(&value_type)?;
    let mut contract = TelemetryColumn::new(
        Metadata::new(
            Identifier::new(name.clone()).map_err(to_value_error)?,
            label.unwrap_or_else(|| title_from_identifier(&name)),
            description,
        ),
        resolved_value_type,
        telemetry_role_from_str(role.as_deref().unwrap_or_else(|| default_role(&name)))?,
    );
    contract.required = required;
    contract.nullable = nullable;
    contract.display_unit = unit;

    pyserde::to_py(py, &contract)
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub(crate) fn telemetry_contract(
    py: Python<'_>,
    stream: String,
    label: Option<String>,
    description: Option<String>,
    columns: Bound<'_, PyAny>,
    default_x: Option<String>,
    allow_extra_fields: bool,
    scalar: bool,
    template_scope: Option<String>,
) -> PyResult<Py<PyAny>> {
    let resolved = stream.clone();
    let contract = TelemetryContract {
        stream: Identifier::new(resolved.clone()).map_err(to_value_error)?,
        schema_version: 1,
        metadata: Metadata::new(
            Identifier::new(resolved).map_err(to_value_error)?,
            label.unwrap_or_else(|| title_from_identifier(&stream)),
            description,
        ),
        template_id: template_scope
            .map(|id| TemplateId::from_uuid(id).map_err(to_value_error))
            .transpose()?,
        owner: None,
        columns: pyserde::from_py::<Vec<TelemetryColumn>>(&columns)?,
        default_x,
        // A stream declared from Python is pushed into by whatever is making
        // the rows, not sampled by the host on a clock, so there is no cadence
        // to state and nothing leases it.
        interval: None,
        plot_window: None,
        allow_extra_fields,
        scalar,
        validation_mode: None,
        foreign: false,
    };

    pyserde::to_py(py, &contract)
}

/* === Private Functions === */

/* === Tests === */
