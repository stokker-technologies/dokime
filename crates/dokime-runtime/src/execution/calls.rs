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
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{CallContext, ContextError, ContextValue, OperationDeclaration, OperationHost};
use harmos::sidecar::{CallError, Supervisor};
use harmos::{Message, StreamSpec};

// Local Imports
use super::builtins::{Binding, served};
use super::instances::refusal_failure;
use super::instances::{LocalRun, StepInstance, refused as step_refusal};

/* === Constants === */

// The JSON-RPC server error code of a refusal raised on the step side of the call.
const STEP_REFUSAL: i64 = -32_000;

/* === Macros === */

/* === Type Aliases === */

pub type Answering = Arc<dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send>> + Send + Sync>;

/* === Structs === */

#[derive(Clone, Default)]
pub struct Caller(Road);

// The host surface of one running stage: each call and submission is
// scoped to the execution and routed by the operation it names, exactly as
// `Context::operation` routes it.
#[derive(Clone)]
pub struct StepOperationHost {
    caller: Caller,
    stage: String,
    context: CallContext,
    binding: Option<Binding>,
    handles: super::instances::Handles,
}

/* === Enums === */

#[derive(Clone, Default)]
enum Road {
    #[default]
    Unmounted,

    Host(Supervisor),
    Local(Arc<BTreeMap<String, Answering>>),
}

/* === Traits === */

/* === Implementations === */

impl Caller {
    pub fn host(host: Supervisor) -> Self {
        Self(Road::Host(host))
    }

    pub fn local(operations: BTreeMap<String, Answering>) -> Self {
        Self(Road::Local(Arc::new(operations)))
    }

    pub(super) fn step_host(&self, stage: &str, binding: Option<Binding>) -> StepOperationHost {
        let handles = binding.as_ref().map_or_else(super::instances::Handles::new, |binding| binding.handles.clone());
        StepOperationHost {
            caller: self.clone(),
            stage: stage.to_owned(),
            context: CallContext::default(),
            binding,
            handles,
        }
    }

    fn hosted(&self) -> bool {
        matches!(self.0, Road::Host(_))
    }
}

impl OperationHost for StepOperationHost {
    async fn invoke<O: OperationDeclaration, E: harmos::operations::EndpointSet>(
        &self, request: O::Request, retry: Option<harmos::RetryPolicy>, _endpoints: E,
    ) -> crate::Result<Self::Instance<O>> {
        let answer: Result<Self::Instance<O>, CallError> = async {
            if E::item_types() != (None, None) {
                return Err(step_refusal("template calls do not accept native stream endpoints"));
            }
            if let Some(retry) = retry {
                retry.validate().map_err(|error| step_refusal(error.to_string()))?;
                if retry != harmos::RetryPolicy::Never && O::REPLAY == harmos::operations::ReplaySafety::Unknown {
                    return Err(step_refusal("call-site retries require declared replay safety"));
                }
            }
            let encoded = Message::encode(&request);
            if let Some(serve) = served(O::ID, self.caller.hosted()) {
                let binding = self.binding.clone().ok_or(CallError::Unavailable)?;
                let payload = encoded.clone();
                let run = LocalRun::spawn::<O, _>(
                    move |cancel| serve(binding.clone(), payload.clone(), cancel),
                    true,
                    retry,
                    self.binding.as_ref().map(|binding| binding.journal.clone()),
                    self.stage.clone(),
                )?;
                return Ok(StepInstance::local(&self.handles, run, &encoded));
            }
            match &self.caller.0 {
                Road::Unmounted => Err(CallError::Unavailable),
                Road::Host(host) => {
                    let routed_host = routed(host, &self.stage, O::ID).map_err(refusal)?;
                    let instance = routed_host.invoke::<O, E>(request, retry, _endpoints).await?;
                    Ok(StepInstance::host(&self.handles, instance, &encoded))
                }
                Road::Local(operations) => {
                    let answer = operations
                        .get(O::ID)
                        .cloned()
                        .ok_or_else(|| step_refusal(format!("the harness registered no answer for operation `{}`", O::ID)))?;
                    let payload = encoded.clone();
                    let run = LocalRun::spawn::<O, _>(
                        move |_| answer(payload.clone()),
                        false,
                        retry,
                        self.binding.as_ref().map(|binding| binding.journal.clone()),
                        self.stage.clone(),
                    )?;
                    Ok(StepInstance::local(&self.handles, run, &encoded))
                }
            }
        }
        .await;
        answer.map_err(crate::Error::from)
    }

    type Error = crate::Error;
    type Instance<O: OperationDeclaration> = StepInstance<O>;

    fn context(&self) -> &CallContext {
        match &self.caller.0 {
            Road::Host(host) => OperationHost::context(host),
            Road::Unmounted | Road::Local(_) => &self.context,
        }
    }

    fn with_value<T: ContextValue>(&self, value: T) -> Result<Self, ContextError> {
        let mut child = self.clone();
        match &mut child.caller.0 {
            Road::Host(host) => *host = OperationHost::with_value(host, value)?,
            Road::Unmounted | Road::Local(_) => child.context.insert_portable(value)?,
        }

        Ok(child)
    }

    async fn submit<O: OperationDeclaration>(&self, request: O::Request) -> crate::Result<Self::Instance<O>> {
        self.invoke::<O, ()>(request, None, ()).await
    }

    // A step lends no process-local resources to the host.
    async fn submit_owned<O: OperationDeclaration>(
        &self, _: O::Request, _: harmos::operations::RequestClaims,
    ) -> crate::Result<StepInstance<O>> {
        Err(CallError::LocalResources { method: O::ID }.into())
    }

    // A step publishes through the attempt's host; a harness without one has no publication route.
    async fn publish_on<S: StreamSpec>(&self, row: &S::Row) -> crate::Result<()>
    where
        S::Row: Message,
    {
        match &self.caller.0 {
            Road::Host(host) => host.publish_on::<S>(row).await.map_err(crate::Error::from),
            Road::Unmounted | Road::Local(_) => Err(CallError::Unavailable.into()),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

// Scope a child of the step's host to the execution stage and the operation it
// routes, so the host files its evidence and idempotency under the step.
fn routed(host: &Supervisor, stage: &str, operation: &str) -> Result<Supervisor, ContextError> {
    let Some(scope) = host.context().value::<super::ExecutionScope>()? else {
        return Ok(host.clone());
    };
    let scope = if stage.is_empty() { scope } else { scope.staged(stage) };

    host.with_value(scope.routed(operation))
}

fn refusal(error: ContextError) -> CallError {
    CallError::Refused {
        code: STEP_REFUSAL,
        failure: refusal_failure(error.to_string()),
    }
}

/* === Tests === */
