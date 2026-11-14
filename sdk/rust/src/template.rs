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
    collections::{HashMap, HashSet},
    sync::Arc,
};

// Third Party Imports

// Local Imports
use crate::{
    Context, Error, Message, Observer, Result, Stream,
    context::Executions,
    data,
    message::Wire,
    operation::{self, Action},
    stream::Registration,
    v1,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Materializer = Arc<dyn Fn(v1::TestCase, &HashMap<String, v1::Value>) -> Result<v1::TestCase> + Send + Sync>;

/* === Structs === */

pub struct Step {
    metadata: v1::Metadata,
    handler: Action<()>,
    policy: harmos::OperationPolicy,
    repeatable: bool,
    replay_safe: bool,
    invalid_retries: bool,
}

pub struct Criterion {
    metadata: v1::Metadata,
    handler: Action<bool>,
}

// An Inert Template. Its Materializer Never Creates Execution Resources.
pub struct Template {
    definition: v1::TestCaseTemplate,
    initialize: Option<Action<()>>,
    cleanup: Option<Action<()>>,
    steps: Vec<Step>,
    criteria: Vec<Criterion>,
    streams: Vec<String>,
    registrations: Vec<Registration>,
    materialize: Option<Materializer>,
    errors: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Step {
    pub fn new<H>(label: impl Into<String>, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a Context) -> Result<()> + Send + Sync + 'static,
    {
        let label = label.into();
        Self {
            metadata: metadata(&label),
            handler: operation::action(handler),
            policy: harmos::OperationPolicy::default(),
            repeatable: false,
            replay_safe: false,
            invalid_retries: false,
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
    // Retrying Is Host Policy; the SDK Does Not Loop or Sleep Locally.
    pub fn retries(mut self, retries: u32) -> Self {
        self.invalid_retries = retries == u32::MAX;
        self.replay_safe = retries > 0;
        self.policy.retries = (retries > 0).then_some(harmos::RetryPolicy {
            total_attempts: u64::from(retries) + 1,
            initial: Some(prost_types::Duration { seconds: 0, nanos: 100_000_000 }),
            maximum: None,
            deterministic_jitter: false,
        });
        self
    }
    pub fn replay_safe(mut self) -> Self {
        self.replay_safe = true;
        self
    }
    pub fn policy(mut self, policy: harmos::OperationPolicy) -> Self {
        self.policy = policy;
        self.invalid_retries = false;
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
            metadata: metadata(&label),
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
    pub(crate) fn identifier(&self) -> &str {
        &self.definition.metadata.as_ref().expect("Template Metadata").identifier
    }
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        let identifier = identifier.into();
        Self {
            definition: v1::TestCaseTemplate {
                metadata: Some(v1::Metadata {
                    identifier: identifier.clone(),
                    label: label.into(),
                    description: String::new(),
                    ..Default::default()
                }),
                materialize: format!("{identifier}.materialize"),
                cleanup: Some(format!("{identifier}.cleanup")),
                ..Default::default()
            },
            initialize: None,
            cleanup: None,
            steps: Vec::new(),
            criteria: Vec::new(),
            streams: Vec::new(),
            registrations: Vec::new(),
            materialize: None,
            errors: Vec::new(),
        }
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
        self.registrations.push(observer.registration());
        self
    }
    // Subscribe to an Already Declared Stream Without Registering a Second Producer.
    pub fn uses_stream(mut self, identifier: impl Into<String>) -> Self {
        self.streams.push(identifier.into());
        self
    }
    pub fn parameters<T: Message>(mut self, defaults: T) -> Self {
        match data::parameters(defaults) {
            Ok(parameters) => self.definition.parameters.extend(parameters),
            Err(error) => self.errors.push(error.to_string()),
        }
        self
    }
    pub fn parameter(mut self, parameter: v1::Parameter) -> Self {
        self.definition.parameters.push(parameter);
        self
    }
    pub fn file(mut self, file: v1::FileParameter) -> Self {
        self.definition.files.push(file);
        self
    }
    pub fn document(mut self, document: v1::Document) -> Self {
        self.definition.documents.push(document);
        self
    }
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.definition.tags.push(tag.into());
        self
    }
    pub fn parameter_group(mut self, group: v1::Metadata) -> Self {
        self.definition.parameter_groups.push(group);
        self
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.definition.metadata.as_mut().expect("Template Metadata").description = description.into();
        self
    }
    pub fn materialize(
        mut self, materialize: impl Fn(v1::TestCase, &HashMap<String, v1::Value>) -> Result<v1::TestCase> + Send + Sync + 'static,
    ) -> Self {
        self.materialize = Some(Arc::new(materialize));
        self
    }

    pub(crate) fn register(
        mut self, mut plugin: harmos::Plugin, executions: Executions, plugin_id: &str,
    ) -> Result<(v1::TestCaseTemplate, harmos::Plugin)> {
        if !self.errors.is_empty() {
            return Err(Error::Invalid(self.errors.join("; ")));
        }
        let identifier = self.definition.metadata.as_ref().expect("Template Metadata").identifier.clone();
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
        let mut case = v1::TestCase {
            streams: self.streams,
            ..Default::default()
        };
        for (index, step) in self.steps.into_iter().enumerate() {
            if step.invalid_retries {
                return Err(Error::Invalid("Step Retry Count Exceeds the Protocol's Attempt Limit".into()));
            }
            let operation = format!("{identifier}.step.{index}");
            case.steps.push(v1::Step {
                metadata: Some(step.metadata),
                operation: operation.clone(),
                repeatable: step.repeatable,
            });
            let store = executions.clone();
            let template = identifier.clone();
            let owner = plugin_id.to_owned();
            let handler = operation::wire(
                operation.clone(),
                harmos::Visibility::Protected,
                async move |transport: &harmos::Context, Wire(request): Wire<dokime_protocol::plugin::StepsExecuteRequest>| {
                    let context =
                        Context::execution(transport, &store, &request.execution, request.test_case, request.origin, &template, &owner)?;
                    let present = context.test_case()?.steps.iter().any(|step| {
                        step.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.step) && step.operation == operation
                    });
                    if !present {
                        return Err(Error::Invalid("Step Does Not Match the Frozen Test Case".into()));
                    }
                    step.handler.execute(&context).await?;
                    Ok(Wire(dokime_protocol::plugin::StepsExecuteResponse {}))
                },
            )
            .policy(step.policy);
            let handler = if step.replay_safe { handler.replay_safe() } else { handler };
            plugin = plugin.operation(handler);
        }
        for (index, criterion) in self.criteria.into_iter().enumerate() {
            let operation = format!("{identifier}.criterion.{index}");
            case.criteria.push(v1::Criterion {
                metadata: Some(criterion.metadata),
                operation: operation.clone(),
            });
            let store = executions.clone();
            let template = identifier.clone();
            let owner = plugin_id.to_owned();
            plugin = plugin.operation(operation::wire(
                operation.clone(),
                harmos::Visibility::Protected,
                async move |transport: &harmos::Context, Wire(request): Wire<dokime_protocol::plugin::CriteriaEvaluateRequest>| {
                    let context =
                        Context::execution(transport, &store, &request.execution, request.test_case, request.origin, &template, &owner)?;
                    let present = context.test_case()?.criteria.iter().any(|criterion| {
                        criterion.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.criterion)
                            && criterion.operation == operation
                    });
                    if !present {
                        return Err(Error::Invalid("Criterion Does Not Match the Frozen Test Case".into()));
                    }
                    let passed = criterion.handler.execute(&context).await?;
                    Ok(Wire(dokime_protocol::plugin::CriteriaEvaluateResponse { passed }))
                },
            ));
        }
        if self.initialize.is_none() && !case.streams.is_empty() {
            self.initialize = Some(operation::action(async |_: &Context| Ok(())));
        }
        let allowed_steps = case.steps.iter().map(|step| step.operation.clone()).collect::<HashSet<_>>();
        let allowed_criteria = case.criteria.iter().map(|criterion| criterion.operation.clone()).collect::<HashSet<_>>();
        let allowed_streams = case.streams.iter().cloned().collect::<HashSet<_>>();
        let template = identifier.clone();
        plugin = plugin.operation(operation::wire(
            self.definition.materialize.clone(),
            harmos::Visibility::Protected,
            async move |transport: &harmos::Context, Wire(request): Wire<dokime_protocol::plugin::TestCasesMaterializeRequest>| {
                if transport.scope().is_ok() {
                    return Err(Error::Invalid("Materialization Requires an Empty Execution Scope".into()));
                }
                if request.template != template {
                    return Err(Error::Invalid("Materialization Names a Different Template".into()));
                }
                let mut case = case.clone();
                case.parameters = request.parameters.clone();
                if let Some(materialize) = &self.materialize {
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
                    case.criteria.iter().filter_map(|criterion| criterion.metadata.as_ref().map(|metadata| metadata.identifier.as_str())),
                    "Criterion",
                )?;
                if case.steps.iter().any(|step| !allowed_steps.contains(&step.operation))
                    || case.criteria.iter().any(|criterion| !allowed_criteria.contains(&criterion.operation))
                    || case.streams.iter().any(|stream| !allowed_streams.contains(stream))
                {
                    return Err(Error::Invalid("Materialization References an Undeclared Operation or Stream".into()));
                }
                Ok(Wire(dokime_protocol::plugin::TestCasesMaterializeResponse { test_case: Some(case) }))
            },
        ));
        if let Some(initialize) = self.initialize {
            let operation = format!("{identifier}.initialize");
            self.definition.initialize = Some(operation.clone());
            let store = executions.clone();
            let template = identifier.clone();
            let owner = plugin_id.to_owned();
            plugin = plugin.operation(operation::wire(
                operation,
                harmos::Visibility::Protected,
                async move |transport: &harmos::Context, Wire(request): Wire<dokime_protocol::plugin::ExecutionsInitializeRequest>| {
                    let context =
                        Context::execution(transport, &store, &request.execution, request.test_case, request.origin, &template, &owner)?;
                    initialize.execute(&context).await?;
                    Ok(Wire(dokime_protocol::plugin::ExecutionsInitializeResponse {}))
                },
            ));
        }
        // Always Expose Cleanup So Even a Template Without a User Hook Releases Its Store.
        let template = identifier;
        let owner = plugin_id.to_owned();
        let store = executions.clone();
        plugin = plugin.operation(operation::wire(
            self.definition.cleanup.clone().expect("SDK Cleanup Operation"),
            harmos::Visibility::Protected,
            async move |transport: &harmos::Context, Wire(request): Wire<dokime_protocol::plugin::ExecutionsCleanupRequest>| {
                let context =
                    Context::execution(transport, &store, &request.execution, request.test_case, request.origin, &template, &owner)?;
                let _end = context.end(store.clone());
                let context = context.with_termination(request.termination)?;
                if let Some(cleanup) = &self.cleanup {
                    cleanup.execute(&context).await?;
                }
                Ok(Wire(dokime_protocol::plugin::ExecutionsCleanupResponse {}))
            },
        ));
        for register in self.registrations {
            plugin = register(plugin, executions.clone());
        }
        Ok((self.definition, plugin))
    }
}

/* === Public Functions === */

pub(crate) fn validate_names<'a>(names: impl IntoIterator<Item = &'a str>, kind: &str) -> Result<()> {
    let mut seen = HashSet::new();
    for name in names {
        if name.is_empty() || !seen.insert(name) {
            return Err(Error::Invalid(format!("Duplicate or Empty {kind}: {name}")));
        }
    }
    Ok(())
}

/* === Private Functions === */

fn metadata(label: &str) -> v1::Metadata {
    let identifier = label
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("_");
    v1::Metadata {
        identifier,
        label: label.into(),
        description: String::new(),
        ..Default::default()
    }
}

/* === Tests === */
