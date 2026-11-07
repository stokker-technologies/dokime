/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Embedded Python Providers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// Third Party Imports
use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3_async_runtimes::TaskLocals;
use serde::Deserialize;
use serde_json::Value;

// Local Imports
use super::{
    callback::Callback,
    schema::{Shape, contract_schema},
    template::{PythonTemplate, TemplateBinding},
};
use dokime::harmos::{
    self,
    operations::{Codec, Contract, OperationDefinition, OperationPolicy, ReplaySafety, Visibility},
};
use dokime::runtime::{EmbeddedHost, EmbeddedObserver, EmbeddedProvider};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type NativeFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, harmos::operations::ServingError>> + Send + 'a>>;

/* === Structs === */

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Spec {
    pub id: String,
    pub version: String,
    pub description: String,
    pub operations: Vec<OperationSpec>,
    pub streams: Vec<StreamSpec>,
    pub templates: Vec<TemplateSpec>,
    pub suites: Vec<SuiteSpec>,
    pub observers: Vec<ObserverSpec>,
    pub settings: dokime::settings::Settings,
    pub settings_status: Option<String>,

    #[serde(deserialize_with = "records")]
    pub records: Vec<dokime::records::RecordContract>,

    pub prompts: Vec<dokime::prompts::PromptDefinition>,
    pub requirements: Vec<OperationSpec>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperationSpec {
    pub id: String,
    pub description: String,
    pub policy: OperationPolicy,
    pub visibility: Visibility,
    pub replay: ReplaySafety,
    pub request: Shape,
    pub response: Shape,
    pub requires: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StreamSpec {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub window: usize,
    pub row: Shape,

    #[serde(deserialize_with = "columns")]
    pub columns: Vec<dokime::telemetry::TelemetryColumn>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TemplateSpec {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub parameters: Vec<ParameterSpec>,
    pub steps: Vec<StepSpec>,
    pub criteria: Vec<CriterionSpec>,
    pub observers: Vec<ObserverSpec>,
    pub streams: Vec<String>,
    pub requires: Vec<String>,
    pub prompts: Vec<String>,
    pub document: Option<DocumentSpec>,
    pub subscriptions: Vec<dokime::definition::Subscription>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DocumentSpec {
    pub label: String,
    pub id: Option<String>,

    #[serde(default)]
    pub sections: Vec<dokime::document::DocumentSection>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ParameterSpec {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub value_type: dokime::values::ValueType,
    pub default: Value,
    pub required: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub unit: Option<String>,
    pub expansion: dokime::definition::ParameterExpansion,
    pub runtime_writable: bool,
    pub choices: Option<Vec<Value>>,
    pub list: Option<dokime::definition::ParameterList>,
    pub group_id: Option<dokime::definition::ParameterGroupId>,
}

#[derive(Clone, Deserialize)]
pub(super) struct StepSpec {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub repeatable: bool,
}

#[derive(Clone, Deserialize)]
pub(super) struct CriterionSpec {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub expected: Option<Value>,
}

#[derive(Clone, Deserialize)]
pub(super) struct ObserverSpec {
    pub id: String,
    pub label: String,
    pub interval_ms: u64,
    pub streams: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub(super) struct SuiteSpec {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub templates: Vec<String>,
    pub tags: Vec<String>,
}

pub(super) struct Compiled {
    pub catalog: dokime::InstalledPlugin,
    pub operations: Vec<(OperationSpec, Contract)>,
    pub streams: BTreeMap<String, StreamSpec>,
    pub declarations: Vec<harmos::sidecar::StreamDeclaration>,
    pub templates: Vec<(TemplateSpec, dokime::execution::TemplateDefinition)>,
    pub observers: Vec<ObserverSpec>,
    pub requirements: Vec<Contract>,
}

pub(super) struct Provider {
    pub compiled: Arc<Compiled>,
    pub callbacks: BTreeMap<String, Callback>,
    host: Arc<tokio::sync::Mutex<Option<EmbeddedHost>>>,
}

struct OperationInstance {
    callback: Callback,
    request: Shape,
    response: Shape,
    payload: Vec<u8>,
    compiled: Arc<Compiled>,
    host: Arc<tokio::sync::Mutex<Option<EmbeddedHost>>>,
    policy: OperationPolicy,
    requires: BTreeSet<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl OperationSpec {
    fn contract(&self) -> Result<Contract, String> {
        self.request.validate().map_err(message)?;
        self.response.validate().map_err(message)?;
        let definition = OperationDefinition {
            id: harmos::Identifier::new(&self.id).map_err(message)?,
            description: self.description.clone(),
            policy: self.policy,
            visibility: self.visibility,
            replay: self.replay,
            fingerprint: harmos::Fingerprint([0; 32]),
        };
        Contract::from_schema(definition, Codec::Postcard, contract_schema(&self.request, &self.response), &Default::default())
            .map_err(message)
    }
}

impl StreamSpec {
    pub fn telemetry(&self) -> dokime::telemetry::TelemetryContract {
        let mut contract = dokime::telemetry::TelemetryContract::declared(&self.id, &self.label)
            .with_storage_schema(self.row.schema.clone())
            .columns(self.columns.clone());
        contract.metadata.description = self.description.clone();
        contract
    }
}

impl Provider {
    pub fn new(compiled: Arc<Compiled>, functions: BTreeMap<String, Py<PyAny>>, locals: TaskLocals) -> Result<Self, String> {
        let mut expected = BTreeSet::new();
        for (operation, _) in &compiled.operations {
            expected.insert(format!("operation:{}", operation.id));
        }
        for observer in &compiled.observers {
            expected.insert(format!("observer:{}", observer.id));
        }
        for (template, _) in &compiled.templates {
            expected.insert(format!("template:{}:initialize", template.id));
            expected.insert(format!("template:{}:cleanup", template.id));
            for step in &template.steps {
                expected.insert(format!("template:{}:step:{}", template.id, step.id));
            }
            for criterion in &template.criteria {
                expected.insert(format!("template:{}:criterion:{}", template.id, criterion.id));
            }
            for observer in &template.observers {
                expected.insert(format!("template:{}:observer:{}", template.id, observer.id));
            }
        }
        if expected != functions.keys().cloned().collect() {
            return Err("callback registration does not match the declared catalog".into());
        }
        Python::attach(|py| {
            for (name, function) in &functions {
                if !function.bind(py).is_callable() {
                    return Err(format!("callback {name} is not callable"));
                }
            }
            Ok(())
        })?;
        let locals = Arc::new(locals);
        let callbacks = functions
            .into_iter()
            .map(|(name, function)| {
                (
                    name.clone(),
                    Callback {
                        name,
                        function: Arc::new(function),
                        locals: locals.clone(),
                    },
                )
            })
            .collect();
        Ok(Self {
            compiled,
            callbacks,
            host: Arc::new(tokio::sync::Mutex::new(None)),
        })
    }
}

#[async_trait]
impl EmbeddedProvider for Provider {
    fn catalog(&self) -> dokime::InstalledPlugin {
        self.compiled.catalog.clone()
    }
    fn streams(&self) -> Vec<harmos::sidecar::StreamDeclaration> {
        self.compiled.declarations.clone()
    }
    fn requirements(&self) -> Vec<harmos::plugin::inventory::contracts::Required> {
        self.compiled
            .requirements
            .iter()
            .cloned()
            .map(|contract| harmos::plugin::inventory::contracts::Required { contract, grants: vec![] })
            .collect()
    }
    fn operations(&self) -> Vec<harmos::operations::EncodedBinding> {
        self.compiled
            .operations
            .iter()
            .map(|(spec, contract)| {
                let request = spec.request.clone();
                let spec = spec.clone();
                let callback = self.callbacks[&format!("operation:{}", spec.id)].clone();
                let compiled = self.compiled.clone();
                let host = self.host.clone();
                // Native Operations Receive Their Serving Host In The Execution Context.
                harmos::operations::EncodedBinding::new(
                    contract.clone(),
                    move |payload| request.memory(payload),
                    move |_, payload| {
                        Ok(Box::new(OperationInstance {
                            callback: callback.clone(),
                            request: spec.request.clone(),
                            response: spec.response.clone(),
                            payload: payload.to_vec(),
                            compiled: compiled.clone(),
                            host: host.clone(),
                            policy: spec.policy,
                            requires: spec.requires.iter().cloned().collect(),
                        }))
                    },
                )
            })
            .collect()
    }
    fn observers(&self) -> Vec<EmbeddedObserver> {
        self.compiled
            .observers
            .iter()
            .map(|spec| {
                let callback = self.callbacks[&format!("observer:{}", spec.id)].clone();
                let compiled = self.compiled.clone();
                let interval = std::time::Duration::from_millis(spec.interval_ms);
                let allowed: BTreeSet<_> = spec.streams.iter().cloned().collect();
                let streams = spec
                    .streams
                    .iter()
                    .filter_map(|id| self.compiled.declarations.iter().find(|stream| &stream.id == id).cloned())
                    .collect();
                EmbeddedObserver {
                    definition: harmos::observers::ObserverDefinition {
                        id: format!("{}.{}", self.compiled.catalog.source, spec.id),
                        streams,
                        demand_keys: vec![],
                    },
                    description: spec.label.clone(),
                    run: Arc::new(move |host, mut activation| {
                        let callback = callback.clone();
                        let compiled = compiled.clone();
                        let allowed = allowed.clone();
                        Box::pin(async move {
                            let mut timer = tokio::time::interval_at(tokio::time::Instant::now() + interval, interval);
                            timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                            activation.ready();
                            let mut index = 0_u64;
                            loop {
                                tokio::select! {
                                    _ = activation.stopping() => return Ok(()),
                                    _ = timer.tick() => {}
                                }
                                let mut context = super::context::PyCallbackContext::plugin(host.clone(), compiled.clone(), None);
                                Arc::get_mut(&mut context.inner).expect("new observer context").allowed_streams = allowed.clone();
                                let work = callback.call(context, serde_json::json!({"index": index}), std::time::Duration::ZERO);
                                tokio::select! {
                                    result = work => result.map_err(|error| harmos::Error::OperationFailed {
                                        operation: callback.name.clone(),
                                        source: Arc::new(std::io::Error::other(error)),
                                    })?,
                                    _ = activation.stopping() => return Ok(()),
                                };
                                index += 1;
                            }
                        })
                    }),
                }
            })
            .collect()
    }
    async fn initialize(&self, host: EmbeddedHost) -> dokime::runtime::Result<()> {
        *self.host.lock().await = Some(host);
        Ok(())
    }
    async fn start(&self) -> dokime::runtime::Result<Vec<Arc<dyn dokime::execution::Runnable>>> {
        let host = self
            .host
            .lock()
            .await
            .clone()
            .ok_or_else(|| dokime::runtime::Error::Operation("Python provider has no activation host".into()))?;
        Ok(self
            .compiled
            .templates
            .iter()
            .map(|(spec, definition)| {
                let binding = TemplateBinding {
                    spec: spec.clone(),
                    definition: definition.clone(),
                    host: host.clone(),
                    compiled: self.compiled.clone(),
                    callbacks: self.callbacks.clone(),
                };
                let observers = spec
                    .observers
                    .iter()
                    .map(|observer| dokime::execution::DeclaredObserver::new(&observer.id, &observer.label, observer.interval_ms))
                    .collect();
                Arc::new(
                    dokime::execution::Workflow::<PythonTemplate>::declaring(
                        "python",
                        &self.compiled.catalog.source,
                        definition.clone(),
                        observers,
                    )
                    .bound(Arc::new(binding)),
                ) as Arc<dyn dokime::execution::Runnable>
            })
            .collect())
    }
    async fn stop(&self) -> dokime::runtime::Result<()> {
        self.host.lock().await.take();
        Ok(())
    }
}

impl harmos::operations::NativeInstance for OperationInstance {
    fn classify(&self, _: &(dyn std::error::Error + Send + Sync + 'static)) -> harmos::operations::FailureKind {
        harmos::operations::FailureKind::Permanent
    }
    fn initialize(&mut self, _: harmos::ExecutionContext) -> NativeFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
    fn run(&mut self, context: harmos::ExecutionContext) -> NativeFuture<'_, harmos::operations::ErasedResponse> {
        Box::pin(async move {
            let serving = context
                .resource::<harmos::operations::HostContext>()
                .map_err(|error| harmos::operations::ServingError::Native(Arc::new(error)))?;
            let host = self.host.lock().await.clone().ok_or(harmos::operations::ServingError::Uninitialized)?;
            let mut callback_context = super::context::PyCallbackContext::operation(serving, host, self.compiled.clone());
            Arc::get_mut(&mut callback_context.inner).expect("new callback context").allowed_operations = Some(self.requires.clone());
            let input = self.request.decode(&self.payload).map_err(harmos::operations::ServingError::Codec)?;
            let answer = self
                .callback
                .call(callback_context, input, self.policy.cancellation_timeout)
                .await
                .map_err(|error| harmos::operations::ServingError::Native(Arc::new(std::io::Error::other(error))))?;
            let payload = self.response.encode(&answer).map_err(harmos::operations::ServingError::Codec)?;
            Ok(harmos::Response::new(payload).into_erased())
        })
    }
    fn stop(&mut self) -> NativeFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
    fn cleanup(&mut self) -> NativeFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
    fn transferred(&self) -> bool {
        false
    }
    fn progress(&self) -> Vec<tokio::sync::watch::Receiver<Option<tokio::time::Instant>>> {
        Vec::new()
    }
}

/* === Public Functions === */

pub(super) fn compile(spec: Spec) -> Result<Compiled, String> {
    harmos::Identifier::new(&spec.id).map_err(message)?;
    spec.settings.validate().map_err(message)?;
    for observer in spec.observers.iter().chain(spec.templates.iter().flat_map(|template| &template.observers)) {
        harmos::Identifier::new(&observer.id).map_err(message)?;
        if observer.interval_ms == 0 {
            return Err(format!("observer {} needs a positive interval", observer.id));
        }
        for stream in &observer.streams {
            if !spec.streams.iter().any(|declared| &declared.id == stream) {
                return Err(format!("observer {} names undeclared stream {stream}", observer.id));
            }
        }
    }
    let mut ids = BTreeSet::new();
    let mut streams = BTreeMap::new();
    let mut declarations = Vec::new();
    for stream in spec.streams {
        if streams.contains_key(&stream.id) {
            return Err(format!("duplicate stream {}", stream.id));
        }
        stream.row.validate().map_err(message)?;
        let declaration = harmos::sidecar::StreamDeclaration {
            metadata: dokime::Metadata::new(
                harmos::Identifier::new(&stream.id).map_err(message)?,
                &stream.label,
                stream.description.clone(),
            ),
            row_metadata: vec![],
            id: stream.id.clone(),
            window: stream.window,
            row: stream.row.schema.clone(),
        };
        harmos::plugin::inventory::publications::validate(&declaration).map_err(message)?;
        declarations.push(declaration);
        streams.insert(stream.id.clone(), stream);
    }
    let operations = spec
        .operations
        .into_iter()
        .map(|operation| {
            if !ids.insert(operation.id.clone()) {
                return Err(format!("duplicate operation {}", operation.id));
            }
            let contract = operation.contract()?;
            Ok((operation, contract))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut templates = Vec::new();
    ids.clear();
    for template in spec.templates {
        if !ids.insert(template.id.clone()) {
            return Err(format!("duplicate template {}", template.id));
        }
        for id in &template.streams {
            if !streams.contains_key(id) {
                return Err(format!("template {} uses undeclared stream {id}", template.id));
            }
        }
        let definition = super::template::definition(&template, &streams, &spec.records, &spec.prompts)?;
        templates.push((template, definition));
    }
    let mut suites = Vec::new();
    ids.clear();
    let mut used = BTreeSet::new();
    for suite in spec.suites {
        if !ids.insert(suite.id.clone()) {
            return Err(format!("duplicate suite {}", suite.id));
        }
        let mut members = Vec::new();
        let mut telemetry = Vec::new();
        for id in suite.templates {
            dokime::definition::template_identifier(&suite.id, &id)
                .map_err(|_| format!("template {id} must belong to suite {}", suite.id))?;
            if !used.insert(id.clone()) {
                return Err(format!("template {id} belongs to multiple suites"));
            }
            let (_, definition) = templates
                .iter()
                .find(|(template, _)| template.id == id)
                .ok_or_else(|| format!("suite {} names undeclared template {id}", suite.id))?;
            members.push(definition.template.clone());
            telemetry.extend(definition.streams.clone());
        }
        let native_suite = dokime::definition::TestSuite {
            metadata: dokime::Metadata::new(harmos::Identifier::new(&suite.id).map_err(message)?, suite.label, suite.description),
            tags: suite.tags,
            parameter_groups: vec![],
            test_case_templates: members,
        };
        let mut definition = dokime::domain::model::plugins::TestSuiteDefinition::new(suite.id, native_suite).map_err(message)?;
        definition.workflow = Some("python".into());
        definition.record_contracts = spec.records.clone();
        definition.telemetry_contracts = telemetry;
        suites.push(definition);
    }
    if used.len() != templates.len() {
        return Err("every template must belong to one declared suite".into());
    }
    let catalog = dokime::InstalledPlugin {
        source: spec.id,
        version: spec.version,
        description: spec.description,
        enabled: true,
        suites: dokime::definition::SuiteSet::new(suites),
        settings: spec.settings,
        settings_status: spec.settings_status.map(|name| dokime::domain::model::plugins::PluginEntrypoint { name }),
        record_contracts: spec.records,
        prompts: spec.prompts,
        telemetry_contracts: streams.values().map(StreamSpec::telemetry).collect(),
        ..Default::default()
    };
    let mut requirements = BTreeMap::new();
    for required in spec.requirements {
        let contract = required.contract()?;
        if let Some((_, provided)) = operations.iter().find(|(operation, _)| operation.id == required.id) {
            if provided != &contract {
                return Err(format!("required operation {} differs from its local declaration", required.id));
            }
            continue;
        }
        if let Some(previous) = requirements.insert(required.id.clone(), contract.clone())
            && previous != contract
        {
            return Err(format!("conflicting required operation {}", required.id));
        }
    }
    Ok(Compiled {
        catalog,
        operations,
        streams,
        declarations,
        templates,
        observers: spec.observers,
        requirements: requirements.into_values().collect(),
    })
}

/* === Private Functions === */

fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn columns<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<dokime::telemetry::TelemetryColumn>, D::Error> {
    let mut values = Vec::<Value>::deserialize(deserializer)?;
    for value in &mut values {
        unit(value).map_err(serde::de::Error::custom)?;
    }
    values.into_iter().map(|value| serde_json::from_value(value).map_err(serde::de::Error::custom)).collect()
}

fn records<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<dokime::records::RecordContract>, D::Error> {
    let mut values = Vec::<Value>::deserialize(deserializer)?;
    for record in &mut values {
        if let Some(fields) = record["fields"].as_array_mut() {
            for field in fields {
                unit(field).map_err(serde::de::Error::custom)?;
            }
        }
    }
    values.into_iter().map(|value| serde_json::from_value(value).map_err(serde::de::Error::custom)).collect()
}

fn unit(value: &mut Value) -> Result<(), serde_json::Error> {
    if let Some(symbol) = value["unit"].as_str() {
        value["unit"] = serde_json::to_value(dokime::definition::authored_unit(symbol))?;
    }
    Ok(())
}

/* === Tests === */
