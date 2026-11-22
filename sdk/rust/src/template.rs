/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Composed Templates and Generated Execution Operations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    sync::Arc,
};

// Third Party Imports
use ::harmos::OperationPolicy;
use ::harmos::Visibility;
use ::harmos::plugin;

// Local Imports
use crate::plugin::{
    CriteriaEvaluateResponse, ExecutionsCleanupResponse, ExecutionsInitializeResponse, StepsExecuteResponse, TestCasesMaterializeResponse,
};
use crate::wire::model;
use crate::{
    Context, Error, Message, Observer, Payload, Record, Result, Stream,
    authoring::{build_identifier, code_name},
    context::Executions,
    data,
    message::Wire,
    operation::{self, Action},
    plugin::Prompt,
    stream::{Registration, Sampler},
};
use crate::{Document, FileParameter, Metadata, Parameter, TestCase, TestCaseTemplate, Value};
use dokime_protocol::plugin::{
    CriteriaEvaluateRequest, ExecutionsCleanupRequest, ExecutionsInitializeRequest, StepsExecuteRequest, TestCasesMaterializeRequest,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(crate) type Materializer = Arc<dyn Fn(TestCase, &HashMap<String, Value>) -> Result<TestCase> + Send + Sync>;

/* === Structs === */

#[derive(Clone)]
pub struct Step {
    pub(crate) metadata: Metadata,
    pub(crate) handler: Action<()>,
    pub(crate) policy: OperationPolicy,
    repeatable: bool,
    pub(crate) replay_safe: bool,
}

#[derive(Clone)]
pub struct Criterion {
    pub(crate) metadata: Metadata,
    pub(crate) handler: Action<bool>,
}

// An Inert Template. Its Materializer Never Creates Execution Resources.
pub struct Template {
    pub(crate) marker: TypeId,
    definition: TestCaseTemplate,
    initialize: Option<Action<()>>,
    cleanup: Option<Action<()>>,
    steps: Vec<Step>,
    criteria: Vec<Criterion>,
    streams: Vec<String>,
    external_streams: HashSet<String>,
    registrations: Vec<Registration>,
    materialize: Option<Materializer>,
    samplers: Vec<Sampler>,
    errors: Vec<String>,
    pub(crate) records: Vec<crate::RecordContract>,
    pub(crate) prompts: Vec<crate::Prompt>,
}

// A Validated Template: Its Frozen Case Skeleton and the Handlers Behind Each Operation.
pub(crate) struct Parts {
    pub definition: TestCaseTemplate,
    pub case: TestCase,
    pub initialize: Option<Action<()>>,
    pub cleanup: Option<Action<()>>,
    pub steps: Vec<(String, Step)>,
    pub criteria: Vec<(String, Criterion)>,
    pub materialize: Option<Materializer>,
    pub registrations: Vec<Registration>,
    pub samplers: Vec<Sampler>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<H> From<H> for Step
where
    H: for<'a> AsyncFn(&'a Context) -> Result<()> + Send + Sync + 'static,
{
    fn from(handler: H) -> Self {
        Self::new("", handler)
    }
}

impl From<&Step> for Step {
    fn from(step: &Step) -> Self {
        step.clone()
    }
}

impl Step {
    pub fn new<H>(label: impl Into<String>, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a Context) -> Result<()> + Send + Sync + 'static,
    {
        let label = label.into();
        Self {
            metadata: Metadata {
                identifier: code_name::<H>(false),
                label,
                ..Default::default()
            },
            handler: operation::action(handler),
            policy: OperationPolicy::default(),
            repeatable: false,
            replay_safe: false,
        }
    }
    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.metadata.identifier = identifier.into();
        self
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = description.into();
        self
    }
    pub fn replay_safe(mut self) -> Self {
        self.replay_safe = true;
        self
    }
    pub fn policy(mut self, policy: OperationPolicy) -> Self {
        self.policy = policy;
        self
    }
    pub fn repeatable(mut self, repeatable: bool) -> Self {
        self.repeatable = repeatable;
        self
    }
}

impl Criterion {
    pub fn new<H>(label: impl Into<String>, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a Context) -> Result<bool> + Send + Sync + 'static,
    {
        let label = label.into();
        Self {
            metadata: Metadata {
                identifier: code_name::<H>(false),
                label,
                ..Default::default()
            },
            handler: operation::action(handler),
        }
    }
    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.metadata.identifier = identifier.into();
        self
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = description.into();
        self
    }
}

impl Template {
    pub(crate) fn id(&self) -> &str {
        &self.definition.metadata.as_ref().expect("Template Metadata").identifier
    }

    pub fn of<T: 'static>() -> Self {
        let identifier = code_name::<T>(true);
        let label = crate::authoring::type_label::<T>();
        Self {
            marker: TypeId::of::<T>(),
            definition: TestCaseTemplate {
                metadata: Some(Metadata {
                    identifier: identifier.clone(),
                    label,
                    description: String::new(),
                    ..Default::default()
                }),
                materialize: build_identifier(&identifier, "materialize", false),
                cleanup: Some(build_identifier(&identifier, "cleanup", false)),
                ..Default::default()
            },
            initialize: None,
            cleanup: None,
            steps: Vec::new(),
            criteria: Vec::new(),
            streams: Vec::new(),
            external_streams: HashSet::new(),
            registrations: Vec::new(),
            materialize: None,
            samplers: Vec::new(),
            errors: Vec::new(),
            records: Vec::new(),
            prompts: Vec::new(),
        }
    }
    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.definition.metadata.as_mut().expect("Template Metadata").identifier = identifier.into();
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.definition.metadata.as_mut().expect("Template Metadata").label = label.into();
        self
    }

    pub fn record<T: Payload>(mut self, record: Record<T>) -> Self {
        self.registrations.push(Box::new(|plugin, _, _| {
            plugin.register(|registrar| registrar.register(plugin::Descriptor::<T::Message>::new())).map_err(Error::from)
        }));
        self.records.push(record.definition);
        self
    }

    pub fn prompt(mut self, prompt: Prompt) -> Self {
        self.prompts.push(prompt.definition);
        self
    }

    pub(crate) fn bind(mut self, suite: &str) -> Self {
        if let Err(error) = validate_names([self.id()], "Template") {
            self.errors.push(error.to_string());
        }
        if let Err(error) = validate_names(
            self.steps
                .iter()
                .map(|step| step.metadata.identifier.as_str())
                .chain(self.criteria.iter().map(|criterion| criterion.metadata.identifier.as_str()))
                .chain(self.records.iter().filter_map(|record| record.metadata.as_ref().map(|metadata| metadata.identifier.as_str())))
                .chain(self.prompts.iter().filter_map(|prompt| prompt.metadata.as_ref().map(|metadata| metadata.identifier.as_str())))
                .chain(self.streams.iter().map(String::as_str)),
            "Template Member",
        ) {
            self.errors.push(error.to_string());
        }
        let local = self.id().to_owned();
        let identifier = build_identifier(suite, &local, false);
        self.definition.metadata.as_mut().expect("Template Metadata").identifier = identifier.clone();
        self.definition.materialize = build_identifier(&identifier, "materialize", false);
        self.definition.cleanup = Some(build_identifier(&identifier, "cleanup", false));
        for step in &mut self.steps {
            step.metadata.identifier = build_identifier(&identifier, &step.metadata.identifier, false);
        }
        for criterion in &mut self.criteria {
            criterion.metadata.identifier = build_identifier(&identifier, &criterion.metadata.identifier, false);
        }
        for record in &mut self.records {
            let metadata = record.metadata.as_mut().expect("Record Metadata");
            metadata.identifier = build_identifier(&identifier, &metadata.identifier, false);
        }
        for prompt in &mut self.prompts {
            let metadata = prompt.metadata.as_mut().expect("Prompt Metadata");
            metadata.identifier = build_identifier(&identifier, &metadata.identifier, false);
            prompt.template = identifier.clone();
        }
        for stream in &mut self.streams {
            if !self.external_streams.contains(stream) {
                *stream = build_identifier(&identifier, stream, false);
            }
        }
        for sampler in &mut self.samplers {
            sampler.stream = build_identifier(&identifier, &sampler.stream, false);
        }
        if let Some(materialize) = self.materialize.take() {
            let identifier = identifier.clone();
            self.materialize = Some(Arc::new(move |mut case: TestCase, parameters: &HashMap<String, Value>| {
                for step in &mut case.steps {
                    if let Some(metadata) = &mut step.metadata {
                        metadata.identifier =
                            metadata.identifier.strip_prefix(&format!("{identifier}.")).unwrap_or(&metadata.identifier).into();
                    }
                }
                for criterion in &mut case.criteria {
                    if let Some(metadata) = &mut criterion.metadata {
                        metadata.identifier =
                            metadata.identifier.strip_prefix(&format!("{identifier}.")).unwrap_or(&metadata.identifier).into();
                    }
                }
                case = materialize(case, parameters)?;
                for step in &mut case.steps {
                    if let Some(metadata) = &mut step.metadata {
                        metadata.identifier = build_identifier(&identifier, &metadata.identifier, false);
                    }
                }
                for criterion in &mut case.criteria {
                    if let Some(metadata) = &mut criterion.metadata {
                        metadata.identifier = build_identifier(&identifier, &metadata.identifier, false);
                    }
                }
                Ok(case)
            }));
        }
        self.registrations = self
            .registrations
            .into_iter()
            .map(|register| {
                let identifier = identifier.clone();
                Box::new(move |plugin, executions, _: &str| register(plugin, executions, &identifier)) as Registration
            })
            .collect();
        self
    }

    pub fn initialize<H>(mut self, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a Context) -> Result<()> + Send + Sync + 'static,
    {
        self.initialize = Some(operation::action(handler));
        self
    }
    pub fn cleanup<H>(mut self, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a Context) -> Result<()> + Send + Sync + 'static,
    {
        self.cleanup = Some(operation::action(handler));
        self
    }
    pub fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }
    pub fn criterion(mut self, criterion: Criterion) -> Self {
        self.criteria.push(criterion);
        self
    }
    pub fn stream<T: Message>(mut self, stream: Stream<T>) -> Self {
        self.streams.push(stream.identifier.clone());
        self.registrations.push(stream.registration());
        self
    }
    pub fn observer(mut self, observer: Observer) -> Self {
        self.samplers.push(observer.sampler.clone());
        self.registrations.push(observer.registration());
        self
    }
    // Subscribe to an Already Declared Stream Without Registering a Second Producer.
    pub fn uses_stream(mut self, identifier: impl Into<String>) -> Self {
        let identifier = identifier.into();
        self.external_streams.insert(identifier.clone());
        self.streams.push(identifier);
        self
    }
    pub fn parameters<T: Message>(mut self, defaults: T) -> Self {
        match data::parameters(defaults) {
            Ok(parameters) => self.definition.parameters.extend(parameters),
            Err(error) => self.errors.push(error.to_string()),
        }
        self
    }
    pub fn parameter(mut self, parameter: impl Into<Parameter>) -> Self {
        self.definition.parameters.push(parameter.into());
        self
    }
    pub fn file(mut self, file: FileParameter) -> Self {
        self.definition.files.push(file);
        self
    }
    pub fn document(mut self, document: Document) -> Self {
        self.definition.documents.push(document);
        self
    }
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.definition.tags.push(tag.into());
        self
    }
    pub fn parameter_group(mut self, group: Metadata) -> Self {
        self.definition.parameter_groups.push(group);
        self
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.definition.metadata.as_mut().expect("Template Metadata").description = description.into();
        self
    }
    pub fn materialize(
        mut self, materialize: impl Fn(TestCase, &HashMap<String, Value>) -> Result<TestCase> + Send + Sync + 'static,
    ) -> Self {
        self.materialize = Some(Arc::new(materialize));
        self
    }

    pub(crate) fn parts(mut self) -> Result<Parts> {
        if !self.errors.is_empty() {
            return Err(Error::Invalid(self.errors.join("; ")));
        }
        let identifier = self.definition.metadata.as_ref().expect("Template Metadata").identifier.clone();
        validate_names(
            self.steps
                .iter()
                .map(|step| step.metadata.identifier.as_str())
                .chain(self.criteria.iter().map(|criterion| criterion.metadata.identifier.as_str()))
                .chain(self.records.iter().filter_map(|record| record.metadata.as_ref().map(|metadata| metadata.identifier.as_str())))
                .chain(self.prompts.iter().filter_map(|prompt| prompt.metadata.as_ref().map(|metadata| metadata.identifier.as_str())))
                .chain(self.streams.iter().map(String::as_str)),
            "Template Member",
        )?;
        validate_names(self.steps.iter().map(|step| step.metadata.identifier.as_str()), "Step")?;
        validate_names(self.criteria.iter().map(|criterion| criterion.metadata.identifier.as_str()), "Criterion")?;
        validate_names(
            self.definition
                .parameters
                .iter()
                .map(|parameter| parameter.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str())),
            "Parameter",
        )?;
        validate_names(
            self.definition.files.iter().map(|file| file.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str())),
            "File Parameter",
        )?;
        validate_names(self.definition.parameter_groups.iter().map(|group| group.identifier.as_str()), "Parameter Group")?;
        validate_names(self.streams.iter().map(String::as_str), "Execution Stream")?;
        let mut case = TestCase {
            streams: self.streams,
            ..Default::default()
        };
        let mut steps = Vec::new();
        for step in self.steps {
            operation::validate_policy(&step.policy)?;
            let operation = step.metadata.identifier.clone();
            case.steps.push(model::Step {
                metadata: Some(step.metadata.clone()),
                operation: operation.clone(),
                repeatable: step.repeatable,
            });
            steps.push((operation, step));
        }
        let mut criteria = Vec::new();
        for criterion in self.criteria {
            let operation = criterion.metadata.identifier.clone();
            case.criteria.push(model::Criterion {
                metadata: Some(criterion.metadata.clone()),
                operation: operation.clone(),
            });
            criteria.push((operation, criterion));
        }
        // Declare the Procedure on the Template So Catalogs Show It Before Materialization.
        self.definition.steps = case.steps.clone();
        if self.initialize.is_none() && !case.streams.is_empty() {
            self.initialize = Some(operation::action(async |_: &Context| Ok(())));
        }
        if self.initialize.is_some() {
            self.definition.initialize = Some(build_identifier(&identifier, "initialize", false));
        }
        Ok(Parts {
            definition: self.definition,
            case,
            initialize: self.initialize,
            cleanup: self.cleanup,
            steps,
            criteria,
            materialize: self.materialize,
            registrations: self.registrations,
            samplers: self.samplers,
        })
    }

    pub(crate) fn register(self, mut plugin: plugin::Plugin, executions: Executions) -> Result<(TestCaseTemplate, plugin::Plugin)> {
        let Parts {
            definition,
            case,
            initialize,
            cleanup,
            steps,
            criteria,
            materialize,
            registrations,
            samplers: _,
        } = self.parts()?;
        let identifier = definition.metadata.as_ref().expect("Template Metadata").identifier.clone();
        for (operation, step) in steps {
            let store = executions.clone();
            let template = identifier.clone();
            let handler = operation::wire(
                operation.clone(),
                Visibility::Protected,
                async move |transport: &plugin::Context, Wire(request): Wire<StepsExecuteRequest>| {
                    let context = Context::execution(transport, &store, &request.execution, request.test_case, request.origin, &template)?;
                    let present = context.test_case()?.steps.iter().any(|step| {
                        step.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.step) && step.operation == operation
                    });
                    if !present {
                        return Err(Error::Invalid("Step Does Not Match the Frozen Test Case".into()));
                    }
                    step.handler.execute(&context).await?;
                    Ok(Wire(StepsExecuteResponse {}))
                },
            );
            let handler =
                if step.policy.timeout.is_some() || step.policy.retries.is_some() { handler.policy(step.policy) } else { handler };
            let handler = if step.replay_safe { handler.replay_safe() } else { handler };
            plugin = plugin.register(|registrar| registrar.register(handler))?;
        }
        for (operation, criterion) in criteria {
            let store = executions.clone();
            let template = identifier.clone();
            plugin = plugin.register(|registrar| {
                registrar.register(operation::wire(
                    operation.clone(),
                    Visibility::Protected,
                    async move |transport: &plugin::Context, Wire(request): Wire<CriteriaEvaluateRequest>| {
                        let context =
                            Context::execution(transport, &store, &request.execution, request.test_case, request.origin, &template)?;
                        let present = context.test_case()?.criteria.iter().any(|criterion| {
                            criterion.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.criterion)
                                && criterion.operation == operation
                        });
                        if !present {
                            return Err(Error::Invalid("Criterion Does Not Match the Frozen Test Case".into()));
                        }
                        let passed = criterion.handler.execute(&context).await?;
                        Ok(Wire(CriteriaEvaluateResponse { passed }))
                    },
                ))
            })?;
        }
        let allowed_steps = case.steps.iter().map(|step| step.operation.clone()).collect::<HashSet<_>>();
        let allowed_criteria = case.criteria.iter().map(|criterion| criterion.operation.clone()).collect::<HashSet<_>>();
        let allowed_streams = case.streams.iter().cloned().collect::<HashSet<_>>();
        let template = identifier.clone();
        plugin = plugin.register(|registrar| {
            registrar.register(operation::wire(
                definition.materialize.clone(),
                Visibility::Protected,
                async move |transport: &plugin::Context, Wire(request): Wire<TestCasesMaterializeRequest>| {
                    if transport.scope().is_ok() {
                        return Err(Error::Invalid("Materialization Requires an Empty Execution Scope".into()));
                    }
                    if request.template != template {
                        return Err(Error::Invalid("Materialization Names a Different Template".into()));
                    }
                    let mut case = case.clone();
                    case.parameters = request.parameters.clone();
                    if let Some(materialize) = &materialize {
                        case = materialize(case, &request.parameters)?;
                    }
                    if case.parameters != request.parameters {
                        return Err(Error::Invalid("Materialization Cannot Change Resolved Parameters".into()));
                    }
                    if case.steps.iter().any(|step| step.metadata.is_none())
                        || case.criteria.iter().any(|criterion| criterion.metadata.is_none())
                    {
                        return Err(Error::Invalid("Materialized Steps and Criteria Require Metadata".into()));
                    }
                    validate_names(
                        case.steps.iter().filter_map(|step| step.metadata.as_ref().map(|metadata| metadata.identifier.as_str())),
                        "Step",
                    )?;
                    validate_names(
                        case.criteria
                            .iter()
                            .filter_map(|criterion| criterion.metadata.as_ref().map(|metadata| metadata.identifier.as_str())),
                        "Criterion",
                    )?;
                    if case.steps.iter().any(|step| !allowed_steps.contains(&step.operation))
                        || case.criteria.iter().any(|criterion| !allowed_criteria.contains(&criterion.operation))
                        || case.streams.iter().any(|stream| !allowed_streams.contains(stream))
                    {
                        return Err(Error::Invalid("Materialization References an Undeclared Operation or Stream".into()));
                    }
                    Ok(Wire(TestCasesMaterializeResponse { test_case: Some(case) }))
                },
            ))
        })?;
        if let Some(initialize) = initialize {
            let operation = build_identifier(&identifier, "initialize", false);
            let store = executions.clone();
            let template = identifier.clone();
            plugin = plugin.register(|registrar| {
                registrar.register(operation::wire(
                    operation,
                    Visibility::Protected,
                    async move |transport: &plugin::Context, Wire(request): Wire<ExecutionsInitializeRequest>| {
                        let context =
                            Context::execution(transport, &store, &request.execution, request.test_case, request.origin, &template)?;
                        initialize.execute(&context).await?;
                        Ok(Wire(ExecutionsInitializeResponse {}))
                    },
                ))
            })?;
        }
        // Always Expose Cleanup So Even a Template Without a User Hook Releases Its Store.
        let template = identifier;
        let store = executions.clone();
        plugin = plugin.register(|registrar| {
            registrar.register(operation::wire(
                definition.cleanup.clone().expect("SDK Cleanup Operation"),
                Visibility::Protected,
                async move |transport: &plugin::Context, Wire(request): Wire<ExecutionsCleanupRequest>| {
                    let context = Context::execution(transport, &store, &request.execution, request.test_case, request.origin, &template)?;
                    let _end = context.end(store.clone());
                    let context = context.with_termination(request.termination)?;
                    if let Some(cleanup) = &cleanup {
                        cleanup.execute(&context).await?;
                    }
                    Ok(Wire(ExecutionsCleanupResponse {}))
                },
            ))
        })?;
        for register in registrations {
            plugin = register(plugin, executions.clone(), "")?;
        }
        Ok((definition, plugin))
    }
}

/* === Public Functions === */

pub(crate) fn validate_names<'a>(names: impl IntoIterator<Item = &'a str>, kind: &str) -> Result<()> {
    let mut seen = HashSet::new();
    for name in names {
        if name.is_empty() || !seen.insert(name) {
            return Err(Error::Invalid(format!(
                "Declare {kind}: Supply Unique Nonempty Code Identifiers, Including for Anonymous Handlers: {name}"
            )));
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::Template;
    use crate::{Parameter, parameter_expansion, parameter_expression, value};

    #[test]
    fn parameter_builders_preserve_types_expansion_and_bounds() {
        let template = Template::of::<()>()
            .identifier("builders.test")
            .label("Builders")
            .parameter(Parameter::number("voltage", "Voltage").unit("V").default(230.0).range(0.0..=300.0))
            .parameter(Parameter::count("samples", "Samples").default(u64::MAX))
            .parameter(Parameter::flag("enabled", "Enabled").default(true))
            .parameter(Parameter::sweep("loads", "Loads", [80.0, 100.0]))
            .parameter(Parameter::numbers("ladder", "Ladder", [10.0, 20.0]).items(1..=8));
        let parts = template.parts().unwrap();
        let values = parts.definition.parameters;
        assert_eq!((values[0].min, values[0].max), (Some(0.0), Some(300.0)));
        let literal = |index: usize| match values[index].expression.as_ref().unwrap().kind.as_ref().unwrap() {
            parameter_expression::Kind::Literal(value) => value.kind.clone().unwrap(),
            _ => panic!("Expected a Literal"),
        };
        assert_eq!(literal(0), value::Kind::Number(230.0));
        assert_eq!(literal(1), value::Kind::UnsignedInteger(u64::MAX));
        assert_eq!(literal(2), value::Kind::Boolean(true));
        assert!(matches!(values[3].expansion.as_ref().unwrap().kind, Some(parameter_expansion::Kind::Cartesian(true))));
        let Some(parameter_expression::Kind::Array(array)) = values[3].expression.as_ref().unwrap().kind.as_ref() else {
            panic!("Expected a Sweep");
        };
        assert_eq!(array.values.len(), 2);
        let value::Kind::List(list) = literal(4) else {
            panic!("Expected a List");
        };
        assert_eq!(list.values[1].kind, Some(value::Kind::Number(20.0)));
        assert_eq!((values[4].min_items, values[4].max_items), (Some(1), Some(8)));
    }
}
