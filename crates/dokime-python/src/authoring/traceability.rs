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
use pyo3::prelude::*;
// Local Imports
use crate::{pyserde, to_value_error};
use dokime::core::Identifier;
use dokime::traceability::{TraceabilityRow, TraceabilitySection};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn sections(value: &Bound<'_, PyAny>) -> PyResult<Vec<TraceabilitySection>> {
    value
        .try_iter()?
        .map(|section| {
            let section = section?;
            let rows = section
                .getattr("rows")?
                .try_iter()?
                .map(|row| {
                    let row = row?;
                    Ok(TraceabilityRow {
                        identifier: identifier(&row)?,
                        label: row.getattr("label")?.extract()?,
                        value: pyserde::value_from_py(&row.getattr("value")?)?,
                        unit: row.getattr("unit")?.extract()?,
                        source: row.getattr("source")?.extract()?,
                    })
                })
                .collect::<PyResult<Vec<_>>>()?;

            Ok(TraceabilitySection {
                identifier: identifier(&section)?,
                label: section.getattr("label")?.extract()?,
                rows,
            })
        })
        .collect()
}

/* === Private Functions === */
fn identifier(value: &Bound<'_, PyAny>) -> PyResult<Identifier> {
    Identifier::new(value.getattr("identifier")?.extract::<String>()?).map_err(to_value_error)
}

/* === Tests === */
