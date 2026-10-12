/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod convert;
pub(crate) mod identifiers;
pub(crate) mod parameters;
pub(crate) mod plots;
pub(crate) mod prompts;
pub(crate) mod telemetry_records;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[pyo3::pymodule]
pub(crate) fn core_contracts(
    module: &pyo3::Bound<'_, pyo3::types::PyModule>,
) -> pyo3::PyResult<()> {
    use pyo3::prelude::*;
    use pyo3::wrap_pyfunction;

    module.add_function(wrap_pyfunction!(identifiers::identifier_contract, module)?)?;
    module.add_function(wrap_pyfunction!(identifiers::metadata_contract, module)?)?;
    module.add_function(wrap_pyfunction!(
        identifiers::measurement_unit_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(identifiers::value_type_name, module)?)?;
    module.add_function(wrap_pyfunction!(identifiers::nullable_value_type, module)?)?;
    module.add_function(wrap_pyfunction!(
        parameters::parameter_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        prompts::prompt_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        parameters::file_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        parameters::execution_phase_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        parameters::criterion_definition_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        parameters::case_template_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(parameters::suite_contract, module)?)?;
    module.add_function(wrap_pyfunction!(
        parameters::materialized_case_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        parameters::materialize_template_cases_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(identifiers::stable_wire_key, module)?)?;
    module.add_function(wrap_pyfunction!(identifiers::stable_wire_hash, module)?)?;
    module.add_function(wrap_pyfunction!(
        telemetry_records::record_field_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        telemetry_records::record_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        telemetry_records::telemetry_column_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(
        telemetry_records::telemetry_contract,
        module
    )?)?;
    module.add_function(wrap_pyfunction!(plots::document_plot_contract, module)?)?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
