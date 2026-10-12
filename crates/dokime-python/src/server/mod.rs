/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod config;

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
pub(crate) fn server(module: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    use pyo3::prelude::*;

    module.add_class::<config::PyServerConfig>()?;
    module.add_class::<config::PyTlsIdentity>()?;
    module.add_class::<config::PyBearerGrant>()?;
    module.add_class::<config::PyRole>()?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
