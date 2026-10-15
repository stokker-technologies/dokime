/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod case_materialization;
pub mod conditions;
pub mod parameters;
pub mod registry;
pub mod suite;

/* === Re-Exports === */
pub use case_materialization::{
    CaseMaterializationError, CaseMaterializationOptions, CaseMaterializationOutput,
    CaseMaterializationSelection, DEFAULT_MATERIALIZED_CASE_CAP, case_label_number,
    materialize_template_cases,
};
pub use conditions::{RuntimeCondition, RuntimeConditionEvaluation};
pub use parameters::{
    DynamicDefaultKey, MaterializedParameter, ParameterBinaryOp, ParameterDefinition,
    ParameterExpansion, ParameterExpr, ParameterGroup, ParameterGroupId, ParameterList,
    ParameterResolverError, ParameterSelection, ParameterSource, ParameterUnaryOp,
    ResolvedParameterSpace, coerce_definition_value, coerce_list_value, coerce_parameter_value,
    materialized_parameter, parameter_name, resolve_parameter_spaces,
};
pub use registry::{SuiteSet, SuiteSnapshot};
pub use suite::{
    CriterionDefinition, ExecutionPhaseDefinition, FileDefinition, Subscription,
    TemplateIdentifierRefusal, TestCase, TestCaseTemplate, TestSuite, template_identifier,
};

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
