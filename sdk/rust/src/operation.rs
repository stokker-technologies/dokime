/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{future::Future, pin::Pin, sync::Arc};

// Third Party Imports

// Local Imports
use crate::message::Wire;
use crate::plugin::AnalysesComputeRequest;
use crate::plugin::AnalysesComputeResponse;
use crate::{Context, Message, Result, context::Executions};
use harmos::Error as HarmosError;
use harmos::OperationPolicy;
use harmos::Visibility;
use harmos::plugin::Context as HarmosContext;
use harmos::plugin::Operation as HarmosOperation;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(crate) type FutureResult<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + 'a>>;
pub(crate) type Action<T> = Arc<dyn Handler<T>>;

/* === Structs === */

// An Ordinary Plugin Operation Composed by Value.
pub struct Operation {
    register: Box<dyn FnOnce(Executions) -> HarmosOperation + Send>,
    configure: Vec<Box<dyn FnOnce(HarmosOperation) -> HarmosOperation + Send>>,
}

struct Function<H>(H);

/* === Enums === */

/* === Traits === */

pub(crate) trait Handler<T>: Send + Sync {
    fn execute<'a>(&'a self, context: &'a Context) -> FutureResult<'a, T>;
}

/* === Implementations === */

impl Operation {
    pub fn new<Q, R, H>(identifier: impl Into<String>, visibility: Visibility, handler: H) -> Self
    where
        Q: Message,
        R: Message,
        H: for<'a> AsyncFn(&'a Context, Q) -> Result<R> + Send + Sync + 'static,
    {
        let identifier = identifier.into();
        Self {
            register: Box::new(move |executions| {
                wire(identifier, visibility, async move |transport: &HarmosContext, request| {
                    let context = Context::invocation(transport, &executions)?;
                    handler(&context, request).await
                })
            }),
            configure: Vec::new(),
        }
    }
    pub fn analysis<H>(identifier: impl Into<String>, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a Context, AnalysesComputeRequest) -> Result<AnalysesComputeResponse> + Send + Sync + 'static,
    {
        let identifier = identifier.into();
        Self {
            register: Box::new(move |executions| {
                wire(
                    identifier,
                    Visibility::Protected,
                    async move |transport: &HarmosContext, Wire(request): Wire<AnalysesComputeRequest>| {
                        let context = Context::invocation(transport, &executions)?;
                        Ok(Wire(handler(&context, request).await?))
                    },
                )
            }),
            configure: Vec::new(),
        }
    }
    pub fn replay_safe(self) -> Self {
        self.configure(|operation| operation.replay_safe())
    }
    pub fn policy(self, policy: OperationPolicy) -> Self {
        self.configure(move |operation| operation.policy(policy))
    }
    pub fn description(self, description: impl Into<String>) -> Self {
        let description = description.into();
        self.configure(move |operation| operation.description(description))
    }
    fn configure(mut self, configure: impl FnOnce(HarmosOperation) -> HarmosOperation + Send + 'static) -> Self {
        self.configure.push(Box::new(configure));
        self
    }
    pub(crate) fn registration(self) -> crate::stream::Registration {
        Box::new(move |plugin, executions| {
            let mut operation = (self.register)(executions);
            for configure in self.configure {
                operation = configure(operation);
            }
            plugin.operation(operation)
        })
    }
}

impl<T, H> Handler<T> for Function<H>
where
    H: for<'a> AsyncFn(&'a Context) -> Result<T> + Send + Sync + 'static,
{
    fn execute<'a>(&'a self, context: &'a Context) -> FutureResult<'a, T> {
        Box::pin((self.0)(context))
    }
}

/* === Public Functions === */

pub(crate) fn action<T, H>(handler: H) -> Action<T>
where
    H: for<'a> AsyncFn(&'a Context) -> Result<T> + Send + Sync + 'static,
{
    Arc::new(Function(handler))
}

pub(crate) fn wire<Q, R, H>(identifier: impl Into<String>, visibility: Visibility, handler: H) -> HarmosOperation
where
    Q: Message,
    R: Message,
    H: for<'a> AsyncFn(&'a HarmosContext, Q) -> Result<R> + Send + Sync + 'static,
{
    HarmosOperation::new(identifier, async move |context: &HarmosContext, request| {
        handler(context, request).await.map_err(HarmosError::from)
    })
    .visibility(visibility)
}

/* === Private Functions === */

/* === Tests === */
