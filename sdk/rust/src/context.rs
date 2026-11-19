/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Context

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    any::{self, Any, TypeId},
    collections::{HashMap, HashSet},
    future,
    io::Cursor,
    mem,
    num::NonZeroUsize,
    ops::Deref,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

// Third Party Imports
use harmos::plugin;
use prost_reflect::ReflectMessage;
use prost_types::Timestamp;

// Local Imports
use crate::EvidenceStatisticsResponse;
use crate::OperationPolicy;
use crate::Record;
use crate::RecordsListResponse;
use crate::client::{ArtifactCreateOptions, mutations};
use crate::operation::FutureResult;
use crate::plugin::{CaptureMode, HostContentArtifactsCreateOptions, HostDocumentsAppendPartsOptions, HostEvidenceTraceabilityGetOptions};
use crate::plugin::{
    HostEffectsCreateOptions, HostEffectsSetOptions, HostEvidenceTraceabilitySetOptions, HostExecutionsProgressSetOptions,
};
use crate::templates::{CaptureResult, Environment};
use crate::templates::{
    DocumentAppendOptions, EffectConfirmOptions, EffectCreateOptions, LogAppendOptions, ProgressSetOptions, RecordAppendOptions,
    TraceabilitySetOptions,
};
use crate::{FileSource, Metadata, TraceabilityEntry};

use crate::plugin::{HostPromptsCreateOptions, Prompt, Step};
use crate::wire::host::EvidenceStatisticsRequest;
use crate::wire::host::evidence::{LogsAppendRequest, RecordsAppendRequest, RecordsListRequest};
use crate::{
    ArtifactMetadata,
    transfers::{self, Uploads},
};
use crate::{Error, Message, Result, data, host, host::Wire};
use crate::{ExecutionProgressValue, PageRequest, Severity};
use crate::{Origin, Termination, TestCase, origin};
use crate::{Payload, authoring::build_identifier, payload};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Resources = Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>;
/* === Structs === */

#[derive(Clone)]
pub(crate) struct Executions {
    store: Arc<Mutex<HashMap<String, Arc<Execution>>>>,
    state: Arc<dyn Any + Send + Sync>,
    secrets: Arc<HashSet<String>>,
}

// One operation's host channel and the resources shared by its execution.
#[derive(Clone)]
pub struct Context {
    transport: Transport,
    execution: Arc<Execution>,
    termination: Option<Termination>,
    state: Arc<dyn Any + Send + Sync>,
    secrets: Arc<HashSet<String>>,
}

// An In-Process Host Answering Encoded Requests by Operation Identity.
pub(crate) struct Local {
    pub host: Arc<dyn LocalHost>,
    pub scope: String,
    pub attempt: u32,
}

pub(crate) struct Execution {
    pub test_case: OnceLock<TestCase>,
    pub origin: OnceLock<Origin>,
    resources: Resources,
}

// Drop Also Runs When Cleanup Is Cancelled or Fails.
pub(crate) struct EndExecution {
    executions: Executions,
    scope: String,
    execution: Arc<Execution>,
}

pub(crate) struct Watching<T: ReflectMessage + Default + 'static> {
    incoming: plugin::Watching<Wire<T>>,
}
impl<T: ReflectMessage + Default + 'static> Watching<T> {
    pub(crate) async fn next(&mut self) -> Result<Option<T>> {
        Ok(self.incoming.next().await?.map(|value| value.0))
    }
}

/// A Typed Record Selection Within the Current Template.
pub struct RecordsQuery<'a, T: Payload> {
    context: &'a Context,
    record: &'a Record<T>,
    step: String,
}

/* === Enums === */

#[derive(Clone)]
enum Transport {
    Harmos(Box<plugin::Context>),
    Local(Arc<Local>),
}

/* === Traits === */

pub(crate) trait LocalHost: Send + Sync {
    fn call<'a>(&'a self, operation: &'static str, request: Vec<u8>) -> FutureResult<'a, Vec<u8>>;
}

/* === Implementations === */

impl Executions {
    pub(crate) fn new(state: Arc<dyn Any + Send + Sync>, secrets: HashSet<String>) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            state,
            secrets: Arc::new(secrets),
        }
    }
}
impl Deref for Executions {
    type Target = Mutex<HashMap<String, Arc<Execution>>>;
    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl Execution {
    pub(crate) fn new(test_case: TestCase, origin: Origin) -> Arc<Self> {
        Arc::new(Self {
            test_case: OnceLock::from(test_case),
            origin: OnceLock::from(origin),
            resources: Mutex::new(HashMap::new()),
        })
    }
    pub(crate) fn clear(&self) -> Result<()> {
        let retired = mem::take(&mut *self.resources.lock().map_err(|_| Error::Invalid("Execution Resource Store Poisoned".into()))?);
        drop(retired);
        Ok(())
    }
}

impl Context {
    pub(crate) fn execution(
        transport: &plugin::Context, executions: &Executions, identifier: &str, test_case: Option<TestCase>, origin: Option<Origin>,
        template: &str,
    ) -> Result<Self> {
        if identifier.is_empty() || transport.scope()? != identifier {
            return Err(Error::Invalid("Execution Must Match the Inherited Harmos Scope".into()));
        }
        let test_case = test_case.ok_or_else(|| Error::Invalid("Execution Call Requires Its Test Case".into()))?;
        let origin = origin.ok_or_else(|| Error::Invalid("Execution Call Requires Its Origin".into()))?;
        let Some(origin::Kind::PluginRun(run)) = origin.kind.as_ref() else {
            return Err(Error::Invalid("Plugin Operations Require a PluginRun Origin".into()));
        };
        if run.template != template || run.plugin.is_empty() || run.test_case.as_ref() != Some(&test_case) {
            return Err(Error::Invalid("Execution Origin Requires an Entry and Matching Template and Test Case".into()));
        }
        let state = executions.state.clone();
        let secrets = executions.secrets.clone();
        let mut executions = executions.lock().map_err(|_| Error::Invalid("Execution Store Poisoned".into()))?;
        let execution = executions.entry(identifier.into()).or_insert_with(|| {
            Arc::new(Execution {
                test_case: OnceLock::new(),
                origin: OnceLock::new(),
                resources: Mutex::new(HashMap::new()),
            })
        });
        if execution.test_case.get().is_some_and(|existing| existing != &test_case)
            || execution.origin.get().is_some_and(|existing| existing != &origin)
        {
            return Err(Error::Invalid("Execution Definition Changed Between Calls".into()));
        }
        execution.test_case.get_or_init(|| test_case);
        execution.origin.get_or_init(|| origin);
        Ok(Self {
            transport: Transport::Harmos(Box::new(transport.clone())),
            execution: execution.clone(),
            termination: None,
            state,
            secrets,
        })
    }

    pub(crate) fn invocation(transport: &plugin::Context, executions: &Executions) -> Result<Self> {
        let state = executions.state.clone();
        let secrets = executions.secrets.clone();
        let executions = executions.lock().map_err(|_| Error::Invalid("Execution Store Poisoned".into()))?;
        let execution = executions.get(transport.scope().unwrap_or_default()).cloned().unwrap_or_else(|| {
            Arc::new(Execution {
                test_case: OnceLock::new(),
                origin: OnceLock::new(),
                resources: Mutex::new(HashMap::new()),
            })
        });
        Ok(Self {
            transport: Transport::Harmos(Box::new(transport.clone())),
            execution,
            termination: None,
            state,
            secrets,
        })
    }

    pub(crate) fn publication(transport: &plugin::Context, executions: &Executions) -> Result<Self> {
        let state = executions.state.clone();
        let secrets = executions.secrets.clone();
        let mut executions = executions.lock().map_err(|_| Error::Invalid("Execution Store Poisoned".into()))?;
        if transport.scope().is_err() {
            executions.entry(String::new()).or_insert_with(|| {
                Arc::new(Execution {
                    test_case: OnceLock::new(),
                    origin: OnceLock::new(),
                    resources: Mutex::new(HashMap::new()),
                })
            });
        }
        let execution = executions.get(transport.scope().unwrap_or_default()).ok_or_else(|| {
            Error::Invalid(format!("Start Execution {} Before Requesting Its Samples", transport.scope().unwrap_or_default()))
        })?;
        Ok(Self {
            transport: Transport::Harmos(Box::new(transport.clone())),
            execution: execution.clone(),
            termination: None,
            state,
            secrets,
        })
    }

    pub(crate) fn local(local: Local, execution: Arc<Execution>, state: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            transport: Transport::Local(Arc::new(local)),
            execution,
            termination: None,
            state,
            secrets: Arc::new(HashSet::new()),
        }
    }

    pub fn plugin<T: Send + Sync + 'static>(&self) -> Result<&T> {
        self.state
            .downcast_ref()
            .ok_or_else(|| Error::Invalid(format!("Read Plugin State: Supply {} With Plugin::state", any::type_name::<T>())))
    }
    pub async fn secret(&self, identifier: &str) -> Result<Vec<u8>> {
        if !self.secrets.contains(identifier) {
            return Err(Error::Invalid(format!("Read Secret: Declare {identifier} on the Plugin")));
        }
        Ok(self.host().secrets.get(identifier.into()).await?.secret)
    }
    pub fn execution_id(&self) -> &str {
        match &self.transport {
            Transport::Harmos(transport) => transport.scope().unwrap_or_default(),
            Transport::Local(local) => &local.scope,
        }
    }
    pub fn termination(&self) -> Option<Termination> {
        self.termination
    }
    pub(crate) fn with_termination(mut self, termination: i32) -> Result<Self> {
        self.termination = Some(Termination::try_from(termination).map_err(|_| Error::Invalid("Invalid Cleanup Termination".into()))?);
        Ok(self)
    }
    pub fn attempt(&self) -> u32 {
        match &self.transport {
            Transport::Harmos(transport) => transport.attempt,
            Transport::Local(local) => local.attempt,
        }
    }
    pub fn test_case(&self) -> Result<&TestCase> {
        self.execution.test_case.get().ok_or_else(|| Error::Invalid("This Call Has No Test Case".into()))
    }
    pub fn origin(&self) -> Result<&Origin> {
        self.execution.origin.get().ok_or_else(|| Error::Invalid("This Call Has No Execution Origin".into()))
    }

    // Replace a resource of this type. Interior mutation belongs to the resource itself.
    pub fn set<T: Send + Sync + 'static>(&self, value: T) {
        let replaced =
            self.execution.resources.lock().expect("Execution Resource Store Poisoned").insert(TypeId::of::<T>(), Arc::new(value));
        drop(replaced);
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.execution
            .resources
            .lock()
            .map_err(|_| Error::Invalid("Execution Resource Store Poisoned".into()))?
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(|resource| resource.downcast().ok())
            .ok_or_else(|| Error::Invalid(format!("Missing Resource: {}", any::type_name::<T>())))
    }

    pub fn parameters<T: Message>(&self) -> Result<T> {
        data::decode(&self.test_case()?.parameters)
    }

    pub fn parameter<T: data::FromValue>(&self, identifier: &str) -> Result<T> {
        let value =
            self.test_case()?.parameters.get(identifier).ok_or_else(|| Error::Invalid(format!("Missing Parameter: {identifier}")))?;
        T::from_value(value).map_err(|error| Error::Invalid(format!("Parameter {identifier}: {error}")))
    }

    pub(crate) async fn call_with<Q: host::Request>(&self, request: Q, policy: Option<OperationPolicy>) -> Result<Q::Response> {
        match &self.transport {
            Transport::Harmos(transport) => Ok(transport.call_with(Wire(request), policy).await?.0),
            Transport::Local(local) => {
                let reply = local.host.call(Q::ID, prost::Message::encode_to_vec(&request)).await?;
                <Q::Response as prost::Message>::decode(reply.as_slice()).map_err(|error| Error::Invalid(format!("{}: {error}", Q::ID)))
            }
        }
    }
    pub(crate) async fn watch<Q: host::Request>(&self, request: Q, max_item_bytes: NonZeroUsize) -> Result<Watching<Q::Response>> {
        match &self.transport {
            Transport::Harmos(transport) => Ok(Watching {
                incoming: transport.watch(Wire(request), max_item_bytes).await?,
            }),
            Transport::Local(_) => Err(Error::failure(
                harmos::Classification::Refused,
                "testing_host_unsupported",
                format!("Testing Host: {} Is Not Simulated", Q::ID),
                "Exercise This Operation Against a Real Host",
            )),
        }
    }

    pub async fn sleep(&self, duration: Duration) -> Result<()> {
        self.host().time.sleep(duration.try_into().map_err(|error| Error::Invalid(format!("Sleep: Invalid Duration: {error}")))?).await?;
        Ok(())
    }
    pub async fn now(&self) -> Result<Timestamp> {
        self.host().time.get().await
    }
    pub async fn cancelled(&self) -> String {
        match &self.transport {
            Transport::Harmos(transport) => transport.cancelled().await,
            // The In-Process Host Never Cancels a Running Invocation.
            Transport::Local(_) => future::pending().await,
        }
    }

    pub async fn settings<T: Message>(&self) -> Result<T> {
        let response = self.host().plugins.settings.list().await?;
        crate::data::decode_settings(response.value.ok_or_else(|| Error::Invalid("Read Settings: Missing Values".into()))?)
    }

    pub fn resolve(&self, local: &str) -> Result<String> {
        let Some(origin::Kind::PluginRun(run)) = self.origin()?.kind.as_ref() else {
            return Err(Error::Invalid("Resolve Declaration: Call From a Template Operation".into()));
        };
        if local.is_empty() {
            return Err(Error::Invalid("Resolve Declaration: Supply a Nonempty Local Identifier".into()));
        }
        Ok(build_identifier(&run.template, local, false))
    }

    pub async fn prompt(&self, question: impl Into<Prompt>, choices: &[&str]) -> Result<String> {
        let mut prompt = question.into();
        if choices.iter().collect::<HashSet<_>>().len() != choices.len() {
            return Err(Error::Invalid("Create Prompt: Supply Distinct Operator Choice Labels".into()));
        }
        if !choices.is_empty() {
            prompt.definition.actions.clear();
            for (index, label) in choices.iter().enumerate() {
                prompt = prompt.action(index.to_string(), *label);
            }
        }
        let mut definition = prompt.definition.clone();
        let metadata = definition.metadata.as_mut().expect("Prompt Metadata");
        metadata.identifier = self.resolve(&metadata.identifier)?;
        let identity = metadata.identifier.clone();
        let options = HostPromptsCreateOptions {
            definition: Some(definition.clone()),
            ..Default::default()
        };
        let answer = self
            .host()
            .prompts
            .create_with(identity.clone(), if prompt.declared { identity } else { String::new() }, HashMap::new(), options)
            .await?;
        if !answer.accepted {
            return Err(Error::Invalid("Create Prompt: Choose an Accepted Operator Answer".into()));
        }
        definition
            .actions
            .iter()
            .find(|action| action.identifier == answer.action)
            .map(|action| action.label.clone())
            .ok_or_else(|| Error::Invalid("Read Prompt Answer: Choose One of the Declared Actions".into()))
    }

    pub async fn record<T: Payload>(&self, record: &Record<T>, value: T) -> Result<u64> {
        self.record_with(record, value, RecordAppendOptions::default()).await
    }
    pub async fn record_with<T: Payload>(&self, record: &Record<T>, value: T, options: RecordAppendOptions) -> Result<u64> {
        let kind = self.resolve(record.identifier())?;
        self.call_with(
            RecordsAppendRequest {
                execution: self.execution_id().into(),
                kind,
                payload: prost::Message::encode_to_vec(&value.into_message()),
                source_at: None,
                request_id: mutations::request_id(options.request_id)?,
            },
            None,
        )
        .await
        .map(|reply| reply.ordinal)
    }

    pub fn records<'a, T: Payload>(&'a self, record: &'a Record<T>) -> RecordsQuery<'a, T> {
        RecordsQuery {
            context: self,
            record,
            step: String::new(),
        }
    }

    pub async fn statistics(
        &self, stream: impl AsRef<str>, field: impl Into<String>, step: impl Into<String>,
    ) -> Result<EvidenceStatisticsResponse> {
        let step = step.into();
        self.call_with(
            EvidenceStatisticsRequest {
                stream: if self.test_case()?.streams.iter().any(|candidate| candidate == stream.as_ref()) {
                    stream.as_ref().into()
                } else {
                    self.resolve(stream.as_ref())?
                },
                field: field.into(),
                step: if step.is_empty() { step } else { self.resolve(&step)? },
                ..Default::default()
            },
            None,
        )
        .await
    }
    pub async fn progress(&self, completed: u64, total: u64, label: impl Into<String>) -> Result<()> {
        self.progress_with(completed, total, label, ProgressSetOptions::default()).await
    }
    pub async fn progress_with(&self, completed: u64, total: u64, label: impl Into<String>, options: ProgressSetOptions) -> Result<()> {
        let current = self.host().executions.progress.get().await?;
        self.host()
            .executions
            .progress
            .set_with(
                ExecutionProgressValue { completed, total, label: label.into() },
                current.revision,
                HostExecutionsProgressSetOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }
    pub async fn log(&self, severity: Severity, message: impl Into<String>) -> Result<()> {
        self.log_with(severity, message, LogAppendOptions::default()).await
    }
    pub async fn log_with(&self, severity: Severity, message: impl Into<String>, options: LogAppendOptions) -> Result<()> {
        self.call_with(
            LogsAppendRequest {
                execution: self.execution_id().into(),
                severity: severity as i32,
                message: message.into(),
                at: None,
                request_id: mutations::request_id(options.request_id)?,
            },
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn capture(&self, source: FileSource, output: impl Into<String>) -> Result<CaptureResult> {
        let reply = self.host().files.capture(source, output.into()).await?;
        Ok(CaptureResult {
            artifacts: reply.artifacts,
            gaps: reply.gaps,
        })
    }
    pub async fn artifact(&self, name: &str, data: Vec<u8>, mime: &str) -> Result<ArtifactMetadata> {
        self.artifact_with(name, data, mime, ArtifactCreateOptions::default()).await
    }
    pub async fn artifact_with(&self, name: &str, data: Vec<u8>, mime: &str, options: ArtifactCreateOptions) -> Result<ArtifactMetadata> {
        let request_id = mutations::request_id(options.request_id)?;
        let length = data.len() as u64;
        let file = transfers::upload(Uploads::Template(self), Cursor::new(data), length, &request_id).await?;
        let reply = self
            .host()
            .content
            .artifacts
            .create_with(
                file,
                name.into(),
                mime.into(),
                HostContentArtifactsCreateOptions {
                    request_id: Some(format!("{request_id}/attach")),
                    ..Default::default()
                },
            )
            .await?;
        reply.artifact.ok_or_else(|| Error::Invalid("Attach Artifact: Return Its Acknowledged Metadata".into()))
    }
    pub async fn start_capture(&self, source: FileSource, output: impl Into<String>, mode: CaptureMode) -> Result<u64> {
        self.host().files.captures.start(source, output.into(), mode).await
    }
    pub async fn stop_capture(&self, capture_id: u64) -> Result<CaptureResult> {
        let reply = self.host().files.captures.stop(capture_id).await?;
        Ok(CaptureResult {
            artifacts: reply.artifacts,
            gaps: reply.gaps,
        })
    }
    pub async fn environment(&self) -> Result<Environment> {
        let reply = self.host().backend.environment().await?;
        Ok(Environment {
            backend_pid: reply.backend_pid,
            in_step: reply.in_step,
        })
    }
    pub async fn traceability(
        &self, identifier: impl Into<String>, value: crate::Value, label: impl Into<String>, source: impl Into<String>,
        unit: impl Into<String>,
    ) -> Result<()> {
        self.traceability_with(
            identifier,
            value,
            TraceabilitySetOptions {
                label: label.into(),
                source: source.into(),
                unit: unit.into(),
                ..Default::default()
            },
        )
        .await
    }
    pub async fn traceability_with(
        &self, identifier: impl Into<String>, value: crate::Value, options: TraceabilitySetOptions,
    ) -> Result<()> {
        let identifier = identifier.into();
        let host = self.host();
        let current = host
            .evidence
            .traceability
            .get_with(HostEvidenceTraceabilityGetOptions {
                entry: Some(identifier.clone()),
                ..Default::default()
            })
            .await?;
        let entry = TraceabilityEntry {
            metadata: Some(Metadata {
                identifier: identifier.clone(),
                label: options.label,
                ..Default::default()
            }),
            value: Some(value),
            source: options.source,
            unit: options.unit,
        };
        host.evidence
            .traceability
            .set_with(
                entry,
                current.revision,
                identifier,
                HostEvidenceTraceabilitySetOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }
    pub async fn document(
        &self, document: impl Into<String>, part: crate::templates::DocumentPart, section: impl Into<String>,
    ) -> Result<()> {
        self.document_with(document, part, section, DocumentAppendOptions::default()).await
    }
    pub async fn document_with(
        &self, document: impl Into<String>, part: crate::templates::DocumentPart, section: impl Into<String>,
        mutation: DocumentAppendOptions,
    ) -> Result<()> {
        let mut options = HostDocumentsAppendPartsOptions {
            request_id: Some(mutations::request_id(mutation.request_id)?),
            ..Default::default()
        };
        match part {
            crate::templates::DocumentPart::Section(value) => options.subsection = Some(*value),
            crate::templates::DocumentPart::Card(value) => options.card = Some(*value),
        }
        self.host().documents.append_parts_with(self.execution_id().into(), document.into(), section.into(), options).await?;
        Ok(())
    }
    pub async fn prepare_effect<T: Message>(
        &self, resource: impl Into<String>, inverse_operation: impl Into<String>, inverse: T, description: impl Into<String>,
    ) -> Result<String> {
        self.prepare_effect_with(resource, inverse_operation, inverse, description, EffectCreateOptions::default()).await
    }
    pub async fn prepare_effect_with<T: Message>(
        &self, resource: impl Into<String>, inverse_operation: impl Into<String>, inverse: T, description: impl Into<String>,
        options: EffectCreateOptions,
    ) -> Result<String> {
        let reply = self
            .host()
            .effects
            .create_with(
                resource.into(),
                inverse_operation.into(),
                prost::Message::encode_to_vec(&inverse),
                description.into(),
                HostEffectsCreateOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    ..Default::default()
                },
            )
            .await?;
        if reply.name.is_empty() {
            return Err(Error::Invalid("Prepare Effect: Return Its Recovery Ticket Before Applying the Effect".into()));
        }
        Ok(reply.name)
    }
    pub async fn confirm_effect(&self, ticket: &str, at: Option<Timestamp>, restored: bool) -> Result<()> {
        self.confirm_effect_with(ticket, at, restored, EffectConfirmOptions::default()).await
    }
    pub async fn confirm_effect_with(
        &self, ticket: &str, at: Option<Timestamp>, restored: bool, options: EffectConfirmOptions,
    ) -> Result<()> {
        let host = self.host();
        let current = host.effects.get(ticket.into()).await?;
        host.effects
            .set_with(
                ticket.into(),
                crate::plugin::EffectWritable {
                    applied_at: Some(match at {
                        Some(at) => at,
                        None => self.now().await?,
                    }),
                    restored,
                },
                current.revision,
                HostEffectsSetOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }

    pub(crate) fn end(&self, executions: Executions) -> EndExecution {
        EndExecution {
            executions,
            scope: self.execution_id().into(),
            execution: self.execution.clone(),
        }
    }
}

impl<T: Payload> RecordsQuery<'_, T> {
    pub fn step(mut self, step: impl Into<Step>) -> Self {
        self.step = step.into().metadata.identifier;
        self
    }

    pub async fn list(&self, limit: u32) -> Result<RecordsListResponse> {
        let step = if self.step.is_empty() { String::new() } else { self.context.resolve(&self.step)? };
        self.context
            .call_with(
                RecordsListRequest {
                    record_type: self.context.resolve(self.record.identifier())?,
                    step,
                    page: Some(PageRequest { page_size: limit, ..Default::default() }),
                    ..Default::default()
                },
                None,
            )
            .await
    }

    pub async fn last(&self) -> Result<T> {
        let record = self.list(1).await?.last.ok_or_else(|| Error::Invalid("Read Record: Capture a Value Before Reading It".into()))?;
        payload::decode(&record.payload)
    }
}

impl Drop for EndExecution {
    fn drop(&mut self) {
        let mut executions = self.executions.lock().expect("Execution Store Poisoned");
        if executions.get(&self.scope).is_some_and(|current| Arc::ptr_eq(current, &self.execution)) {
            executions.remove(&self.scope);
        }
        drop(executions);
        self.execution.clear().expect("Execution Resource Store Poisoned");
    }
}

/* === Public Functions === */

pub(crate) fn clear(executions: &Executions) -> Result<()> {
    let retired = executions
        .lock()
        .map_err(|_| Error::Invalid("Execution Store Poisoned".into()))?
        .drain()
        .map(|(_, execution)| execution)
        .collect::<Vec<_>>();
    for execution in retired {
        execution.clear()?;
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use prost::Message;
    use prost_types::Timestamp;

    use super::{Context, Execution, Local, LocalHost};
    use crate::operation::FutureResult;
    use crate::templates::{
        DocumentAppendOptions, DocumentPart, EffectConfirmOptions, EffectCreateOptions, LogAppendOptions, ProgressSetOptions,
        RecordAppendOptions, TraceabilitySetOptions,
    };
    use crate::wire::host::evidence::{
        LogsAppendRequest, LogsAppendResponse, RecordsAppendRequest, RecordsAppendResponse, TraceabilityGetResponse,
        TraceabilitySetRequest, TraceabilitySetResponse,
    };
    use crate::wire::host::executions::{ProgressGetResponse, ProgressSetRequest, ProgressSetResponse};
    use crate::wire::host::{
        DocumentsAppendPartsRequest, DocumentsAppendPartsResponse, EffectsCreateRequest, EffectsCreateResponse, EffectsGetResponse,
        EffectsSetRequest, EffectsSetResponse,
    };
    use crate::{Error, Origin, PluginRun, Record, Result, Severity, TestCase, Value, origin};

    #[derive(Default)]
    struct Host(Mutex<Vec<String>>);

    impl LocalHost for Host {
        fn call<'a>(&'a self, operation: &'static str, request: Vec<u8>) -> FutureResult<'a, Vec<u8>> {
            Box::pin(async move {
                let decode = |error| Error::Invalid(format!("Decode Context Fixture: {error}"));
                let (identity, response) = match operation {
                    "dokime.host.v1.evidence.Records/Append" => {
                        let request = RecordsAppendRequest::decode(request.as_slice()).map_err(decode)?;
                        assert_eq!(request.execution, "execution");
                        (request.request_id, RecordsAppendResponse { ordinal: 1, ..Default::default() }.encode_to_vec())
                    }
                    "dokime.host.v1.executions.Progress/Get" => return Ok(ProgressGetResponse::default().encode_to_vec()),
                    "dokime.host.v1.executions.Progress/Set" => {
                        let request = ProgressSetRequest::decode(request.as_slice()).map_err(decode)?;
                        (request.request_id, ProgressSetResponse::default().encode_to_vec())
                    }
                    "dokime.host.v1.evidence.Logs/Append" => {
                        let request = LogsAppendRequest::decode(request.as_slice()).map_err(decode)?;
                        assert_eq!(request.execution, "execution");
                        (request.request_id, LogsAppendResponse::default().encode_to_vec())
                    }
                    "dokime.host.v1.evidence.Traceability/Get" => return Ok(TraceabilityGetResponse::default().encode_to_vec()),
                    "dokime.host.v1.evidence.Traceability/Set" => {
                        let request = TraceabilitySetRequest::decode(request.as_slice()).map_err(decode)?;
                        (request.request_id, TraceabilitySetResponse::default().encode_to_vec())
                    }
                    "dokime.host.v1.Documents/AppendParts" => {
                        let request = DocumentsAppendPartsRequest::decode(request.as_slice()).map_err(decode)?;
                        (request.request_id, DocumentsAppendPartsResponse { receipt: Some(Default::default()) }.encode_to_vec())
                    }
                    "dokime.host.v1.Effects/Create" => {
                        let request = EffectsCreateRequest::decode(request.as_slice()).map_err(decode)?;
                        (
                            request.request_id,
                            EffectsCreateResponse {
                                name: "ticket".into(),
                                ..Default::default()
                            }
                            .encode_to_vec(),
                        )
                    }
                    "dokime.host.v1.Effects/Get" => return Ok(EffectsGetResponse::default().encode_to_vec()),
                    "dokime.host.v1.Effects/Set" => {
                        let request = EffectsSetRequest::decode(request.as_slice()).map_err(decode)?;
                        (request.request_id, EffectsSetResponse::default().encode_to_vec())
                    }
                    _ => panic!("Unexpected Context Operation: {operation}"),
                };
                self.0.lock().unwrap().push(identity);
                Ok(response)
            })
        }
    }

    #[tokio::test]
    async fn context_mutations_preserve_explicit_request_identities() -> Result<()> {
        let host = Arc::new(Host::default());
        let origin = Origin {
            kind: Some(origin::Kind::PluginRun(PluginRun {
                template: "capture.confirmation".into(),
                ..Default::default()
            })),
        };
        let context = Context::local(
            Local {
                host: host.clone(),
                scope: "execution".into(),
                attempt: 1,
            },
            Execution::new(TestCase::default(), origin),
            Arc::new(()),
        );

        context
            .record_with(&Record::<bool>::new("result", "Result"), true, RecordAppendOptions { request_id: Some("record".into()) })
            .await?;
        context.progress_with(1, 2, "", ProgressSetOptions { request_id: Some("progress".into()) }).await?;
        context.log_with(Severity::Info, "", LogAppendOptions { request_id: Some("log".into()) }).await?;
        context
            .traceability_with(
                "serial",
                Value::default(),
                TraceabilitySetOptions {
                    request_id: Some("traceability".into()),
                    ..Default::default()
                },
            )
            .await?;
        context
            .document_with("report", DocumentPart::Card(Box::default()), "", DocumentAppendOptions { request_id: Some("document".into()) })
            .await?;
        let ticket = context
            .prepare_effect_with("relay", "relay.restore", (), "", EffectCreateOptions { request_id: Some("prepare".into()) })
            .await?;
        context
            .confirm_effect_with(&ticket, Some(Timestamp::default()), false, EffectConfirmOptions { request_id: Some("confirm".into()) })
            .await?;
        assert_eq!(*host.0.lock().unwrap(), ["record", "progress", "log", "traceability", "document", "prepare", "confirm"]);
        Ok(())
    }
}
