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
use std::sync::Arc;

// Third Party Imports
// Third-Party Imports
use serde::de::DeserializeOwned;

// Local Imports
use super::context::{Cleanup, CriterionContext, Initialize, Step, Traceability, TraceabilityContext};
use super::document::DocumentIndex;
use super::evidence::ExecutionEvidence;
use super::{CleanupContext, Context, InitializationContext, StepContext};
use crate::artifacts::Artifacts;
use crate::records::Deliveries;
use crate::telemetry::Streams;
use crate::transport::ExecutionTransport;
use dokime_domain::model::execution::termination::TerminationReason;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Environment<'a, C = (), O = (), I = ()> {
    parameters: &'a I,
    raw: &'a BTreeMap<String, Value>,
    bindings: &'a crate::plugin::Configuration,
    calls: std::marker::PhantomData<fn() -> C>,

    #[cfg(feature = "native")]
    caller: super::Caller,

    #[cfg(feature = "native")]
    binding: Option<Arc<dyn std::any::Any + Send + Sync>>,

    own: Option<Arc<O>>,
    artifacts: Artifacts,
    streams: Streams,
    deliveries: Deliveries,
    document: Arc<std::sync::Mutex<DocumentIndex>>,

    #[cfg(feature = "native")]
    callbacks: super::Callbacks,

    #[cfg(feature = "native")]
    identity: Arc<super::ExecutionIdentity>,

    #[cfg(feature = "native")]
    handles: super::instances::Handles,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<'a, C, O, I> Environment<'a, C, O, I> {
    pub fn new(
        parameters: &'a I, raw: &'a BTreeMap<String, Value>, bindings: &'a crate::plugin::Configuration, artifacts: Artifacts,
        streams: Streams, deliveries: Deliveries,
    ) -> Self {
        Self {
            parameters,
            raw,
            bindings,
            calls: std::marker::PhantomData,
            #[cfg(feature = "native")]
            caller: super::Caller::default(),
            #[cfg(feature = "native")]
            binding: None,
            own: None,
            artifacts,
            streams,
            deliveries,
            document: Arc::new(std::sync::Mutex::new(DocumentIndex::new(None))),
            #[cfg(feature = "native")]
            callbacks: super::Callbacks::default(),
            #[cfg(feature = "native")]
            identity: Arc::default(),
            #[cfg(feature = "native")]
            handles: super::instances::Handles::new(),
        }
    }

    #[cfg(feature = "native")]
    #[must_use]
    pub fn identified(mut self, identity: super::ExecutionIdentity) -> Self {
        self.identity = Arc::new(identity);

        self
    }

    #[cfg(feature = "native")]
    pub(crate) fn identity(&self) -> Arc<super::ExecutionIdentity> {
        Arc::clone(&self.identity)
    }

    #[cfg(feature = "native")]
    pub(crate) fn handles(&self) -> &super::instances::Handles {
        &self.handles
    }

    #[cfg(feature = "native")]
    #[must_use]
    pub fn calling(mut self, caller: super::Caller) -> Self {
        self.caller = caller;

        self
    }

    #[cfg(feature = "native")]
    pub(crate) fn caller(&self) -> &super::Caller {
        &self.caller
    }

    #[cfg(feature = "native")]
    #[must_use]
    pub fn bound(mut self, binding: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        self.binding = Some(binding);

        self
    }

    #[cfg(feature = "native")]
    pub fn binding<B: 'static>(&self) -> &B {
        self.binding
            .as_deref()
            .and_then(|binding| binding.downcast_ref::<B>())
            .expect("a bound workflow is run against the binding it was built from")
    }

    #[cfg(feature = "native")]
    #[must_use]
    pub(crate) fn declaring(mut self, document: Arc<std::sync::Mutex<DocumentIndex>>) -> Self {
        self.document = document;

        self
    }

    pub(super) fn document(&self) -> &Arc<std::sync::Mutex<DocumentIndex>> {
        &self.document
    }

    #[cfg(feature = "native")]
    pub(super) fn with_callbacks(mut self, callbacks: super::Callbacks) -> Self {
        self.callbacks = callbacks;

        self
    }

    #[must_use]
    pub fn holding(mut self, own: Arc<O>) -> Self {
        self.own = Some(own);
        self
    }

    pub fn release(&mut self) {
        self.own = None;
    }

    pub fn initialize<'c>(&'c self, transport: &'c mut dyn ExecutionTransport) -> InitializationContext<'c, C, O, I> {
        Context::new(self, transport, Initialize).calling_stage("initialize")
    }

    pub fn traceability<'c>(&'c self, transport: &'c mut dyn ExecutionTransport) -> TraceabilityContext<'c, C, O, I> {
        Context::new(self, transport, Traceability).calling_stage("traceability")
    }

    pub fn step<'c>(&'c self, transport: &'c mut dyn ExecutionTransport) -> StepContext<'c, C, O, I> {
        Context::new(self, transport, Step)
    }

    #[cfg(feature = "native")]
    pub(super) fn observing<'c>(
        &'c self, transport: &'c mut dyn ExecutionTransport, tick: super::tick::Tick, publisher: Arc<super::observing::ExecutionPublisher>,
    ) -> super::ObserverContext<'c, C, O, I> {
        Context::new(self, transport, super::context::Observing { tick, publisher })
    }

    pub fn cleanup<'c>(&'c self, transport: &'c mut dyn ExecutionTransport, reason: TerminationReason) -> CleanupContext<'c, C, O, I> {
        Context::new(self, transport, Cleanup { reason }).calling_stage("cleanup")
    }

    pub fn criterion(&'a self, evidence: &'a ExecutionEvidence) -> CriterionContext<'a, C, I> {
        let context = CriterionContext::new(self.parameters, self.raw, evidence);
        #[cfg(feature = "native")]
        let context = context.calling(self.caller.clone()).bound(self.binding.clone()).with_callbacks(self.callbacks.clone());

        context
    }

    #[cfg(feature = "native")]
    pub fn callbacks(&self) -> &super::Callbacks {
        &self.callbacks
    }

    pub fn resources(&self) -> &O {
        self.own.as_deref().expect("the engine opens the attempt's resources before traceability and the first step")
    }

    pub fn artifacts(&self) -> &Artifacts {
        &self.artifacts
    }

    pub fn streams(&self) -> &Streams {
        &self.streams
    }

    pub fn deliveries(&self) -> &Deliveries {
        &self.deliveries
    }

    pub fn parameters(&self) -> &'a I {
        self.parameters
    }

    pub fn raw_parameters(&self) -> &'a BTreeMap<String, Value> {
        self.raw
    }

    pub fn parameter<T: DeserializeOwned>(&self, id: &str) -> crate::Result<T> {
        parameter(self.raw, id)
    }

    pub fn secret(&self, id: &str) -> crate::Result<crate::plugin::Secret> {
        self.bindings.secret(id)
    }
}

/* === Public Functions === */

pub(super) fn parameter<C: DeserializeOwned>(values: &BTreeMap<String, Value>, id: &str) -> crate::Result<C> {
    let failure = |reason: &str| crate::Error::Parameter {
        name: id.to_owned(),
        reason: reason.to_owned(),
    };
    let value = values.get(id).ok_or_else(|| failure("unknown parameter"))?;
    let json = value.to_plain_json().map_err(|_| failure("parameter cannot be represented as JSON"))?;
    serde_json::from_value(json).map_err(|_| failure("parameter has the wrong type"))
}

/* === Private Functions === */

/* === Tests === */
