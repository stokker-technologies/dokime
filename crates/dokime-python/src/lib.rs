/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![allow(clippy::too_many_arguments)]

/* === Modules === */
mod authoring;
mod client;
mod core_contracts;
mod error;
mod json_io;
mod pyserde;
mod runtime;
mod server;
mod suite_contracts;

/* === Re-Exports === */
pub use error::Error;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use pyo3::{exceptions::PyValueError, prelude::*, types::PyAny};
// Local Imports
use dokime_app::core::{
    CaseId, CriterionId, Identifier, Metadata, ParameterId, StepId, TemplateId, Timestamp,
};
use dokime_app::definition::{
    CriterionDefinition, ExecutionPhaseDefinition, ParameterDefinition, TestCaseTemplate, TestSuite,
};
use dokime_app::domains::TestSuiteDefinition;

/* === Constants === */

/* === Macros === */
macro_rules! host_fields {
    ($($ty:ty => $annotation:literal),* $(,)?) => {$(
        impl HostField for $ty {
            fn annotation() -> String { $annotation.into() }
        }
    )*};
}

host_fields! { bool => "bool", i64 => "int", f64 => "float", String => "str", Timestamp => "datetime" }

/* === Type Aliases === */

/* === Structs === */
#[pyclass(name = "Runtime", module = "dokime._native")]
pub struct PyRuntime {
    inner: std::sync::Arc<tokio::sync::Mutex<runtime::State>>,
}

/* === Enums === */

/* === Traits === */
trait HostField {
    fn annotation() -> String;
}

/* === Implementations === */
impl<T: HostField> HostField for Option<T> {
    fn annotation() -> String {
        format!("{} | None", T::annotation())
    }
}

impl<T: HostField> HostField for Vec<T> {
    fn annotation() -> String {
        format!("list[{}]", T::annotation())
    }
}

/* === Public Functions === */
pub(crate) fn to_value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/* === Private Functions === */
#[pymodule]
#[tracing::instrument(level = "debug", skip_all)]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let client = pyo3::wrap_pymodule!(client::client)(module.py());
    module.add_submodule(client.bind(module.py()))?;
    client
        .bind(module.py())
        .setattr("__name__", "dokime._native.client")?;
    client
        .bind(module.py())
        .setattr("__package__", "dokime._native")?;
    module
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item("dokime._native.client", &client)?;

    let authoring = pyo3::wrap_pymodule!(authoring::authoring)(module.py());
    module.add_submodule(authoring.bind(module.py()))?;
    authoring
        .bind(module.py())
        .setattr("__name__", "dokime._native.authoring")?;
    authoring
        .bind(module.py())
        .setattr("__package__", "dokime._native")?;
    module
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item("dokime._native.authoring", &authoring)?;
    let core_contracts = pyo3::wrap_pymodule!(core_contracts::core_contracts)(module.py());
    module.add_submodule(core_contracts.bind(module.py()))?;
    core_contracts
        .bind(module.py())
        .setattr("__name__", "dokime._native.core_contracts")?;
    core_contracts
        .bind(module.py())
        .setattr("__package__", "dokime._native")?;
    module
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item("dokime._native.core_contracts", &core_contracts)?;
    let server = pyo3::wrap_pymodule!(server::server)(module.py());
    module.add_submodule(server.bind(module.py()))?;
    server
        .bind(module.py())
        .setattr("__name__", "dokime._native.server")?;
    server
        .bind(module.py())
        .setattr("__package__", "dokime._native")?;
    module
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item("dokime._native.server", &server)?;

    module.add_class::<PyRuntime>()?;
    module.add_class::<authoring::context::PyContext>()?;
    module.add_class::<authoring::artifacts::PyArtifacts>()?;
    module.add_class::<authoring::artifacts::PyArtifactWriter>()?;
    module.add_class::<server::config::PyServerConfig>()?;
    module.add_class::<server::config::PyTlsIdentity>()?;
    module.add_class::<server::config::PyBearerGrant>()?;
    module.add_class::<server::config::PyRole>()?;
    module.add_function(wrap_pyfunction!(host_stream_contracts, module)?)?;
    module.add_function(wrap_pyfunction!(new_template_id, module)?)?;
    module.add_function(wrap_pyfunction!(new_parameter_id, module)?)?;
    module.add_function(wrap_pyfunction!(new_step_id, module)?)?;
    module.add_function(wrap_pyfunction!(new_criterion_id, module)?)?;
    module.add_function(wrap_pyfunction!(new_case_id, module)?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::identifiers::identifier_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::identifiers::metadata_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::identifiers::measurement_unit_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::identifiers::value_type_name,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::identifiers::nullable_value_type,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::parameters::parameter_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::prompts::prompt_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::parameters::file_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::parameters::execution_phase_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::parameters::criterion_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::parameters::case_template_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::parameters::suite_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::parameters::materialized_case_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::parameters::materialize_template_cases_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::identifiers::stable_wire_key,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::identifiers::stable_wire_hash,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::telemetry_records::record_field_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::telemetry_records::record_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::telemetry_records::telemetry_column_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::telemetry_records::telemetry_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        core_contracts::plots::document_plot_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        suite_contracts::suite_registration_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(validate_identifier, module)?)?;
    module.add_function(wrap_pyfunction!(validate_metadata, module)?)?;
    module.add_function(wrap_pyfunction!(validate_parameter_definition, module)?)?;
    module.add_function(wrap_pyfunction!(
        validate_execution_phase_definition,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(validate_criterion_definition, module)?)?;
    module.add_function(wrap_pyfunction!(validate_case_template, module)?)?;
    module.add_function(wrap_pyfunction!(validate_suite, module)?)?;
    module.add_function(wrap_pyfunction!(validate_materialized_case, module)?)?;
    module.add_function(wrap_pyfunction!(validate_suite_registration, module)?)?;
    module.add_function(wrap_pyfunction!(builtin_record_contracts, module)?)?;
    module.add_function(wrap_pyfunction!(json_io::write_json_file, module)?)?;
    module.add_function(wrap_pyfunction!(json_io::write_json_lines, module)?)?;

    Ok(())
}

#[pyfunction]
fn new_template_id() -> String {
    TemplateId::new().to_string()
}

#[pyfunction]
fn new_parameter_id() -> String {
    ParameterId::new().to_string()
}

#[pyfunction]
fn new_step_id() -> String {
    StepId::new().to_string()
}

#[pyfunction]
fn new_criterion_id() -> String {
    CriterionId::new().to_string()
}

#[pyfunction]
fn new_case_id() -> String {
    CaseId::new().to_string()
}

#[pyfunction]
fn validate_identifier(py: Python<'_>, payload: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<Identifier>(py, &payload)
}

#[pyfunction]
fn validate_metadata(py: Python<'_>, payload: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<Metadata>(py, &payload)
}

#[pyfunction]
fn validate_parameter_definition(py: Python<'_>, payload: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<ParameterDefinition>(py, &payload)
}

#[pyfunction]
fn validate_execution_phase_definition(
    py: Python<'_>,
    payload: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<ExecutionPhaseDefinition>(py, &payload)
}

#[pyfunction]
fn validate_criterion_definition(py: Python<'_>, payload: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<CriterionDefinition>(py, &payload)
}

#[pyfunction]
fn validate_case_template(py: Python<'_>, payload: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<TestCaseTemplate>(py, &payload)
}

#[pyfunction]
fn validate_suite(py: Python<'_>, payload: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<TestSuite>(py, &payload)
}

#[pyfunction]
fn validate_materialized_case(py: Python<'_>, payload: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<dokime_app::definition::TestCase>(py, &payload)
}

#[pyfunction]
fn validate_suite_registration(py: Python<'_>, payload: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    pyserde::round_trip_py::<TestSuiteDefinition>(py, &payload)
}

#[pyfunction]
fn builtin_record_contracts(py: Python<'_>) -> PyResult<Py<PyAny>> {
    pyserde::to_py(py, &dokime_app::records::builtin_record_contracts())
}

#[pyfunction]
fn host_stream_contracts(py: Python<'_>) -> PyResult<Py<PyAny>> {
    macro_rules! rows {
        ($( $stream:ident { id: $id:literal, label: $label:literal, columns: {
            $( $field:ident: $value_type:ident, $role:ident, $column_label:literal, $ty:ty; )*
        } } )*) => {{
            let mut rows = std::collections::BTreeMap::new();
            $(
                let fields = std::collections::BTreeMap::from([
                    $((stringify!($field), <$ty as HostField>::annotation())),*
                ]);
                rows.insert(stringify!($stream), serde_json::json!({
                    "contract": dokime_contract::telemetry::host_telemetry_contract(
                        dokime_contract::telemetry::HostTelemetryStream::$stream
                    ),
                    "fields": fields,
                }));
            )*
            rows
        }};
    }
    let rows = dokime_contract::host_streams!(rows);
    pyserde::to_py(py, &rows)
}

/* === Tests === */
