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
use ::harmos::OperationPolicy;
use ::harmos::Visibility;
use ::harmos::plugin;

// Local Imports
use crate::operations;
use crate::templates::Context;
use crate::{Error, Lane, Message, Payload, Result, context::Executions, stream::Registration};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(crate) type FutureResult<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + 'a>>;
pub(crate) type Action<T> = Arc<dyn Handler<T>>;

/* === Structs === */

// An Ordinary Plugin Operation Composed by Value.
pub struct Operation {
    policy: Option<OperationPolicy>,
    register: Box<dyn FnOnce(Executions) -> plugin::Operation + Send>,
    configure: Vec<Box<dyn FnOnce(plugin::Operation) -> plugin::Operation + Send>>,
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
        Q: Payload,
        R: Payload,
        H: for<'a> AsyncFn(&'a operations::Context, Q) -> Result<R> + Send + Sync + 'static,
    {
        let identifier = identifier.into();
        Self {
            policy: None,
            register: Box::new(move |executions| {
                wire(identifier, visibility, async move |transport: &plugin::Context, request: Q::Message| {
                    let context = operations::Context::invocation(transport, &executions)?;
                    let response = handler(&context, Q::from_message(request)).await?;
                    Ok(response.into_message())
                })
            }),
            configure: Vec::new(),
        }
    }
    pub fn lane(self, lane: &Lane) -> Self {
        let lane = lane.clone();
        self.configure(move |operation| operation.lane(&lane))
    }
    pub fn replay_safe(self) -> Self {
        self.configure(|operation| operation.replay_safe())
    }
    pub fn policy(mut self, policy: OperationPolicy) -> Self {
        self.policy = Some(policy);
        self
    }
    pub fn description(self, description: impl Into<String>) -> Self {
        let description = description.into();
        self.configure(move |operation| operation.description(description))
    }
    fn configure(mut self, configure: impl FnOnce(plugin::Operation) -> plugin::Operation + Send + 'static) -> Self {
        self.configure.push(Box::new(configure));
        self
    }
    pub(crate) fn registration(self) -> Result<Registration> {
        if let Some(policy) = &self.policy {
            validate_policy(policy)?;
        }
        Ok(Box::new(move |plugin, executions, _| {
            let mut operation = (self.register)(executions);
            for configure in self.configure {
                operation = configure(operation);
            }
            if let Some(policy) = self.policy {
                operation = operation.policy(policy);
            }
            plugin.register(|registrar| registrar.register(operation)).map_err(Error::from)
        }))
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

pub(crate) fn wire<Q, R, H>(identifier: impl Into<String>, visibility: Visibility, handler: H) -> plugin::Operation
where
    Q: Message,
    R: Message,
    H: for<'a> AsyncFn(&'a plugin::Context, Q) -> Result<R> + Send + Sync + 'static,
{
    plugin::Operation::new(identifier, async move |context: &plugin::Context, request| {
        handler(context, request).await.map_err(::harmos::Error::from)
    })
    .visibility(visibility)
}

pub(crate) fn validate_policy(policy: &OperationPolicy) -> Result<()> {
    let Some(retries) = policy.retries.as_ref() else {
        return Ok(());
    };
    let positive = |duration: &prost_types::Duration| {
        duration.seconds >= 0 && (0..1_000_000_000).contains(&duration.nanos) && (duration.seconds > 0 || duration.nanos > 0)
    };
    if retries.total_attempts == 0
        || !retries.initial.as_ref().is_some_and(positive)
        || retries.maximum.as_ref().is_some_and(|maximum| !positive(maximum))
    {
        return Err(Error::Invalid(
            "Declare Retry Policy: Supply Positive Attempts and a Positive Initial Backoff, With a Positive Maximum When Present".into(),
        ));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::validate_policy;
    use crate::{OperationPolicy, RetryPolicy};
    use prost_types::Duration;

    #[test]
    fn retries_require_positive_backoff_before_host_admission() {
        assert!(validate_policy(&OperationPolicy::default()).is_ok());
        for initial in [None, Some(Duration::default()), Some(Duration { seconds: -1, nanos: 0 }), Some(Duration { seconds: 0, nanos: -1 })]
        {
            let policy = OperationPolicy {
                retries: Some(RetryPolicy {
                    total_attempts: 2,
                    initial,
                    ..Default::default()
                }),
                ..Default::default()
            };
            assert!(validate_policy(&policy).is_err());
        }
        let mut policy = OperationPolicy {
            retries: Some(RetryPolicy {
                total_attempts: 2,
                initial: Some(Duration { seconds: 0, nanos: 1 }),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(validate_policy(&policy).is_ok());
        policy.retries.as_mut().unwrap().maximum = Some(Duration::default());
        assert!(validate_policy(&policy).is_err());
    }
}
