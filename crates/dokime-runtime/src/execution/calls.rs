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
use harmos::operations::{CallContext, ContextError, ContextValue, Host, OperationDeclaration, Response};
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
// `Context::call` routes it.
#[derive(Clone)]
pub struct StepHost {
    caller: Caller,
    stage: String,
    context: CallContext,
    binding: Binding,
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

    pub async fn call<O: OperationDeclaration<Request = O> + harmos::Portable>(&self, request: O) -> crate::Result<O::Response> {
        self.call_at("", request).await
    }

    pub(super) fn step_host(&self, stage: &str, binding: Binding) -> StepHost {
        StepHost {
            caller: self.clone(),
            stage: stage.to_owned(),
            context: CallContext::default(),
            binding,
        }
    }

    fn hosted(&self) -> bool {
        matches!(self.0, Road::Host(_))
    }

    pub(super) async fn call_at<O: OperationDeclaration<Request = O> + harmos::Portable>(
        &self, stage: &str, request: O,
    ) -> crate::Result<O::Response> {
        let operation = O::ID;
        let refused = |reason: String| crate::Error::Call { operation, reason };
        match &self.0 {
            Road::Unmounted => Err(refused("this template is running without a host to route through".into())),
            Road::Host(host) => routed(host, stage, operation)
                .map_err(|error| refused(error.to_string()))?
                .call::<O>(request)
                .await
                .map_err(|error| refused(error.to_string()))?
                .try_unwrap()
                .map_err(|_| refused("the sidecar response is shared".into())),
            Road::Local(operations) => {
                let answer =
                    operations.get(operation).ok_or_else(|| refused("the harness registered no answer for this operation".into()))?;
                let bytes = answer(<O::Request as Message>::encode(&request)).await.map_err(refused)?;
                <O::Response as Message>::decode(bytes.as_slice()).map_err(|error| refused(error.to_string()))
            }
        }
    }
}

impl Host for StepHost {
    type Error = CallError;
    type Instance<O: OperationDeclaration> = StepInstance<O>;

    fn context(&self) -> &CallContext {
        match &self.caller.0 {
            Road::Host(host) => Host::context(host),
            Road::Unmounted | Road::Local(_) => &self.context,
        }
    }

    fn with_value<T: ContextValue>(&self, value: T) -> Result<Self, ContextError> {
        let mut child = self.clone();
        match &mut child.caller.0 {
            Road::Host(host) => *host = Host::with_value(host, value)?,
            Road::Unmounted | Road::Local(_) => child.context.insert_portable(value)?,
        }

        Ok(child)
    }

    async fn submit<O: OperationDeclaration>(&self, request: O::Request) -> Result<Self::Instance<O>, CallError> {
        let encoded = Message::encode(&request);
        let handles = &self.binding.handles;
        if let Some(serve) = served(O::ID, self.caller.hosted()) {
            let binding = self.binding.clone();
            let payload = encoded.clone();
            let run = LocalRun::spawn(move |cancel, ready| serve(binding, payload, cancel, ready), true);
            return Ok(StepInstance::local(handles, run, &encoded));
        }
        match &self.caller.0 {
            Road::Unmounted => Err(CallError::Unavailable),
            Road::Host(host) => {
                let instance = routed(host, &self.stage, O::ID).map_err(refusal)?.submit::<O>(request).await?;
                let readiness_host = routed(host, &self.stage, O::ID).map_err(refusal)?;
                Ok(StepInstance::host(handles, instance, &encoded, readiness_host))
            }
            Road::Local(operations) => {
                let answer =
                    operations.get(O::ID).cloned().ok_or_else(|| step_refusal("the harness registered no answer for this operation"))?;
                let payload = encoded.clone();
                let run = LocalRun::spawn(move |_, _| answer(payload), false);
                Ok(StepInstance::local(handles, run, &encoded))
            }
        }
    }

    async fn call<O: OperationDeclaration>(&self, request: O::Request) -> Result<Response<O::Response>, CallError> {
        if served(O::ID, self.caller.hosted()).is_some() {
            return self.submit::<O>(request).await?.join().await;
        }
        match &self.caller.0 {
            Road::Unmounted => Err(CallError::Unavailable),
            Road::Host(host) => routed(host, &self.stage, O::ID).map_err(refusal)?.call::<O>(request).await,
            Road::Local(operations) => {
                let answer = operations.get(O::ID).ok_or_else(|| CallError::Refused {
                    code: STEP_REFUSAL,
                    failure: refusal_failure("the harness registered no answer for this operation"),
                })?;
                let bytes = answer(Message::encode(&request)).await.map_err(|message| CallError::Refused {
                    code: STEP_REFUSAL,
                    failure: refusal_failure(message),
                })?;
                let response = O::Response::decode(bytes.as_slice()).map_err(|error| CallError::Malformed {
                    method: O::ID,
                    message: error.to_string(),
                })?;

                Ok(Response::new(response))
            }
        }
    }

    // A step lends no process-local resources to the host.
    async fn submit_owned<O: OperationDeclaration>(
        &self, _: O::Request, _: harmos::operations::RequestClaims,
    ) -> Result<StepInstance<O>, CallError> {
        Err(CallError::LocalResources { method: O::ID })
    }

    // A step publishes through the attempt's host; a harness without one has no publication route.
    async fn publish_on<S: StreamSpec>(&self, row: &S::Row) -> Result<(), CallError>
    where
        S::Row: Message,
    {
        match &self.caller.0 {
            Road::Host(host) => host.publish_on::<S>(row).await,
            Road::Unmounted | Road::Local(_) => Err(CallError::Unavailable),
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
