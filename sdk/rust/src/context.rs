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
use crate::{Error, Message, OperationRequest, Result, data, host, message::Wire, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Resources = Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>;
pub(crate) type Executions = Arc<Mutex<HashMap<String, Arc<Execution>>>>;

/* === Structs === */

// One operation's host channel and the resources shared by its execution.
pub struct Context {
    transport: harmos::Context,
    execution: Arc<Execution>,
    termination: Option<v1::Termination>,
}

pub(crate) struct Execution {
    pub test_case: OnceLock<v1::TestCase>,
    pub origin: OnceLock<v1::Origin>,
    resources: Resources,
}

// Drop Also Runs When Cleanup Is Cancelled or Fails.
pub(crate) struct EndExecution {
    executions: Executions,
    scope: String,
    execution: Arc<Execution>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Execution {
    fn clear(&self) -> Result<()> {
        let retired = std::mem::take(&mut *self.resources.lock().map_err(|_| Error::Invalid("Execution Resource Store Poisoned".into()))?);
        drop(retired);
        Ok(())
    }
}

impl Context {
    pub(crate) fn execution(
        transport: &harmos::Context, executions: &Executions, identifier: &str, test_case: Option<v1::TestCase>,
        origin: Option<v1::Origin>, template: &str, plugin: &str,
    ) -> Result<Self> {
        if identifier.is_empty() || transport.scope()? != identifier {
            return Err(Error::Invalid("Execution Must Match the Inherited Harmos Scope".into()));
        }
        let test_case = test_case.ok_or_else(|| Error::Invalid("Execution Call Requires Its Test Case".into()))?;
        let origin = origin.ok_or_else(|| Error::Invalid("Execution Call Requires Its Origin".into()))?;
        let Some(v1::origin::Kind::PluginRun(run)) = origin.kind.as_ref() else {
            return Err(Error::Invalid("Plugin Operations Require a PluginRun Origin".into()));
        };
        if run.template != template || run.plugin != plugin || run.test_case.as_ref() != Some(&test_case) {
            return Err(Error::Invalid("Execution Origin Does Not Match the Plugin, Template and Test Case".into()));
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
            transport: transport.clone(),
            execution: execution.clone(),
            termination: None,
        })
    }

    pub(crate) fn invocation(transport: &harmos::Context, executions: &Executions) -> Result<Self> {
        let executions = executions.lock().map_err(|_| Error::Invalid("Execution Store Poisoned".into()))?;
        let execution = executions.get(transport.scope().unwrap_or_default()).cloned().unwrap_or_else(|| {
            Arc::new(Execution {
                test_case: OnceLock::new(),
                origin: OnceLock::new(),
                resources: Mutex::new(HashMap::new()),
            })
        });
        Ok(Self {
            transport: transport.clone(),
            execution,
            termination: None,
        })
    }

    pub(crate) fn publication(transport: &harmos::Context, executions: &Executions) -> Result<Self> {
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
            transport: transport.clone(),
            execution: execution.clone(),
            termination: None,
        })
    }

    pub fn execution_id(&self) -> &str {
        self.transport.scope().unwrap_or_default()
    }
    pub fn termination(&self) -> Option<v1::Termination> {
        self.termination
    }
    pub(crate) fn with_termination(mut self, termination: i32) -> Result<Self> {
        self.termination = Some(v1::Termination::try_from(termination).map_err(|_| Error::Invalid("Invalid Cleanup Termination".into()))?);
        Ok(self)
    }
    pub fn attempt(&self) -> u32 {
        self.transport.attempt
    }
    pub fn test_case(&self) -> Result<&v1::TestCase> {
        self.execution.test_case.get().ok_or_else(|| Error::Invalid("This Call Has No Test Case".into()))
    }
    pub fn origin(&self) -> Result<&v1::Origin> {
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
        self.transport.operation(request).await.map_err(Error::from)
    }

    // Route a generated Dokime request without copying its fields into a parallel model.
    pub async fn host<Q: host::Request>(&self, request: Q) -> Result<Q::Response> {
        Ok(self.operation(Wire(request)).await?.0)
    }

    pub async fn sleep(&self, duration: Duration) -> Result<()> {
        self.transport.sleep(duration).await.map_err(Error::from)
    }
    pub async fn now(&self) -> Result<prost_types::Timestamp> {
        self.transport.now().await.map_err(Error::from)
    }
    pub async fn cancelled(&self) -> String {
        self.transport.cancelled().await
    }

    pub async fn prompt(&self, question: v1::OperatorQuestion) -> Result<v1::OperatorAnswer> {
        self.host(question).await
    }
    pub async fn settings(&self) -> Result<v1::SavedSettings> {
        self.host(v1::ReadSettings {}).await
    }
    pub async fn typed_settings<T: Message>(&self) -> Result<T> {
        data::decode(&self.settings().await?.values)
    }
    pub async fn secret(&self, identifier: impl Into<String>) -> Result<v1::Secret> {
        self.host(v1::ReadSecret { identifier: identifier.into() }).await
    }

    pub async fn record<T: Message>(&self, record: &crate::Record<T>, value: T) -> Result<v1::RecordReceipt> {
        self.host(v1::PublishRecord {
            execution: self.execution_id().into(),
            kind: record.identifier().into(),
            payload: value.encode_to_vec(),
            at: None,
        })
        .await
    }

    pub async fn records(&self, kind: impl Into<String>, step: impl Into<String>, limit: u32) -> Result<v1::Records> {
        self.host(v1::RecordsRequest {
            execution: self.execution_id().into(),
            kind: kind.into(),
            step: step.into(),
            limit,
        })
        .await
    }

    pub async fn statistics(&self, stream: impl Into<String>, field: impl Into<String>, step: impl Into<String>) -> Result<v1::Statistics> {
        self.host(v1::StatisticsRequest {
            execution: self.execution_id().into(),
            stream: stream.into(),
            field: field.into(),
            step: step.into(),
        })
        .await
    }

    pub async fn progress(&self, completed: u64, total: u64, label: impl Into<String>) -> Result<()> {
        self.host(v1::ReportProgress {
            execution: self.execution_id().into(),
            completed,
            total,
            label: label.into(),
        })
        .await?;
        Ok(())
    }

    pub async fn log(&self, severity: v1::Severity, message: impl Into<String>) -> Result<()> {
        self.host(v1::WriteLog {
            execution: self.execution_id().into(),
            severity: severity as i32,
            message: message.into(),
            at: None,
        })
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

/* === Tests === */
