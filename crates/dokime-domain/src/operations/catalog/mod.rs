/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod get_record_schema;
mod get_suite;
mod get_telemetry_schema;
mod list_record_schemas;
mod list_suites;
mod list_telemetry_schemas;

/* === Re-Exports === */
pub use get_record_schema::GetRecordSchema;
pub use get_suite::GetSuite;
pub use get_telemetry_schema::GetTelemetrySchema;
pub use list_record_schemas::ListRecordSchemas;
pub use list_suites::ListSuites;
pub use list_telemetry_schemas::ListTelemetrySchemas;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::common::Metadata;
#[allow(unused_imports)]
use super::common::*;
use super::cursor::Cursor;
#[allow(unused_imports)]
use super::schema::*;
use super::value::Value;
use super::{Access, Endpoint, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Pages are ordered by registered suite ID. A changed cursor requires restarting
// pagination; pages are not pinned across runtime changes.


#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListSuitesReply {
    pub suites: Vec<SuiteIdentity>,
    pub next_after_suite_id: Option<String>,
    pub cursor: Option<Cursor>,
}



#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SuiteIdentity {
    pub suite_id: String,
    pub source: String,
    pub metadata: Option<Metadata>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SuiteDefinition {
    pub identity: Option<SuiteIdentity>,
    pub templates: Vec<TemplateDefinition>,
    pub tags: Vec<String>,
    pub parameter_groups: Vec<ParameterGroupDefinition>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateDefinition {
    pub template_id: String,
    pub metadata: Option<Metadata>,
    pub parameters: Vec<ParameterSchema>,
    pub tags: Vec<String>,
    pub files: Vec<FileParameterDefinition>,
    pub steps: Vec<StepDefinition>,
    pub parameter_groups: Vec<ParameterGroupDefinition>,
    pub continue_on_failure: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileParameterDefinition {
    pub metadata: Option<Metadata>,
    pub default_value: Option<String>,
    pub required: bool,
    pub media_types: Vec<String>,
    pub attributes: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StepDefinition {
    pub step_id: String,
    pub metadata: Option<Metadata>,
    pub repeatable: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterGroupDefinition {
    pub group_id: String,
    pub metadata: Option<Metadata>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSchemaExtensionsEntry {
    pub key: String,
    pub value: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub parameter_id: String,
    pub metadata: Option<Metadata>,
    pub group_id: Option<String>,
    pub value_type: ValueType,
    pub unit: Option<MeasurementUnit>,
    pub required: bool,
    pub runtime_writable: bool,
    pub expression: Option<ParameterExpression>,
    pub selection: Option<ParameterSelection>,
    // Plugin-defined typed metadata retained from the registered parameter.
    #[row(description = "Plugin-defined typed metadata retained from the registered parameter.")]
    pub extensions: Vec<ParameterSchemaExtensionsEntry>,
    // Present when the parameter is a list: one array value of typed elements
    // belonging to one case, never a parameter space. `value_type` stays
    // VALUE_TYPE_ARRAY; this carries the element type and the length bounds a
    // list editor needs.
    #[row(
        description = "Present when the parameter is a list: one array value of typed elements belonging to one case, never a parameter space. `value_type` stays VALUE_TYPE_ARRAY; this carries the element type and the length bounds a list editor needs."
    )]
    pub list: Option<ParameterList>,
    // Inclusive numeric bounds; for lists, applied to each element.
    #[row(description = "Inclusive numeric bounds; for lists, applied to each element.")]
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub expansion: Option<ParameterExpansion>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterList {
    pub element_type: ValueType,
    pub min_items: Option<u32>,
    pub max_items: Option<u32>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSelection {
    pub choices: Vec<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterExpansion {
    pub kind: Option<ParameterExpansionKind>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
// An expression tree flattened into its nodes. `nodes[0]` is the root and a
// node names its operands by their index in `nodes`; an operand index always
// exceeds the index of the node that uses it, so the tree has no cycles.
pub struct ParameterExpression {
    pub nodes: Vec<ParameterNode>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterNode {
    pub kind: Option<ParameterNodeKind>,
}

// Operand indices into the enclosing expression's nodes.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterOperands {
    pub values: Vec<u32>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegerRange {
    pub start: i64,
    pub stop: i64,
    pub step: i64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberRange {
    pub start: f64,
    pub stop: f64,
    pub step: f64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearSpace {
    pub start: f64,
    pub stop: f64,
    pub count: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicDefault {
    pub key: DynamicDefaultKey,
    // Operand index of the fallback expression.
    #[row(description = "Operand index of the fallback expression.")]
    pub fallback: Option<u32>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryExpression {
    pub op: UnaryOperator,
    // Operand index.
    #[row(description = "Operand index.")]
    pub value: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub op: BinaryOperator,
    // Operand index.
    #[row(description = "Operand index.")]
    pub left: u32,
    // Operand index.
    #[row(description = "Operand index.")]
    pub right: u32,
}

// Current admitted schemas, ordered by type/stream ID. These catalogs exclude
// retired definitions retained only for immutable historical evidence. A cursor
// belongs to one runtime epoch/revision; restart pagination when it expires.


#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListRecordSchemasReply {
    pub schemas: Vec<RecordSchemaIdentity>,
    pub next_after_record_type: Option<String>,
    pub cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordSchemaIdentity {
    pub record_type: String,
    pub schema_version: u32,
    pub metadata: Option<Metadata>,
}





#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListTelemetrySchemasReply {
    pub schemas: Vec<TelemetrySchemaIdentity>,
    pub next_after_stream: Option<String>,
    pub cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySchemaIdentity {
    pub stream: String,
    pub schema_version: u32,
    pub metadata: Option<Metadata>,
}



/* === Enums === */
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicDefaultKey {
    Unspecified,
    OperatorName,
    OperatorEmail,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Unspecified,
    Negate,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Unspecified,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterExpansionKind {
    Single(bool),
    Cartesian(bool),
    Zip(String),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterNodeKind {
    Literal(Value),
    Array(ParameterOperands),
    Range(IntegerRange),
    Arange(NumberRange),
    Linspace(LinearSpace),
    Reference(String),
    DynamicDefault(DynamicDefault),
    Unary(UnaryExpression),
    Binary(BinaryExpression),
}

#[harmos::catalog]
pub enum Operations {
    ListSuites(ListSuites),
    GetSuite(GetSuite),
    ListRecordSchemas(ListRecordSchemas),
    GetRecordSchema(GetRecordSchema),
    ListTelemetrySchemas(ListTelemetrySchemas),
    GetTelemetrySchema(GetTelemetrySchema),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListSuites {
    const PATH: &'static str = "/dokime.v1.Catalog/ListSuites";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetSuite {
    const PATH: &'static str = "/dokime.v1.Catalog/GetSuite";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListRecordSchemas {
    const PATH: &'static str = "/dokime.v1.Catalog/ListRecordSchemas";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetRecordSchema {
    const PATH: &'static str = "/dokime.v1.Catalog/GetRecordSchema";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListTelemetrySchemas {
    const PATH: &'static str = "/dokime.v1.Catalog/ListTelemetrySchemas";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for GetTelemetrySchema {
    const PATH: &'static str = "/dokime.v1.Catalog/GetTelemetrySchema";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
