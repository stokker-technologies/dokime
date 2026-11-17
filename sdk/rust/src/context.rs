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
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

// Third Party Imports

// Local Imports
use crate::EvidenceStatisticsResponse;
use crate::OperationPolicy;
use crate::Record;
use crate::RecordsListResponse;
use crate::operation::FutureResult;
use crate::plugin::RecordsAppendResponse;
use crate::wire::host::EvidenceStatisticsRequest;
use crate::wire::host::evidence::{LogsAppendRequest, RecordsAppendRequest, RecordsListRequest};
use crate::{Error, Message, OperationRequest, Result, data, host, host::Wire};
use crate::{ExecutionProgressValue, PageRequest, Severity};
use crate::{Origin, Termination, TestCase, origin};
use ::harmos::plugin;
use uuid::Uuid;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Resources = Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>;
pub(crate) type Executions = Arc<Mutex<HashMap<String, Arc<Execution>>>>;

/* === Structs === */

// One operation's host channel and the resources shared by its execution.
#[derive(Clone)]
pub struct Context {
    transport: Transport,
    execution: Arc<Execution>,
    termination: Option<Termination>,
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

pub(crate) struct Watching<T: prost_reflect::ReflectMessage + Default + 'static> {
    incoming: plugin::Watching<Wire<T>>,
}
impl<T: prost_reflect::ReflectMessage + Default + 'static> Watching<T> {
    pub(crate) async fn next(&mut self) -> Result<Option<T>> {
        Ok(self.incoming.next().await?.map(|value| value.0))
    }
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

impl Execution {
    pub(crate) fn new(test_case: TestCase, origin: Origin) -> Arc<Self> {
        Arc::new(Self {
            test_case: OnceLock::from(test_case),
            origin: OnceLock::from(origin),
            resources: Mutex::new(HashMap::new()),
        })
    }
    pub(crate) fn clear(&self) -> Result<()> {
        let retired = std::mem::take(&mut *self.resources.lock().map_err(|_| Error::Invalid("Execution Resource Store Poisoned".into()))?);
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
        })
    }

    pub(crate) fn invocation(transport: &plugin::Context, executions: &Executions) -> Result<Self> {
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
        })
    }

    pub(crate) fn publication(transport: &plugin::Context, executions: &Executions) -> Result<Self> {
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
        })
    }

    pub(crate) fn local(local: Local, execution: Arc<Execution>) -> Self {
        Self {
            transport: Transport::Local(Arc::new(local)),
            execution,
            termination: None,
        }
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
            .ok_or_else(|| Error::Invalid(format!("Missing Resource: {}", std::any::type_name::<T>())))
    }

    pub fn parameters<T: Message>(&self) -> Result<T> {
        data::decode(&self.test_case()?.parameters)
    }

    pub fn parameter<T: data::FromValue>(&self, identifier: &str) -> Result<T> {
        let value =
            self.test_case()?.parameters.get(identifier).ok_or_else(|| Error::Invalid(format!("Missing Parameter: {identifier}")))?;
        T::from_value(value).map_err(|error| Error::Invalid(format!("Parameter {identifier}: {error}")))
    }

    // Route a typed operation to the host; its requirement must be declared on Plugin.
    pub async fn operation<Q: OperationRequest>(&self, request: Q) -> Result<Q::Response> {
        match &self.transport {
            Transport::Harmos(transport) => transport.call(request).await.map_err(Error::from),
            Transport::Local(_) => Err(unsimulated(Q::ID)),
        }
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
    pub(crate) async fn watch<Q: host::Request>(
        &self, request: Q, max_item_bytes: std::num::NonZeroUsize,
    ) -> Result<Watching<Q::Response>> {
        match &self.transport {
            Transport::Harmos(transport) => Ok(Watching {
                incoming: transport.watch(Wire(request), max_item_bytes).await?,
            }),
            Transport::Local(_) => Err(unsimulated(Q::ID)),
        }
    }

    pub async fn sleep(&self, duration: Duration) -> Result<()> {
        self.host().time.sleep(duration.try_into().map_err(|error| Error::Invalid(format!("Sleep: Invalid Duration: {error}")))?).await?;
        Ok(())
    }
    pub async fn now(&self) -> Result<prost_types::Timestamp> {
        self.host().time.get().await
    }
    pub async fn cancelled(&self) -> String {
        match &self.transport {
            Transport::Harmos(transport) => transport.cancelled().await,
            // The In-Process Host Never Cancels a Running Invocation.
            Transport::Local(_) => std::future::pending().await,
        }
    }

    pub async fn typed_settings<T: Message>(&self) -> Result<T> {
        let response = self.host().plugins.settings.list().await?;
        crate::data::decode_settings(response.value.ok_or_else(|| Error::Invalid("Read Settings: Missing Values".into()))?)
    }

    pub async fn record<T: Message>(&self, record: &Record<T>, value: T) -> Result<RecordsAppendResponse> {
        self.call_with(
            RecordsAppendRequest {
                execution: self.execution_id().into(),
                kind: record.identifier().into(),
                payload: value.encode_to_vec(),
                source_at: None,
                request_id: Uuid::now_v7().to_string(),
            },
            None,
        )
        .await
    }
    pub async fn records(&self, record_type: impl Into<String>, step: impl Into<String>, limit: u32) -> Result<RecordsListResponse> {
        self.call_with(
            RecordsListRequest {
                record_type: record_type.into(),
                step: step.into(),
                page: Some(PageRequest { page_size: limit, ..Default::default() }),
                ..Default::default()
            },
            None,
        )
        .await
    }
    pub async fn statistics(
        &self, stream: impl Into<String>, field: impl Into<String>, step: impl Into<String>,
    ) -> Result<EvidenceStatisticsResponse> {
        self.call_with(
            EvidenceStatisticsRequest {
                stream: stream.into(),
                field: field.into(),
                step: step.into(),
                ..Default::default()
            },
            None,
        )
        .await
    }
    pub async fn progress(&self, completed: u64, total: u64, label: impl Into<String>) -> Result<()> {
        let current = self.host().executions.progress.get().await?;
        self.host().executions.progress.set(ExecutionProgressValue { completed, total, label: label.into() }, current.revision).await?;
        Ok(())
    }
    pub async fn log(&self, severity: Severity, message: impl Into<String>) -> Result<()> {
        self.call_with(
            LogsAppendRequest {
                execution: self.execution_id().into(),
                severity: severity as i32,
                message: message.into(),
                at: None,
                request_id: Uuid::now_v7().to_string(),
            },
            None,
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

fn unsimulated(operation: &str) -> Error {
    Error::failure(
        harmos::Classification::Refused,
        "testing_host_unsupported",
        format!("Testing Host: {operation} Is Not Simulated"),
        "Exercise This Operation Against a Real Host",
    )
}

/* === Tests === */
