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
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use crate::core::{
    CriterionId, Identifier, MeasurementUnit, Metadata, ParameterId, StepId, TemplateId,
    UnitFamily, Value, ValueFormat, ValueType, validate_value_format,
};
use crate::definition::{
    CriterionDefinition, DynamicDefaultKey, ExecutionPhaseDefinition, ParameterDefinition,
    ParameterExpr, ParameterGroup, ParameterGroupId, ParameterList, ParameterSelection,
    TestCaseTemplate,
};
use crate::document::DocumentDefinition;
use crate::domains::TestSuiteDefinition;
use crate::records::RecordContract;
use crate::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */
product_error!(TemplateAuthoringError, formatter {
    Self::MissingMetadata => formatter.write_str("test case template metadata is required"),
    Self::InvalidIdentifier { field, value, reason } => write!(formatter, "invalid {field} identifier {value:?}: {reason}"),
});

/* === Type Aliases === */
type StepAction<T, C, E> = dyn Fn(&mut T, &mut C) -> Result<(), E> + Send + Sync;

/* === Structs === */
pub struct TemplateContext {
    template_id: Option<TemplateId>,
    metadata: Option<Metadata>,
    tags: Vec<String>,
    parameter_groups: Vec<ParameterGroup>,
    parameters: Vec<ParameterDefinition>,
    document: Option<DocumentDefinition>,
    steps: Vec<Step>,
    criteria: Vec<CriterionDefinition>,
    continue_on_failure: bool,
    prompts: Vec<crate::prompts::PromptDefinition>,
    subscriptions: Vec<super::Subscription>,
    attributes: BTreeMap<String, Value>,
    errors: Vec<TemplateAuthoringError>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredMetadata {
    pub identifier: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Criterion {
    pub identifier: String,
    pub label: String,
    pub description: Option<String>,
    pub expected: Option<Value>,
}

pub struct Parameter {
    pub expansion: super::ParameterExpansion,
    pub identifier: String,
    pub label: String,
    pub description: Option<String>,
    pub group_id: Option<ParameterGroupId>,
    pub value_type: ValueType,
    pub unit: Option<MeasurementUnit>,
    pub required: bool,
    pub default: ParameterDefault,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub runtime_writable: bool,
    pub selection: Option<ParameterSelection>,
    pub list: Option<ParameterList>,
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub repeatable: bool,
    pub identifier: String,
    pub label: String,
    pub description: Option<String>,
}

pub struct StepBinding<T, C, E = ()> {
    step: Step,
    action: Box<StepAction<T, C, E>>,
}

/* === Enums === */
pub enum ParameterDefault {
    Expression(ParameterExpr),
    Source {
        source: Box<dyn Fn() -> Option<Value> + Send + Sync>,
        fallback: Value,
    },
}

#[derive(Debug)]
pub enum TemplateAuthoringError {
    MissingMetadata,
    InvalidIdentifier {
        field: &'static str,
        value: String,
        reason: dokime_contract::Error,
    },
}

/* === Traits === */
pub trait RustTemplate {
    fn define(&self, ctx: &mut TemplateContext);

    fn to_test_case_template(&self) -> Result<TestCaseTemplate, TemplateAuthoringError> {
        let mut ctx = TemplateContext::new();
        self.define(&mut ctx);
        ctx.into_test_case_template()
    }
}

pub trait DeclaredTemplate {
    fn template_id() -> TemplateId;

    fn declaration() -> TestCaseTemplate;

    fn stream_contracts() -> Vec<TelemetryContract> {
        Vec::new()
    }

    fn record_contracts() -> Vec<RecordContract> {
        Vec::new()
    }
}

pub trait DeclaredSuite {
    #[cfg(feature = "native")]
    const OPERATIONS: &'static [harmos::catalog::Node<harmos::operations::WireDeclaration>];

    fn definition() -> TestSuiteDefinition;

    #[cfg(feature = "native")]
    fn bind<P: harmos::artifact::plugin::Declared>(
        table: &mut harmos::sidecar::operations::WireTable<harmos::sidecar::Supervisor>,
        workflow: &str,
        records: &[RecordContract],
    ) -> Result<(), harmos::sidecar::operations::ServingError>;

    #[cfg(feature = "native")]
    fn runners() -> Vec<std::sync::Arc<dyn crate::execution::Runnable>>;
}

/* === Implementations === */
impl TemplateContext {
    pub fn new() -> Self {
        Self {
            template_id: None,
            metadata: None,
            tags: Vec::new(),
            parameter_groups: Vec::new(),
            parameters: Vec::new(),
            document: None,
            steps: Vec::new(),
            criteria: Vec::new(),
            continue_on_failure: false,
            prompts: Vec::new(),
            subscriptions: Vec::new(),
            attributes: BTreeMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn set_template_id(&mut self, template_id: TemplateId) {
        self.template_id = Some(template_id);
    }

    pub fn set_metadata(&mut self, metadata: AuthoredMetadata) {
        match metadata.build() {
            Ok(metadata) => self.metadata = Some(metadata),
            Err(error) => self.errors.push(error),
        }
    }

    pub fn set_tags(&mut self, tags: Vec<impl Into<String>>) {
        self.tags = tags.into_iter().map(Into::into).collect();
    }

    pub fn set_parameter_groups(&mut self, parameter_groups: Vec<ParameterGroup>) {
        self.parameter_groups = parameter_groups;
    }

    pub fn set_parameters(&mut self, parameters: Vec<Parameter>) {
        self.parameters = self.compile_parameters(parameters);
    }

    pub fn set_parameter_definitions(&mut self, parameters: Vec<ParameterDefinition>) {
        self.parameters = parameters;
    }

    pub fn set_document(&mut self, document: DocumentDefinition) {
        self.document = Some(document);
    }

    pub fn set_steps(&mut self, steps: Vec<Step>) {
        self.steps = steps;
    }

    pub fn set_criteria(&mut self, criteria: Vec<Criterion>) {
        self.criteria.clear();
        for criterion in criteria {
            match criterion.into_definition() {
                Ok(criterion) => self.criteria.push(criterion),
                Err(error) => self.errors.push(error),
            }
        }
    }

    pub fn set_continue_on_failure(&mut self, continue_on_failure: bool) {
        self.continue_on_failure = continue_on_failure;
    }

    pub fn set_prompts(
        &mut self,
        prompts: impl IntoIterator<Item = crate::prompts::PromptDefinition>,
    ) {
        self.prompts = prompts.into_iter().collect();
    }

    pub fn set_subscriptions(&mut self, subscriptions: Vec<super::Subscription>) {
        self.subscriptions.clear();
        for subscription in subscriptions {
            if !self.subscriptions.contains(&subscription) {
                self.subscriptions.push(subscription);
            }
        }
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: Value) {
        self.attributes.insert(key.into(), value);
    }

    pub fn into_test_case_template(mut self) -> Result<TestCaseTemplate, TemplateAuthoringError> {
        if let Some(error) = self.errors.drain(..).next() {
            return Err(error);
        }
        let metadata = self
            .metadata
            .ok_or(TemplateAuthoringError::MissingMetadata)?;
        let mut phases = self
            .steps
            .into_iter()
            .map(ExecutionPhaseDefinition::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let template_id = self
            .template_id
            .unwrap_or_else(|| TemplateId::from_seed(metadata.identifier.as_str()));
        // These are declaration identities. Execution IDs are minted separately.
        // References in parameter expressions and authored criteria use identifiers.
        for parameter in &mut self.parameters {
            parameter.parameter_id = ParameterId::from_seed(format!(
                "{template_id}:local:{}",
                parameter.metadata.identifier
            ));
        }
        for criterion in &mut self.criteria {
            criterion.criterion_id =
                CriterionId::from_seed(format!("{template_id}:{}", criterion.metadata.identifier));
        }
        for phase in &mut phases {
            phase.step_id =
                StepId::from_seed(format!("{template_id}:{}", phase.metadata.identifier));
        }
        let attributes = self.attributes;

        Ok(TestCaseTemplate {
            test_case_template_id: template_id,
            metadata,
            tags: self.tags,
            parameter_groups: self.parameter_groups,
            // A template declares its own parameters and names the files a
            // suite already published; nothing an author writes fills either
            // of these, so the declaration leaves them empty.
            parameters: self.parameters,
            files: Vec::new(),
            document: self.document,
            phases,
            criteria: self.criteria,
            continue_on_failure: self.continue_on_failure,
            prompt_definitions: self.prompts,
            subscriptions: self.subscriptions,
            attributes,
        })
    }

    fn compile_parameters(&mut self, parameters: Vec<Parameter>) -> Vec<ParameterDefinition> {
        let mut definitions = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            match parameter.try_into_definition_as("parameter") {
                Ok(definition) => definitions.push(definition),
                Err(error) => self.errors.push(error),
            }
        }
        definitions
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthoredMetadata {
    pub fn build(self) -> Result<Metadata, TemplateAuthoringError> {
        Ok(Metadata::new(
            validated_identifier("metadata", self.identifier)?,
            self.label,
            self.description
                .filter(|description| !description.trim().is_empty()),
        ))
    }
}

impl Criterion {
    fn into_definition(self) -> Result<CriterionDefinition, TemplateAuthoringError> {
        Ok(CriterionDefinition {
            criterion_id: CriterionId::from_seed(&self.identifier),
            metadata: Metadata::new(
                validated_identifier("criterion", self.identifier)?,
                self.label,
                self.description,
            ),
            expected: self.expected,
        })
    }
}

impl Parameter {
    pub fn new(
        identifier: impl Into<String>,
        label: impl Into<String>,
        value_type: ValueType,
    ) -> Self {
        Self {
            expansion: super::ParameterExpansion::Single,
            identifier: identifier.into(),
            label: label.into(),
            description: None,
            group_id: None,
            value_type,
            unit: None,
            required: true,
            default: ParameterDefault::literal(Value::Null),
            min: None,
            max: None,
            runtime_writable: false,
            selection: None,
            list: None,
            extensions: BTreeMap::new(),
        }
    }

    pub fn expansion(mut self, expansion: super::ParameterExpansion) -> Self {
        self.expansion = expansion;
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn default(mut self, value: impl Into<ParameterDefault>) -> Self {
        self.default = value.into();
        self
    }

    pub fn unit(mut self, unit: impl Into<Option<MeasurementUnit>>) -> Self {
        self.unit = unit.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn text(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(identifier, label, ValueType::Text)
    }

    pub fn file(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(identifier, label, ValueType::File)
    }

    pub fn boolean(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(identifier, label, ValueType::Boolean)
    }

    pub fn integer(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(identifier, label, ValueType::Integer)
    }

    pub fn number(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(identifier, label, ValueType::Number)
    }

    pub fn array(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(identifier, label, ValueType::Array)
    }

    pub fn list(
        identifier: impl Into<String>,
        label: impl Into<String>,
        element: ValueType,
    ) -> Self {
        let mut parameter = Self::new(identifier, label, ValueType::Array);
        parameter.list = Some(ParameterList {
            element_type: element,
            min_items: None,
            max_items: None,
        });
        parameter
    }

    pub fn choices<I, V>(mut self, choices: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        self.selection = Some(ParameterSelection {
            choices: choices.into_iter().map(Into::into).collect(),
        });
        self
    }

    pub fn min_items(mut self, min_items: usize) -> Self {
        if let Some(list) = &mut self.list {
            list.min_items = Some(min_items);
        }
        self
    }

    pub fn max_items(mut self, max_items: usize) -> Self {
        if let Some(list) = &mut self.list {
            list.max_items = Some(max_items);
        }
        self
    }

    pub fn object(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self::new(identifier, label, ValueType::Object)
    }

    pub fn try_into_definition(self) -> Result<ParameterDefinition, TemplateAuthoringError> {
        self.try_into_definition_as("parameter")
    }

    fn try_into_definition_as(
        self,
        field: &'static str,
    ) -> Result<ParameterDefinition, TemplateAuthoringError> {
        Ok(ParameterDefinition {
            expansion: self.expansion,
            parameter_id: ParameterId::from_seed(&self.identifier),
            metadata: Metadata::new(
                validated_identifier(field, self.identifier)?,
                self.label,
                self.description
                    .filter(|description| !description.trim().is_empty()),
            ),
            group_id: self.group_id,
            value_type: self.value_type,
            unit: self.unit,
            min: self.min,
            max: self.max,
            required: self.required,
            expression: self.default.into_expression(),
            runtime_writable: self.runtime_writable,
            selection: self.selection,
            list: self.list,
            extensions: self.extensions,
        })
    }
}

impl ParameterDefault {
    pub fn literal(value: impl Into<Value>) -> Self {
        Self::Expression(ParameterExpr::Literal {
            value: value.into(),
        })
    }

    pub fn expression(expression: ParameterExpr) -> Self {
        Self::Expression(expression)
    }

    pub fn dynamic(key: DynamicDefaultKey, fallback: impl Into<Value>) -> Self {
        Self::Expression(ParameterExpr::DynamicDefault {
            key,
            fallback: Some(Box::new(ParameterExpr::Literal {
                value: fallback.into(),
            })),
        })
    }

    pub fn source<F, V>(source: F, fallback: impl Into<Value>) -> Self
    where
        F: Fn() -> Option<V> + Send + Sync + 'static,
        V: Into<Value>,
    {
        Self::Source {
            source: Box::new(move || source().map(Into::into)),
            fallback: fallback.into(),
        }
    }

    pub fn space<I, V>(values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Value>,
    {
        Self::Expression(ParameterExpr::Array {
            values: values
                .into_iter()
                .map(|value| ParameterExpr::Literal {
                    value: value.into(),
                })
                .collect(),
        })
    }

    pub fn range(start: i64, stop: i64, step: i64) -> Self {
        Self::Expression(ParameterExpr::Range { start, stop, step })
    }

    pub fn arange(start: f64, stop: f64, step: f64) -> Self {
        Self::Expression(ParameterExpr::Arange { start, stop, step })
    }

    pub fn linspace(start: f64, stop: f64, count: usize) -> Self {
        Self::Expression(ParameterExpr::Linspace { start, stop, count })
    }

    fn into_expression(self) -> ParameterExpr {
        match self {
            Self::Expression(expression) => expression,
            Self::Source { source, fallback } => ParameterExpr::Literal {
                value: source().unwrap_or(fallback),
            },
        }
    }
}

impl From<Value> for ParameterDefault {
    fn from(value: Value) -> Self {
        Self::literal(value)
    }
}

impl<T: Into<Value>> From<Option<T>> for ParameterDefault {
    fn from(value: Option<T>) -> Self {
        Self::literal(value)
    }
}

impl From<&str> for ParameterDefault {
    fn from(value: &str) -> Self {
        Self::literal(value)
    }
}

impl From<String> for ParameterDefault {
    fn from(value: String) -> Self {
        Self::literal(value)
    }
}

impl From<bool> for ParameterDefault {
    fn from(value: bool) -> Self {
        Self::literal(value)
    }
}

impl From<i64> for ParameterDefault {
    fn from(value: i64) -> Self {
        Self::literal(value)
    }
}

impl From<i32> for ParameterDefault {
    fn from(value: i32) -> Self {
        Self::literal(value)
    }
}

impl From<f64> for ParameterDefault {
    fn from(value: f64) -> Self {
        Self::literal(value)
    }
}

impl From<ParameterExpr> for ParameterDefault {
    fn from(value: ParameterExpr) -> Self {
        Self::expression(value)
    }
}

impl<T: Into<Value>> From<Vec<T>> for ParameterDefault {
    fn from(value: Vec<T>) -> Self {
        Self::literal(Value::from(value))
    }
}

impl Step {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            identifier: identifier.into(),
            label: label.into(),
            description: None,
            repeatable: false,
        }
    }

    pub fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }
}

impl TryFrom<Step> for ExecutionPhaseDefinition {
    type Error = TemplateAuthoringError;

    fn try_from(step: Step) -> Result<Self, Self::Error> {
        Ok(Self {
            step_id: StepId::from_seed(&step.identifier),
            repeatable: step.repeatable,
            metadata: Metadata::new(
                validated_identifier("step", step.identifier)?,
                step.label,
                step.description,
            ),
        })
    }
}

impl<T, C, E> StepBinding<T, C, E> {
    pub fn new<F>(identifier: impl Into<String>, label: impl Into<String>, action: F) -> Self
    where
        F: Fn(&mut T, &mut C) -> Result<(), E> + Send + Sync + 'static,
    {
        Self {
            step: Step::new(identifier, label),
            action: Box::new(action),
        }
    }

    pub fn with_step<F>(step: Step, action: F) -> Self
    where
        F: Fn(&mut T, &mut C) -> Result<(), E> + Send + Sync + 'static,
    {
        Self {
            step,
            action: Box::new(action),
        }
    }

    pub fn step(&self) -> &Step {
        &self.step
    }

    pub fn into_step(self) -> Step {
        self.step
    }

    pub fn run(&self, template: &mut T, ctx: &mut C) -> Result<(), E> {
        (self.action)(template, ctx)
    }
}

/* === Public Functions === */
pub fn criterion(
    identifier_value: impl Into<String>,
    label: impl Into<String>,
    expected: Option<Value>,
) -> Criterion {
    Criterion {
        identifier: identifier_value.into(),
        label: label.into(),
        description: None,
        expected,
    }
}

pub fn metadata(
    identifier_value: impl Into<String>,
    label: impl Into<String>,
    description: impl Into<Option<String>>,
) -> AuthoredMetadata {
    AuthoredMetadata {
        identifier: identifier_value.into(),
        label: label.into(),
        description: description.into(),
    }
}

pub fn unit(symbol: impl Into<String>) -> MeasurementUnit {
    let symbol = symbol.into();
    let sanitized = symbol
        .replace('%', "percent")
        .replace('.', "_")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || character == '_'
                || character == '-'
                || character == '.'
            {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    MeasurementUnit {
        metadata: Metadata::new(
            // Sanitization keeps only valid identifier characters and the
            // "unit." prefix guarantees a valid start, so this cannot fail.
            Identifier::new(format!("unit.{sanitized}"))
                .expect("sanitized unit identifier is valid"),
            symbol.clone(),
            None,
        ),
        family: match symbol.as_str() {
            "s" | "min" => UnitFamily::Duration,
            "%" => UnitFamily::Ratio,
            _ => UnitFamily::Custom,
        },
        symbol,
    }
}

pub fn format(value: ValueFormat) -> ValueFormat {
    if let Err(message) = validate_value_format(&value) {
        panic!("invalid value format: {message}");
    }

    value
}

/* === Private Functions === */
fn validated_identifier(
    field: &'static str,
    value: String,
) -> Result<Identifier, TemplateAuthoringError> {
    Identifier::new(value.clone()).map_err(|reason| TemplateAuthoringError::InvalidIdentifier {
        field,
        value,
        reason: reason.into(),
    })
}

/* === Tests === */
