/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

// Third Party Imports
use prost::{
    bytes::{Buf, BufMut},
    encoding::{DecodeContext, WireType},
};

// Local Imports
use crate::{
    Error, Message, Observer, Operation, OperationRequest, Prompt, Record, Result, Stream, Suite, context::Executions, host, message::Wire,
    stream::Registration, template::validate_names, v1,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A Product Plugin. Composition Is Inert Until Catalog Validation or Serve.
pub struct Plugin<C = ()> {
    assembly: Assembly,
    marker: PhantomData<fn() -> C>,
}

struct Assembly {
    inner: harmos::Plugin,
    identifier: String,
    suites: Vec<Suite>,
    records: Vec<v1::RecordContract>,
    prompts: Vec<v1::Prompt>,
    settings: Vec<v1::Setting>,
    registrations: Vec<Registration>,
    errors: Vec<String>,
    stop: Vec<Box<dyn Fn() -> Result<()> + Send + Sync>>,
}

struct Records<C> {
    message: Wire<v1::RecordContracts>,
    marker: PhantomData<fn() -> C>,
}

/* === Enums === */

/* === Traits === */

// Descriptor Accumulation for Typed Record Contracts; Normally Inferred by the Builder.
#[doc(hidden)]
pub trait CatalogTypes: Send + Sync + 'static {
    fn types() -> prost_types::FileDescriptorSet;
}

/* === Implementations === */

impl Plugin {
    pub fn new(identifier: impl Into<String>, version: impl Into<String>) -> Self {
        let identifier = identifier.into();
        Self {
            assembly: Assembly {
                inner: harmos::Plugin::new(identifier.clone(), version),
                identifier,
                suites: Vec::new(),
                records: Vec::new(),
                prompts: Vec::new(),
                settings: Vec::new(),
                registrations: Vec::new(),
                errors: Vec::new(),
                stop: Vec::new(),
            },
            marker: PhantomData,
        }
    }
}

impl<C: CatalogTypes> Plugin<C> {
    pub fn suite(mut self, suite: Suite) -> Self {
        self.assembly.suites.push(suite);
        self
    }
    pub fn operation(mut self, operation: Operation) -> Self {
        self.assembly.registrations.push(operation.registration());
        self
    }
    pub fn requires<Q: OperationRequest>(mut self) -> Self {
        self.assembly.inner = self.assembly.inner.requires::<Q>();
        self
    }
    pub fn requires_host<Q: host::Request>(self) -> Self {
        self.requires::<Wire<Q>>()
    }
    pub fn prompt(mut self, prompt: Prompt) -> Self {
        self.assembly.prompts.push(prompt.definition);
        self
    }
    pub fn setting(mut self, setting: v1::Setting) -> Self {
        self.assembly.settings.push(setting);
        self
    }
    pub fn settings<T: Message>(mut self, defaults: T) -> Self {
        match crate::data::settings(defaults) {
            Ok(settings) => self.assembly.settings.extend(settings),
            Err(error) => self.assembly.errors.push(error.to_string()),
        }
        self
    }
    pub fn record<T: Message>(mut self, record: Record<T>) -> Plugin<(C, T)> {
        self.assembly.records.push(record.definition);
        Plugin {
            assembly: self.assembly,
            marker: PhantomData,
        }
    }
    pub fn stream<T: Message>(mut self, stream: Stream<T>) -> Self {
        self.assembly.registrations.push(stream.registration());
        self
    }
    pub fn observer(mut self, observer: Observer) -> Self {
        self.assembly.registrations.push(observer.registration());
        self
    }
    pub fn on_stop(mut self, cleanup: impl Fn() -> Result<()> + Send + Sync + 'static) -> Self {
        self.assembly.stop.push(Box::new(cleanup));
        self
    }
    // Managed Is the Default; Both Placements Honor HARMOS_ADDRESS.
    pub fn remote(mut self) -> Self {
        self.assembly.inner = self.assembly.inner.remote();
        self
    }

    // Finish Composition Without Opening a Socket or Running Author Callbacks.
    pub fn into_harmos(self) -> Result<harmos::Plugin> {
        let Assembly {
            mut inner,
            identifier,
            suites,
            records,
            prompts,
            settings,
            registrations,
            errors,
            stop,
        } = self.assembly;
        if !errors.is_empty() {
            return Err(Error::Invalid(errors.join("; ")));
        }
        validate_names(
            suites.iter().filter_map(|suite| suite.definition.metadata.as_ref().map(|metadata| metadata.identifier.as_str())),
            "Suite",
        )?;
        validate_names(suites.iter().flat_map(|suite| suite.templates.iter().map(|template| template.identifier())), "Template")?;
        validate_names(
            records.iter().filter_map(|record| record.metadata.as_ref().map(|metadata| metadata.identifier.as_str())),
            "Record",
        )?;
        validate_names(
            prompts.iter().filter_map(|prompt| prompt.metadata.as_ref().map(|metadata| metadata.identifier.as_str())),
            "Prompt",
        )?;
        validate_names(
            settings.iter().map(|setting| setting.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str())),
            "Setting",
        )?;
        for prompt in &prompts {
            validate_names(
                prompt.fields.iter().map(|field| field.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str())),
                "Prompt Field",
            )?;
            validate_names(prompt.actions.iter().map(|action| action.identifier.as_str()), "Prompt Action")?;
        }
        for setting in &settings {
            if setting.kind.is_none() {
                return Err(Error::Invalid("Declare the Setting Kind Before Serving".into()));
            }
        }
        let executions: Executions = Arc::new(Mutex::new(HashMap::new()));
        let mut definitions = Vec::new();
        for mut suite in suites {
            for template in suite.templates {
                let (definition, registered) = template.register(inner, executions.clone(), &identifier)?;
                inner = registered;
                suite.definition.templates.push(definition);
            }
            definitions.push(suite.definition);
        }
        if !definitions.is_empty() {
            inner = inner.section(Wire(v1::Suites { suites: definitions }));
        }
        if !records.is_empty() {
            inner = inner.section(Records::<C> {
                message: Wire(v1::RecordContracts { records }),
                marker: PhantomData,
            });
        }
        if !prompts.is_empty() {
            inner = inner.section(Wire(v1::PromptCatalog { prompts }));
        }
        if !settings.is_empty() {
            inner = inner.section(Wire(v1::SettingsCatalog { settings }));
        }
        for register in registrations {
            inner = register(inner, executions.clone());
        }
        inner = inner.on_stop(move || crate::context::clear(&executions).map_err(harmos::Error::from));
        for cleanup in stop {
            inner = inner.on_stop(move || cleanup().map_err(harmos::Error::from));
        }
        inner.catalog()?;
        Ok(inner)
    }
    pub fn catalog(self) -> Result<harmos_protocol::v1::Catalog> {
        self.into_harmos()?.catalog().map_err(Error::from)
    }
    pub fn serve(self) -> Result<()> {
        self.into_harmos()?.serve().map_err(Error::from)
    }
}

impl CatalogTypes for () {
    fn types() -> prost_types::FileDescriptorSet {
        prost_types::FileDescriptorSet::default()
    }
}

impl<C: CatalogTypes, T: Message> CatalogTypes for (C, T) {
    fn types() -> prost_types::FileDescriptorSet {
        let mut types = C::types();
        harmos::encoding::merge_descriptor_types(&mut types, T::types());
        types
    }
}

impl<C> std::fmt::Debug for Records<C> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(formatter)
    }
}

impl<C: Send + Sync> prost::Message for Records<C> {
    fn encode_raw(&self, buffer: &mut impl BufMut) {
        self.message.encode_raw(buffer);
    }
    fn merge_field(
        &mut self, tag: u32, wire: WireType, buffer: &mut impl Buf, context: DecodeContext,
    ) -> std::result::Result<(), prost::DecodeError> {
        self.message.merge_field(tag, wire, buffer, context)
    }
    fn encoded_len(&self) -> usize {
        self.message.encoded_len()
    }
    fn clear(&mut self) {
        self.message.clear();
    }
}

impl<C: CatalogTypes> harmos::Message for Records<C> {
    fn name() -> String {
        <Wire<v1::RecordContracts> as harmos::Message>::name()
    }
    fn types() -> prost_types::FileDescriptorSet {
        let mut types = <Wire<v1::RecordContracts> as harmos::Message>::types();
        harmos::encoding::merge_descriptor_types(&mut types, C::types());
        types
    }
}

impl<C> Default for Records<C> {
    fn default() -> Self {
        Self {
            message: Wire::default(),
            marker: PhantomData,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
