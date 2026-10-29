/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(super) use bounds::Budget;

use tonic::Status;

use dokime::definition::ParameterExpr;
use dokime::definition::parameters::bounds;

pub(super) fn expression(expr: &ParameterExpr, budget: &mut Budget, depth: usize) -> Result<(), Status> {
    bounds::expression(expr, budget, depth).map_err(status)
}

pub(super) fn status(error: dokime::ApiError) -> Status {
    let dokime::ApiError::ParameterLimit(error) = error else {
        return super::rejection("invalid_selection", error.to_string());
    };
    use dokime::definition::parameters::limits::ParameterLimitError;
    match error {
        ParameterLimitError::Structure => Status::resource_exhausted(error.to_string()),
        ParameterLimitError::Expansion => super::rejection("parameter_expansion_limit", error.to_string()),
        ParameterLimitError::Invalid(message) => super::rejection("invalid_selection", message),
    }
}
