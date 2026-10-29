/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[non_exhaustive]
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Identifier(#[from] harmos::IdentifierError),

    #[error("{0}")]
    Distribution(#[from] crate::model::identity::distribution::DistributionIdentityError),

    #[error("{0}")]
    Id(#[from] crate::model::identity::id::HubIdError),

    #[error("{0}")]
    Settings(#[from] crate::model::settings::SettingsError),

    #[error("{0}")]
    DataExpression(#[from] crate::model::expressions::DataExpressionError),

    #[error("{0}")]
    Expression(#[from] crate::model::expressions::ExpressionError),

    #[error("{0}")]
    Parameter(#[from] crate::model::definition::ParameterResolverError),

    #[error("{0}")]
    ParameterLimit(#[from] crate::model::definition::parameters::limits::ParameterLimitError),

    #[error("{0}")]
    CaseMaterialization(#[from] crate::model::definition::CaseMaterializationError),

    #[error("{0}")]
    Action(#[from] crate::model::plugins::ActionDispatcherError),

    #[error("{0}")]
    Permission(#[from] crate::model::plugins::PermissionError),

    #[error("{0}")]
    Suite(#[from] crate::model::plugins::SuiteContractError),
}
