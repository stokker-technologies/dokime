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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Result<T, E = Error> = std::result::Result<T, E>;

/* === Structs === */

/* === Enums === */
#[non_exhaustive]
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Identifier(#[from] harmos::IdentifierError),
    #[error("{0}")]
    Id(#[from] crate::core::id::HubIdError),
    #[error("{0}")]
    Settings(#[from] crate::settings::SettingsError),
    #[error("{0}")]
    DataExpression(#[from] crate::expressions::DataExpressionError),
    #[error("{0}")]
    Expression(#[from] crate::expressions::ExpressionError),
    #[error("{0}")]
    Parameter(#[from] crate::definition::ParameterResolverError),
    #[error("{0}")]
    ParameterLimit(#[from] crate::definition::parameters::limits::ParameterLimitError),
    #[error("{0}")]
    CaseMaterialization(#[from] crate::definition::CaseMaterializationError),
    #[error("{0}")]
    Action(#[from] crate::domains::ActionDispatcherError),
    #[error("{0}")]
    Permission(#[from] crate::domains::PermissionError),
    #[error("{0}")]
    Suite(#[from] crate::domains::SuiteContractError),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
