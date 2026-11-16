/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, atomic::AtomicU32},
    time::Duration,
};

// Third Party Imports
use harmos_protocol::model::Failure;
use harmos_runtime::clock::Controller;
use harmos_runtime::{operations::OperationHost, plugin::grpc::ScopedBinding};
use prost_reflect::ReflectMessage;
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::streams::{Attempts, ExecutionTime};
use crate::{Error, Result};
use dokime_operations::Resources;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Dispatch<'a> {
    pub time: Arc<ExecutionTime>,
    pub sample_failures: Arc<Mutex<HashMap<String, String>>>,
    pub binding: &'a ScopedBinding,
    pub resources: Resources,
    pub cancellation: CancellationToken,
    pub controller: Option<Arc<Controller>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Dispatch<'_> {
    pub async fn invoke<Q: ReflectMessage + Default, R: ReflectMessage + Default>(&self, operation: &str, request: Q) -> Result<R> {
        let mut operations = self.binding.operations().clone();
        let mut resources = self.resources.clone();
        resources.operation = operation.into();
        resources.attempt = Arc::new(AtomicU32::new(1));
        let attempts = Arc::new(Attempts::new(resources.clock.now_ns()));
        let mut context = OperationHost::context(&operations).clone();
        context.insert(self.time.clone()).map_err(|source| Error::Invalid(format!("Invoke {operation}: {source}")))?;
        context.insert(attempts).map_err(|source| Error::Invalid(format!("Invoke {operation}: {source}")))?;
        context.insert(resources).map_err(|source| Error::Invalid(format!("Invoke {operation}: {source}")))?;
        operations = operations.with_context(context);
        let action = operation;
        let mut invocation = operations.submit_encoded(action, request.encode_to_vec())?;
        let mut sleeping = self.time.sleeping.subscribe();
        let mut advanced = false;
        let result = loop {
            let advancing = !sleeping.borrow_and_update().is_empty();
            advanced |= advancing;
            tokio::select! {
                biased;
                _ = self.cancellation.cancelled() => {
                    invocation.request_cancel();
                    invocation.settle().await?;
                    break Err(Error::Cancelled);
                }
                result = &mut invocation => break result.map_err(|source| match source.failure() {
                    Some(failure) => Error::Plugin(Box::new(Failure::from(failure))),
                    None => Error::Library(source),
                }),
                _ = sleeping.changed() => {},
                _ = async {
                    match &self.controller { Some(controller) => controller.advance_next().await, None => std::future::pending().await }
                }, if advancing => {}
            }
        };
        if advanced && let Some(controller) = &self.controller {
            controller.advance_by(Duration::ZERO).await;
        }
        if let Some(source) = self
            .sample_failures
            .lock()
            .map_err(|source| Error::Invalid(format!("Read Sample Failure: {source}")))?
            .get(&self.resources.execution)
        {
            // Each Recorded Sample Failure Already Names Its Stream and the Action to Take.
            return Err(Error::Invalid(source.clone()));
        }
        let payload = result?;
        R::decode(payload.as_slice()).map_err(Into::into)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
