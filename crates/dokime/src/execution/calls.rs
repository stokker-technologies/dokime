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
use harmos::Message;
use harmos::operations::{
    CallContext, Callable, ContextError, ContextValue, Declares, Host, OperationDeclaration,
    Response,
};
use harmos::sidecar::{CallError, Supervisor};

// Local Imports

/* === Constants === */
// The JSON-RPC server error code of a refusal raised on the step side of the call.
const STEP_REFUSAL: i64 = -32_000;

/* === Macros === */

/* === Type Aliases === */
pub type Answering = Arc<
    dyn Fn(Vec<u8>) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send>> + Send + Sync,
>;

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

    pub async fn call<O>(
        &self,
        request: <O::Declaration as OperationDeclaration>::Request,
    ) -> crate::Result<<O::Declaration as OperationDeclaration>::Response>
    where
        O: Declares,
        <O::Declaration as OperationDeclaration>::Request: Message,
        <O::Declaration as OperationDeclaration>::Response: Message,
    {
        self.call_at::<O>("", request).await
    }

    pub(super) fn step_host(&self, stage: &str) -> StepHost {
        StepHost {
            caller: self.clone(),
            stage: stage.to_owned(),
            context: CallContext::default(),
        }
    }

    pub(super) async fn call_at<O>(
        &self,
        stage: &str,
        request: <O::Declaration as OperationDeclaration>::Request,
    ) -> crate::Result<<O::Declaration as OperationDeclaration>::Response>
    where
        O: Declares,
        <O::Declaration as OperationDeclaration>::Request: Message,
        <O::Declaration as OperationDeclaration>::Response: Message,
    {
        let operation = <O::Declaration as OperationDeclaration>::ID;
        let refused = |reason: String| crate::Error::Call { operation, reason };
        match &self.0 {
            Road::Unmounted => Err(refused(
                "this template is running without a host to route through".into(),
            )),
            Road::Host(host) => routed(host, stage, operation)
                .map_err(|error| refused(error.to_string()))?
                .call_as::<O::Declaration>(request)
                .await
                .map_err(|error| refused(error.to_string()))?
                .try_unwrap()
                .map_err(|_| refused("the sidecar response is shared".into())),
            Road::Local(operations) => {
                let answer = operations.get(operation).ok_or_else(|| {
                    refused("the harness registered no answer for this operation".into())
                })?;
                let bytes = answer(request.encode_to_vec()).await.map_err(refused)?;
                <<O::Declaration as OperationDeclaration>::Response as Message>::decode(
                    bytes.as_slice(),
                )
                .map_err(|error| refused(error.to_string()))
            }
        }
    }
}

impl Host for StepHost {
    type Error = CallError;
    type Instance<O: Callable> = <Supervisor as Host>::Instance<O>;

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

    async fn submit_as<O: Callable>(
        &self,
        request: O::Request,
    ) -> Result<Self::Instance<O>, CallError> {
        let Road::Host(host) = &self.caller.0 else {
            return Err(CallError::Unavailable);
        };
        routed(host, &self.stage, O::ID)
            .map_err(refusal)?
            .submit_as::<O>(request)
            .await
    }

    async fn call_as<O: Callable>(
        &self,
        request: O::Request,
    ) -> Result<Response<O::Response>, CallError> {
        match &self.caller.0 {
            Road::Unmounted => Err(CallError::Unavailable),
            Road::Host(host) => {
                routed(host, &self.stage, O::ID)
                    .map_err(refusal)?
                    .call_as::<O>(request)
                    .await
            }
            Road::Local(operations) => {
                let answer = operations.get(O::ID).ok_or_else(|| CallError::Refused {
                    code: STEP_REFUSAL,
                    message: "the harness registered no answer for this operation".into(),
                })?;
                let bytes = answer(request.encode_to_vec()).await.map_err(|message| {
                    CallError::Refused {
                        code: STEP_REFUSAL,
                        message,
                    }
                })?;
                let response = O::Response::decode(bytes.as_slice()).map_err(|error| {
                    CallError::Malformed {
                        method: O::ID,
                        message: error.to_string(),
                    }
                })?;

                Ok(Response::new(response))
            }
        }
    }

    // A step lends no process-local resources to the host.
    async fn submit_with_resources<O: Callable<Response = ()>, R: Send + Sync + 'static>(
        &self,
        _: O::Request,
        _: Arc<R>,
    ) -> Result<Self::Instance<O>, CallError> {
        Err(CallError::LocalResources { method: O::ID })
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
    let scope = if stage.is_empty() {
        scope
    } else {
        scope.staged(stage)
    };

    host.with_value(scope.routed(operation))
}

fn refusal(error: ContextError) -> CallError {
    CallError::Refused {
        code: STEP_REFUSAL,
        message: error.to_string(),
    }
}

/* === Tests === */
