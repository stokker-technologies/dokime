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
pub(super) use bounds::Budget;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use tonic::Status;
// Local Imports
use dokime_app::definition::ParameterExpr;
use dokime_app::definition::parameters::bounds;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn expression(
    expr: &ParameterExpr,
    budget: &mut Budget,
    depth: usize,
) -> Result<(), Status> {
    bounds::expression(expr, budget, depth).map_err(status)
}

pub(super) fn status(error: dokime_app::ApiError) -> Status {
    let dokime_app::ApiError::ParameterLimit(error) = error else {
        return super::rejection("invalid_selection", error.to_string());
    };
    use dokime_app::definition::parameters::limits::ParameterLimitError;
    match error {
        ParameterLimitError::Structure => Status::resource_exhausted(error.to_string()),
        ParameterLimitError::Expansion => {
            super::rejection("parameter_expansion_limit", error.to_string())
        }
        ParameterLimitError::Payload => super::payload::limit(),
        ParameterLimitError::Invalid(message) => super::rejection("invalid_selection", message),
    }
}

/* === Private Functions === */

/* === Tests === */
