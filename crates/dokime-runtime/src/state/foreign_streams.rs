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

// Local Imports
use super::RuntimeStateInstallError;
use crate::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn declares_foreign_view(contract: &TelemetryContract) -> bool {
    contract.foreign
}

pub(super) fn admit_foreign_view(owner: &TelemetryContract, view: &TelemetryContract) -> Result<(), RuntimeStateInstallError> {
    let owner = owner.normalized();
    let declared = owner.column_map();
    let view = view.normalized();

    for column in &view.columns {
        let name = column.metadata.identifier.as_str();
        let Some(owned) = declared.get(name) else {
            return Err(RuntimeStateInstallError::ForeignTelemetryColumn {
                stream: view.stream.clone(),
                column: name.to_owned(),
                detail: "the stream's owner does not declare".to_owned(),
            });
        };
        if owned.value_type != column.value_type {
            return Err(RuntimeStateInstallError::ForeignTelemetryColumn {
                stream: view.stream.clone(),
                column: name.to_owned(),
                detail: format!("the stream's owner declares as {:?} and not {:?}", owned.value_type, column.value_type),
            });
        }
    }

    Ok(())
}

/* === Private Functions === */

/* === Tests === */
