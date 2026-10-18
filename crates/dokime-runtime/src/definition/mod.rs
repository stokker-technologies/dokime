/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod authoring;
pub mod conditions;
#[cfg(feature = "host")]
pub mod cycle;
#[cfg(feature = "host")]
pub mod registry;

/* === Re-Exports === */
pub use authoring::{
    AuthoredMetadata, Criterion, DeclaredSuite, DeclaredTemplate, Parameter, ParameterDefault, RustTemplate, Step, StepBinding,
    TemplateAuthoringError, TemplateContext, criterion as authored_criterion, format as authored_format, metadata as authored_metadata,
    unit as authored_unit,
};
pub use conditions::{RuntimeCondition, RuntimeConditionEvaluation, evaluate_runtime_condition};
#[cfg(feature = "host")]
pub use cycle::TestCycleInstance;
pub use dokime_domain::model::definition::CaseMaterializationError;
pub use dokime_domain::model::definition::CaseMaterializationOptions;
pub use dokime_domain::model::definition::CaseMaterializationOutput;
pub use dokime_domain::model::definition::CaseMaterializationSelection;
pub use dokime_domain::model::definition::CriterionDefinition;
pub use dokime_domain::model::definition::DEFAULT_MATERIALIZED_CASE_CAP;
pub use dokime_domain::model::definition::DynamicDefaultKey;
pub use dokime_domain::model::definition::ExecutionPhaseDefinition;
pub use dokime_domain::model::definition::FileDefinition;
pub use dokime_domain::model::definition::MaterializedParameter;
pub use dokime_domain::model::definition::ParameterBinaryOp;
pub use dokime_domain::model::definition::ParameterDefinition;
pub use dokime_domain::model::definition::ParameterExpansion;
pub use dokime_domain::model::definition::ParameterExpr;
pub use dokime_domain::model::definition::ParameterGroup;
pub use dokime_domain::model::definition::ParameterGroupId;
pub use dokime_domain::model::definition::ParameterList;
pub use dokime_domain::model::definition::ParameterResolverError;
pub use dokime_domain::model::definition::ParameterSelection;
pub use dokime_domain::model::definition::ParameterSource;
pub use dokime_domain::model::definition::ParameterUnaryOp;
pub use dokime_domain::model::definition::ResolvedParameterSpace;
pub use dokime_domain::model::definition::Subscription;
pub use dokime_domain::model::definition::TemplateIdentifierRefusal;
pub use dokime_domain::model::definition::TestCase;
pub use dokime_domain::model::definition::TestCaseTemplate;
pub use dokime_domain::model::definition::TestSuite;
pub use dokime_domain::model::definition::case_materialization;
pub use dokime_domain::model::definition::coerce_definition_value;
pub use dokime_domain::model::definition::coerce_list_value;
pub use dokime_domain::model::definition::coerce_parameter_value;
pub use dokime_domain::model::definition::materialize_template_cases;
pub use dokime_domain::model::definition::materialized_parameter;
pub use dokime_domain::model::definition::parameter_name;
pub use dokime_domain::model::definition::parameters;
pub use dokime_domain::model::definition::registry::SuiteSet;
pub use dokime_domain::model::definition::registry::SuiteSnapshot;
pub use dokime_domain::model::definition::resolve_parameter_spaces;
pub use dokime_domain::model::definition::suite;
pub use dokime_domain::model::definition::template_identifier;
#[cfg(feature = "host")]
pub use registry::SuiteRegistry;

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

/* === Private Functions === */

/* === Tests === */
