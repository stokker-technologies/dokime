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
use crate::{pyserde, to_value_error};
use dokime_app::definition::TestSuite;
use dokime_app::domains::TestSuiteDefinition;

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
pub(crate) fn suite_registration_contract(
    py: Python<'_>,
    suite_id: String,
    product: Option<String>,
    tags: Vec<String>,
    workflow: Option<String>,
    suite: Bound<'_, PyAny>,
    record_contracts: Bound<'_, PyAny>,
    telemetry_contracts: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let mut registration =
        TestSuiteDefinition::new(suite_id, pyserde::from_py::<TestSuite>(&suite)?)
            .map_err(to_value_error)?;
    registration.product = product;
    registration.tags = tags;
    registration.workflow = workflow;
    registration.record_contracts = pyserde::from_py(&record_contracts)?;
    registration.telemetry_contracts = pyserde::from_py(&telemetry_contracts)?;

    pyserde::to_py(py, &registration)
}

/* === Private Functions === */

/* === Tests === */
