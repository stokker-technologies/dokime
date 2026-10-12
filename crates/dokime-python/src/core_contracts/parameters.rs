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
use pyo3::{prelude::*, types::PyAny};
// Local Imports
use super::convert::*;
use crate::{pyserde, to_value_error};
use dokime_app::core::{CaseId, CriterionId, Metadata, ParameterId, StepId, TemplateId, ValueType};
use dokime_app::definition::{
    CaseMaterializationOptions, CaseMaterializationSelection, CriterionDefinition,
    ExecutionPhaseDefinition, FileDefinition, ParameterDefinition, TestCase, TestCaseTemplate,
    TestSuite, materialize_template_cases,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[pyfunction]
pub(crate) fn parameter_definition_contract(
    py: Python<'_>,
    parameter_id: String,
    metadata: Bound<'_, PyAny>,
    value_type: String,
    default: Bound<'_, PyAny>,
    values: Bound<'_, PyAny>,
    required: bool,
    unit: Bound<'_, PyAny>,
    group: Option<String>,
    attributes: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let metadata_value = pyserde::from_py::<Metadata>(&metadata)?;
    let default_value = if default.is_none() {
        None
    } else {
        Some(pyserde::value_from_py(&default)?)
    };
    let values_value = pyserde::values_from_py(&values)?;
    let attributes_value = pyserde::value_map_from_py(&attributes)?;
    let list = parameter_list(&attributes_value)?;
    let contract = ParameterDefinition {
        expansion: match attributes_value.get("expansion") {
            None => Default::default(),
            Some(value) => serde_json::from_value(value.to_plain_json().map_err(to_value_error)?)
                .map_err(to_value_error)?,
        },
        min: numeric_bound(&attributes_value, "min")?,
        max: numeric_bound(&attributes_value, "max")?,
        parameter_id: ParameterId::from_uuid(parameter_id).map_err(to_value_error)?,
        metadata: metadata_value.clone(),
        group_id: group
            .as_deref()
            .map(group_identifier)
            .transpose()
            .map_err(to_value_error)?,
        // A declared element type is what makes this a list, so the parameter
        // serializes as an array whichever alias the author spelled.
        value_type: if list.is_some() {
            ValueType::Array
        } else {
            value_type_from_str(&value_type)?
        },
        unit: optional_contract(&unit)?,
        required,
        expression: parameter_expression(
            &metadata_value,
            default_value,
            values_value,
            &attributes_value,
        )?,
        runtime_writable: false,
        selection: parameter_selection(&attributes_value),
        list,
        extensions: parameter_extensions(attributes_value, group),
    };

    pyserde::to_py(py, &contract)
}

#[pyfunction]
pub(crate) fn file_definition_contract(
    py: Python<'_>,
    metadata: Bound<'_, PyAny>,
    default: Option<String>,
    required: bool,
    media_types: Vec<String>,
    attributes: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let contract = FileDefinition {
        metadata: pyserde::from_py::<Metadata>(&metadata)?,
        default,
        required,
        media_types,
        attributes: pyserde::value_map_from_py(&attributes)?,
    };

    pyserde::to_py(py, &contract)
}

#[pyfunction]
pub(crate) fn execution_phase_definition_contract(
    py: Python<'_>,
    step_id: String,
    metadata: Bound<'_, PyAny>,
    repeatable: bool,
) -> PyResult<Py<PyAny>> {
    let contract = ExecutionPhaseDefinition {
        repeatable,
        step_id: StepId::from_uuid(step_id).map_err(to_value_error)?,
        metadata: pyserde::from_py::<Metadata>(&metadata)?,
    };

    pyserde::to_py(py, &contract)
}

#[pyfunction]
pub(crate) fn criterion_definition_contract(
    py: Python<'_>,
    criterion_id: String,
    metadata: Bound<'_, PyAny>,
    expected: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let contract = CriterionDefinition {
        criterion_id: CriterionId::from_uuid(criterion_id).map_err(to_value_error)?,
        metadata: pyserde::from_py::<Metadata>(&metadata)?,
        expected: optional_contract(&expected)?,
    };

    pyserde::to_py(py, &contract)
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub(crate) fn case_template_contract(
    py: Python<'_>,
    template_id: String,
    metadata: Bound<'_, PyAny>,
    tags: Vec<String>,
    parameters: Bound<'_, PyAny>,
    files: Bound<'_, PyAny>,
    document: Bound<'_, PyAny>,
    phases: Bound<'_, PyAny>,
    criteria: Bound<'_, PyAny>,
    continue_on_failure: bool,
    prompt_definitions: Bound<'_, PyAny>,
    subscriptions: Bound<'_, PyAny>,
    attributes: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let parameters = pyserde::from_py::<Vec<ParameterDefinition>>(&parameters)?;
    let contract = TestCaseTemplate {
        test_case_template_id: TemplateId::from_uuid(template_id).map_err(to_value_error)?,
        metadata: pyserde::from_py::<Metadata>(&metadata)?,
        tags,
        parameter_groups: parameter_groups_from_definitions(&parameters),
        parameters,
        files: pyserde::from_py::<Vec<FileDefinition>>(&files)?,
        document: optional_contract(&document)?,
        phases: pyserde::from_py::<Vec<ExecutionPhaseDefinition>>(&phases)?,
        criteria: pyserde::from_py::<Vec<CriterionDefinition>>(&criteria)?,
        continue_on_failure,
        prompt_definitions: pyserde::from_py(&prompt_definitions)?,
        subscriptions: pyserde::from_py::<Vec<dokime_app::definition::Subscription>>(
            &subscriptions,
        )?,
        attributes: pyserde::value_map_from_py(&attributes)?,
    };

    pyserde::to_py(py, &contract)
}

#[pyfunction]
pub(crate) fn suite_contract(
    py: Python<'_>,
    metadata: Bound<'_, PyAny>,
    tags: Vec<String>,
    templates: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let contract = TestSuite {
        metadata: pyserde::from_py::<Metadata>(&metadata)?,
        tags,
        // A suite declares no parameters of its own: every group belongs to the
        // template whose parameters name it.
        parameter_groups: Vec::new(),
        test_case_templates: pyserde::from_py::<Vec<TestCaseTemplate>>(&templates)?,
    };

    pyserde::to_py(py, &contract)
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub(crate) fn materialized_case_contract(
    py: Python<'_>,
    suite_id: String,
    case_id: String,
    template_id: String,
    metadata: Bound<'_, PyAny>,
    parameters: Bound<'_, PyAny>,
    generation_index: Option<u64>,
) -> PyResult<Py<PyAny>> {
    if suite_id.trim().is_empty() {
        return Err(to_value_error("suite registration ID is required"));
    }

    let contract = TestCase {
        suite_id,
        test_case_id: CaseId::from_wire(case_id).map_err(to_value_error)?,
        test_case_template_id: TemplateId::from_uuid(template_id).map_err(to_value_error)?,
        suite: None,
        metadata: pyserde::from_py::<Metadata>(&metadata)?,
        parameters: pyserde::value_map_from_py(&parameters)?,
        generation_index: generation_index.unwrap_or_default(),
    };

    pyserde::to_py(py, &contract)
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub(crate) fn materialize_template_cases_contract(
    py: Python<'_>,
    suite_id: String,
    template: Bound<'_, PyAny>,
    case_filter: Bound<'_, PyAny>,
    case_overrides: Bound<'_, PyAny>,
    file_overrides: Bound<'_, PyAny>,
    max_cases: usize,
    force: bool,
) -> PyResult<Py<PyAny>> {
    let output = materialize_template_cases(
        &suite_id,
        &pyserde::from_py::<TestCaseTemplate>(&template)?,
        CaseMaterializationSelection {
            case_filter: pyserde::value_lists_from_py(&case_filter)?,
            case_overrides: pyserde::value_lists_from_py(&case_overrides)?,
            file_overrides: pyserde::from_py::<BTreeMap<String, Option<String>>>(&file_overrides)?,
        },
        CaseMaterializationOptions {
            max_cases,
            force,
            ..CaseMaterializationOptions::default()
        },
    )
    .map_err(to_value_error)?;

    pyserde::to_py(py, &output)
}

/* === Private Functions === */

/* === Tests === */
