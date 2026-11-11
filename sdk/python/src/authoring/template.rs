/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Python Templates On The Native Workflow

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use async_trait::async_trait;
use serde_json::Value;

// Local Imports
use super::{
    callback::Callback,
    context::{Action, PyCallbackContext},
    provider::{Compiled, StreamSpec, TemplateSpec},
};
use dokime::definition::{AuthoredMetadata, Criterion, Parameter, ParameterDefault, Step, TemplateContext};
use dokime::execution::{
    CleanupContext, CriterionContext, EvaluationFuture, InitializationContext, InitializationFuture, ObserverContext, SidecarTemplate,
    StepContext, StepFuture, TemplateDefinition,
};
use dokime::plugin::Diagnostic;

/* === Constants === */

/* === Macros === */

macro_rules! log {
    ($ctx:expr, $payload:expr) => {{
        let payload = &$payload;
        let message = payload["message"].as_str().unwrap_or_default();
        match payload["level"].as_str().unwrap_or("info") {
            "trace" => {
                $ctx.trace(message);
                Ok::<(), String>(())
            }
            "debug" => {
                $ctx.debug(message);
                Ok(())
            }
            "info" => {
                $ctx.info(message);
                Ok(())
            }
            "warn" => {
                $ctx.warn(message);
                Ok(())
            }
            "error" => {
                $ctx.error(message);
                Ok(())
            }
            _ => Err("invalid log level".into()),
        }
    }};
}

macro_rules! common {
    ($ctx:expr, $kind:expr, $payload:expr) => {{
        match $kind {
            "log" => Some(log!($ctx, $payload).map(|_| Value::Null)),
            "section" => Some((|| {
                let spec: super::provider::DocumentSpec = serde_json::from_value($payload.clone()).map_err(|error| error.to_string())?;
                let section = dokime::document::protobuf::decode_section(&spec.message)?;
                $ctx.document().section(section);
                Ok(Value::Null)
            })()),
            _ => None,
        }
    }};
}

/* === Type Aliases === */

/* === Structs === */

pub(super) struct PythonTemplate {
    binding: Arc<TemplateBinding>,
}

pub(super) struct TemplateBinding {
    pub spec: TemplateSpec,
    pub definition: TemplateDefinition,
    pub host: dokime::runtime::EmbeddedHost,
    pub compiled: Arc<Compiled>,
    pub callbacks: BTreeMap<String, Callback>,
}

struct Prompt {
    definition: dokime::prompts::PromptDefinition,
    values: BTreeMap<String, dokime::Value>,
}

/* === Enums === */

/* === Traits === */

#[async_trait]
trait Actions: Send {
    async fn action(&mut self, kind: &str, payload: Value) -> Result<Value, String>;
}

/* === Implementations === */

impl SidecarTemplate for PythonTemplate {
    const RUN: &'static str = "dokime.template.run";
    const SIGNAL: &'static str = "dokime.template.signal";
    type Own = ();
    type Parameters = ();

    fn definition() -> TemplateDefinition {
        unreachable!("Python workflows carry their owned definition")
    }

    fn initialize<'a>(ctx: &'a mut InitializationContext<'_, Self, (), ()>) -> InitializationFuture<'a, (Self, ())> {
        Box::pin(async move {
            let binding = Arc::new(ctx.environment().binding::<TemplateBinding>().clone());
            let (context, receiver) = callback_context(
                &binding,
                Some(ctx.environment().streams().clone()),
                ctx.execution_scope(),
                ctx.raw_parameters(),
                "initialize",
                ctx.environment().callbacks().clone(),
            )?;
            let callback = &binding.callbacks[&format!("template:{}:initialize", binding.spec.id)];
            call(callback, context, receiver, ctx, Value::Null).await?;
            Ok((Self { binding }, ()))
        })
    }
    fn run_step<'a>(&'a mut self, step: &'a str, ctx: &'a mut StepContext<'_, Self, (), ()>) -> StepFuture<'a> {
        Box::pin(async move {
            let (context, receiver) = callback_context(
                &self.binding,
                Some(ctx.environment().streams().clone()),
                ctx.execution_scope(),
                ctx.raw_parameters(),
                "step",
                ctx.environment().callbacks().clone(),
            )?;
            let callback = &self.binding.callbacks[&format!("template:{}:step:{step}", self.binding.spec.id)];
            call(callback, context, receiver, ctx, Value::Null).await.map(|_| ())
        })
    }
    fn observe<'a>(observer: &'a str, ctx: &'a mut ObserverContext<'_, Self, (), ()>) -> StepFuture<'a> {
        Box::pin(async move {
            let binding = ctx.environment().binding::<TemplateBinding>().clone();
            let tick = serde_json::json!({"index": ctx.tick().index(), "elapsed_s": ctx.tick().elapsed().as_secs_f64()});
            let (context, receiver) = callback_context(
                &binding,
                Some(ctx.environment().streams().clone()),
                ctx.execution_scope(),
                ctx.raw_parameters(),
                "observer",
                ctx.environment().callbacks().clone(),
            )?;
            let callback = &binding.callbacks[&format!("template:{}:observer:{observer}", binding.spec.id)];
            call(callback, context, receiver, ctx, tick).await.map(|_| ())
        })
    }
    fn cleanup<'a>(&'a mut self, ctx: &'a mut CleanupContext<'_, Self, (), ()>) -> StepFuture<'a> {
        Box::pin(async move {
            let (context, receiver) = callback_context(
                &self.binding,
                None,
                ctx.execution_scope(),
                ctx.raw_parameters(),
                "cleanup",
                ctx.environment().callbacks().clone(),
            )?;
            let callback = &self.binding.callbacks[&format!("template:{}:cleanup", self.binding.spec.id)];
            let input = serde_json::json!({"reason": format!("{:?}", ctx.reason())});
            call(callback, context, receiver, ctx, input).await.map(|_| ())
        })
    }
    fn evaluate<'a>(criterion: &'a str, ctx: &'a CriterionContext<'_, Self, ()>) -> EvaluationFuture<'a, bool> {
        Box::pin(async move {
            let binding = ctx.binding::<TemplateBinding>();
            let scope = ctx.execution_scope().ok_or_else(|| Diagnostic::of("criterion has no admitted execution binding"))?;
            let (mut context, mut receiver) =
                callback_context(binding, None, scope, ctx.raw_parameters(), "criterion", ctx.callbacks().clone())?;
            Arc::get_mut(&mut context.inner).expect("new criterion context").evidence = Some(ctx.evidence());
            let callback = &binding.callbacks[&format!("template:{}:criterion:{criterion}", binding.spec.id)];
            let future = callback.call(context, Value::Null, Duration::ZERO);
            tokio::pin!(future);
            loop {
                tokio::select! {
                    answer = &mut future => return answer.map_err(Diagnostic::of)?.as_bool().map(Some).ok_or_else(|| Diagnostic::of("a criterion must return bool")),
                    Some(action) = receiver.recv() => {
                        let answer = if action.kind == "log" { log!(ctx, action.payload).map(|_| Value::Null) } else { Err("criteria can read captured evidence and log only".into()) };
                        let _ = action.answer.send(answer);
                    }
                }
            }
        })
    }
}

impl Clone for TemplateBinding {
    fn clone(&self) -> Self {
        Self {
            spec: self.spec.clone(),
            definition: self.definition.clone(),
            host: self.host.clone(),
            compiled: self.compiled.clone(),
            callbacks: self.callbacks.clone(),
        }
    }
}

#[async_trait]
impl Actions for InitializationContext<'_, PythonTemplate, (), ()> {
    async fn action(&mut self, kind: &str, payload: Value) -> Result<Value, String> {
        common!(self, kind, payload).unwrap_or_else(|| Err("initialization cannot perform this action".into()))
    }
}

#[async_trait]
impl Actions for CleanupContext<'_, PythonTemplate, (), ()> {
    async fn action(&mut self, kind: &str, payload: Value) -> Result<Value, String> {
        common!(self, kind, payload).unwrap_or_else(|| Err("cleanup cannot perform this action".into()))
    }
}

#[async_trait]
impl Actions for StepContext<'_, PythonTemplate, (), ()> {
    async fn action(&mut self, kind: &str, payload: Value) -> Result<Value, String> {
        if let Some(answer) = common!(self, kind, payload) {
            return answer;
        }
        match kind {
            "record" => {
                let request = record(payload)?;
                self.publish(request).await;
                Ok(Value::Null)
            }
            "ask" => {
                let id = payload["prompt"].as_str().ok_or("prompt identity is absent")?;
                let definition = self
                    .environment()
                    .binding::<TemplateBinding>()
                    .compiled
                    .catalog
                    .prompts
                    .iter()
                    .find(|prompt| prompt.prompt_type.as_str() == id)
                    .cloned()
                    .ok_or_else(|| format!("undeclared prompt {id}"))?;
                let values = payload["values"]
                    .as_object()
                    .ok_or("prompt values must be an object")?
                    .iter()
                    .map(|(id, value)| {
                        dokime::Value::from_plain_json(value.clone()).map(|value| (id.clone(), value)).map_err(|error| error.to_string())
                    })
                    .collect::<Result<_, _>>()?;
                let response = self.ask(Prompt { definition, values }).await.map_err(|error| error.to_string())?;
                let values = response
                    .values
                    .iter()
                    .map(|(id, value)| value.to_plain_json().map(|value| (id.clone(), value)))
                    .collect::<Result<serde_json::Map<_, _>, _>>()
                    .map_err(|error| error.to_string())?;
                Ok(serde_json::json!({"action": response.action, "accepted": response.accepted, "values": values}))
            }
            _ => Err("step cannot perform this action".into()),
        }
    }
}

#[async_trait]
impl Actions for ObserverContext<'_, PythonTemplate, (), ()> {
    async fn action(&mut self, kind: &str, payload: Value) -> Result<Value, String> {
        if let Some(answer) = common!(self, kind, payload) {
            return answer;
        }
        if kind != "record" {
            return Err("observer cannot perform this action".into());
        }
        let request = record(payload)?;
        self.publish(request).await;
        Ok(Value::Null)
    }
}

impl dokime::prompts::Prompt for Prompt {
    type Response = dokime::prompts::PromptResponse;
    fn definition(&self) -> dokime::prompts::PromptDefinition {
        self.definition.clone()
    }
    fn decode(&self, response: Self::Response) -> dokime::Result<Self::Response> {
        Ok(response)
    }
    fn request(&self) -> dokime::prompts::PromptRequest {
        dokime::prompts::PromptRequest {
            prompt_id: dokime::model::identity::new_id(),
            prompt_type: self.definition.prompt_type.clone(),
            values: self.values.clone(),
            scope: Default::default(),
        }
    }
}

/* === Public Functions === */

pub(super) fn definition(
    spec: &TemplateSpec, streams: &BTreeMap<String, StreamSpec>, records: &[dokime::records::RecordContract],
    prompts: &[dokime::prompts::PromptDefinition],
) -> Result<TemplateDefinition, String> {
    let mut context = TemplateContext::new();
    context.set_metadata(AuthoredMetadata {
        identifier: spec.id.clone(),
        label: spec.label.clone(),
        description: spec.description.clone(),
    });
    context.set_tags(spec.tags.clone());
    for identifiers in [
        spec.parameters.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
        spec.steps.iter().map(|item| item.id.as_str()).collect(),
        spec.criteria.iter().map(|item| item.id.as_str()).collect(),
        spec.observers.iter().map(|item| item.id.as_str()).collect(),
    ] {
        let mut seen = BTreeSet::new();
        for id in identifiers {
            dokime::Identifier::new(id).map_err(|error| error.to_string())?;
            if !seen.insert(id) {
                return Err(format!("duplicate declaration {id} in {}", spec.id));
            }
        }
    }
    let parameters = spec
        .parameters
        .iter()
        .map(|parameter| {
            let mut native = Parameter::new(&parameter.id, &parameter.label, parameter.value_type.clone());
            native.description = parameter.description.clone();
            native.required = parameter.required;
            native.min = parameter.min;
            native.max = parameter.max;
            native.unit = parameter.unit.as_ref().map(dokime::definition::authored_unit);
            native.expansion = parameter.expansion.clone();
            native.runtime_writable = parameter.runtime_writable;
            native.list = parameter.list.clone();
            native.group_id = parameter.group_id.clone();
            native.selection = parameter
                .choices
                .as_ref()
                .map(|choices| {
                    choices
                        .iter()
                        .cloned()
                        .map(dokime::Value::from_plain_json)
                        .collect::<Result<Vec<_>, _>>()
                        .map(|choices| dokime::definition::ParameterSelection { choices })
                })
                .transpose()
                .map_err(|error| error.to_string())?;
            native.default =
                ParameterDefault::literal(dokime::Value::from_plain_json(parameter.default.clone()).map_err(|error| error.to_string())?);
            Ok(native)
        })
        .collect::<Result<Vec<_>, String>>()?;
    context.set_parameters(parameters);
    context.set_subscriptions(spec.subscriptions.clone());
    let mut ids = BTreeSet::new();
    for step in &spec.steps {
        if !ids.insert(&step.id) {
            return Err(format!("duplicate step {} in {}", step.id, spec.id));
        }
    }
    context.set_steps(
        spec.steps
            .iter()
            .map(|step| Step {
                identifier: step.id.clone(),
                label: step.label.clone(),
                description: step.description.clone(),
                repeatable: step.repeatable,
            })
            .collect(),
    );
    let criteria = spec
        .criteria
        .iter()
        .map(|criterion| {
            Ok(Criterion {
                identifier: criterion.id.clone(),
                label: criterion.label.clone(),
                description: criterion.description.clone(),
                expected: criterion.expected.clone().map(dokime::Value::from_plain_json).transpose().map_err(|error| error.to_string())?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    context.set_criteria(criteria);
    let declared_prompts = spec
        .prompts
        .iter()
        .map(|id| prompts.iter().find(|prompt| prompt.prompt_type.as_str() == id).cloned().ok_or_else(|| format!("undeclared prompt {id}")))
        .collect::<Result<Vec<_>, _>>()?;
    context.set_prompts(declared_prompts);
    if let Some(spec) = &spec.document {
        context.set_document(dokime::document::protobuf::decode_document(&spec.message)?);
    }
    let template = context.into_test_case_template().map_err(|error| error.to_string())?;
    let rows = spec.streams.iter().chain(spec.observers.iter().flat_map(|observer| &observer.streams)).collect::<BTreeSet<_>>();
    let telemetry = rows
        .into_iter()
        .map(|id| {
            streams
                .get(id)
                .map(|stream| stream.telemetry().for_template(template.test_case_template_id.clone()))
                .ok_or_else(|| format!("undeclared stream {id}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TemplateDefinition {
        template,
        records: records.to_vec(),
        streams: telemetry,
    })
}

/* === Private Functions === */

fn record(payload: Value) -> Result<dokime::domain::model::records::publication::RecordPublishRequest, String> {
    let id = payload["record_type"].as_str().ok_or("record identity is absent")?;
    let severity = serde_json::from_value(payload["severity"].clone()).map_err(|error| error.to_string())?;
    let mut request = dokime::domain::model::records::publication::RecordPublishRequest::new(
        dokime::Identifier::new(id).map_err(|error| error.to_string())?,
        severity,
    );
    request.details = payload["details"]
        .as_object()
        .ok_or("record details must be an object")?
        .iter()
        .map(|(key, value)| {
            dokime::Value::from_plain_json(value.clone()).map(|value| (key.clone(), value)).map_err(|error| error.to_string())
        })
        .collect::<Result<_, _>>()?;
    Ok(request)
}

fn callback_context(
    binding: &TemplateBinding, streams: Option<dokime::telemetry::Streams>, scope: dokime::execution::ExecutionScope,
    parameters: &BTreeMap<String, dokime::Value>, phase: &str, settlement: dokime::execution::Callbacks,
) -> Result<(PyCallbackContext, tokio::sync::mpsc::Receiver<Action>), Diagnostic> {
    let parameters = parameters
        .iter()
        .map(|(id, value)| value.to_plain_json().map(|value| (id.clone(), value)))
        .collect::<Result<serde_json::Map<_, _>, _>>()
        .map_err(|error| Diagnostic::of(error.to_string()))?;
    let allowed = binding.definition.streams.iter().map(|stream| stream.stream.to_string()).collect();
    let (mut context, receiver) = PyCallbackContext::template(
        binding.host.clone(),
        binding.compiled.clone(),
        streams,
        scope,
        Value::Object(parameters),
        phase,
        allowed,
    )
    .map_err(Diagnostic::of)?;
    Arc::get_mut(&mut context.inner).expect("new callback context").allowed_operations =
        Some(binding.spec.requires.iter().cloned().collect());
    Arc::get_mut(&mut context.inner).expect("new callback context").settlement = Some(settlement);
    Ok((context, receiver))
}

async fn call(
    callback: &Callback, context: PyCallbackContext, mut receiver: tokio::sync::mpsc::Receiver<Action>, actions: &mut impl Actions,
    input: Value,
) -> Result<Value, Diagnostic> {
    let future = callback.call(context, input, Duration::ZERO);
    tokio::pin!(future);
    loop {
        tokio::select! {
            answer = &mut future => return answer.map_err(Diagnostic::of),
            Some(action) = receiver.recv() => { let answer = actions.action(&action.kind, action.payload).await; let _ = action.answer.send(answer); }
        }
    }
}

/* === Tests === */
