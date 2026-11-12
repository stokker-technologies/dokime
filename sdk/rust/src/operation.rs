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
use crate::{Context, Message, Result, context::Executions};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(crate) type FutureResult<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + 'a>>;
pub(crate) type Action<T> = Arc<dyn Handler<T>>;

/* === Structs === */

// An Ordinary Plugin Operation Composed by Value.
pub struct Operation {
    register: Box<dyn FnOnce(Executions) -> harmos::Operation + Send>,
    configure: Vec<Box<dyn FnOnce(harmos::Operation) -> harmos::Operation + Send>>,
}

struct Function<H>(H);

/* === Enums === */

/* === Traits === */

pub(crate) trait Handler<T>: Send + Sync {
    fn execute<'a>(&'a self, context: &'a Context) -> FutureResult<'a, T>;
}

/* === Implementations === */

impl Operation {
    pub fn new<Q, R, H>(identifier: impl Into<String>, handler: H) -> Self
    where
        Q: Message,
        R: Message,
        H: for<'a> AsyncFn(&'a Context, Q) -> Result<R> + Send + Sync + 'static,
    {
        let identifier = identifier.into();
        Self {
            register: Box::new(move |executions| {
                wire(identifier, async move |transport: &harmos::Context, request| {
                    let context = Context::invocation(transport, &executions)?;
                    handler(&context, request).await
                })
            }),
            configure: Vec::new(),
        }
    }
    pub fn visibility(self, visibility: harmos::Visibility) -> Self {
        self.configure(move |operation| operation.visibility(visibility))
    }
    pub fn policy(self, policy: harmos::OperationPolicy) -> Self {
        self.configure(move |operation| operation.policy(policy))
    }
    pub fn description(self, description: impl Into<String>) -> Self {
        let description = description.into();
        self.configure(move |operation| operation.description(description))
    }
    fn configure(mut self, configure: impl FnOnce(harmos::Operation) -> harmos::Operation + Send + 'static) -> Self {
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

pub(crate) fn wire<Q, R, H>(identifier: impl Into<String>, handler: H) -> harmos::Operation
where
    Q: Message,
    R: Message,
    H: for<'a> AsyncFn(&'a harmos::Context, Q) -> Result<R> + Send + Sync + 'static,
{
    harmos::Operation::new(identifier, async move |context: &harmos::Context, request| {
        handler(context, request).await.map_err(harmos::Error::from)
    })
}

/* === Private Functions === */

/* === Tests === */
