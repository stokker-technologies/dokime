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

/* === Re-Exports === */
pub use authoring::{
    AuthoredMetadata, Criterion, DeclaredSuite, DeclaredTemplate, Parameter, ParameterDefault,
    RustTemplate, Step, StepBinding, TemplateAuthoringError, TemplateContext,
    criterion as authored_criterion, format as authored_format, metadata as authored_metadata,
    unit as authored_unit,
};
pub use conditions::{RuntimeCondition, RuntimeConditionEvaluation, evaluate_runtime_condition};
pub use dokime_contract::definition::registry::{SuiteSet, SuiteSnapshot};
pub use dokime_contract::definition::{
    CaseMaterializationError, CaseMaterializationOptions, CaseMaterializationOutput,
    CaseMaterializationSelection, CriterionDefinition, DEFAULT_MATERIALIZED_CASE_CAP,
    DynamicDefaultKey, ExecutionPhaseDefinition, FileDefinition, MaterializedParameter,
    ParameterBinaryOp, ParameterDefinition, ParameterExpansion, ParameterExpr, ParameterGroup,
    ParameterGroupId, ParameterList, ParameterResolverError, ParameterSelection, ParameterSource,
    ParameterUnaryOp, ResolvedParameterSpace, Subscription, TemplateIdentifierRefusal, TestCase,
    TestCaseTemplate, TestSuite, coerce_definition_value, coerce_list_value,
    coerce_parameter_value, materialize_template_cases, materialized_parameter, parameter_name,
    resolve_parameter_spaces, template_identifier,
};
pub use dokime_contract::definition::{case_materialization, parameters, suite};

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
