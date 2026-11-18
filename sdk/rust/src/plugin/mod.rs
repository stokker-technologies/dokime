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

#[rustfmt::skip]
mod host;
#[rustfmt::skip]
mod types;

/* === Re-Exports === */

pub use crate::context::Context;
pub use crate::operation::Operation;
pub use crate::prompt::Prompt;
pub use crate::record::Record;
pub use crate::stream::{Observer, Stream};
pub use crate::suite::Suite;
pub use crate::template::{Criterion, Step, Template};
pub use harmos::plugin::Publisher;
pub use harmos::plugin::Window;
pub use types::*;
include!("host_exports.rs");

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// Third Party Imports

// Local Imports
use crate::SecretDeclaration;
use crate::SecretsDeclarations;
use crate::SettingDeclaration;
use crate::SettingsDeclarations;
use crate::{Error, Message, OperationRequest, Result, context::Executions, message::Wire, stream::Registration, template::validate_names};
use crate::{PromptDefinitions, RecordContracts, Suites};
use harmos;

use harmos::plugin;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A Product Plugin. Composition Is Inert Until Declaration Validation or Serve.
pub struct Plugin {
    assembly: Assembly,
}

struct Assembly {
    identifier: String,
    requirements: Vec<&'static str>,
    samplers: Vec<crate::stream::Sampler>,
    inner: plugin::Plugin,
    suites: Vec<Suite>,
    settings: Vec<SettingDeclaration>,
    secrets: Vec<SecretDeclaration>,
    registrations: Vec<Registration>,
    errors: Vec<String>,
    stop: Vec<Box<dyn Fn() -> Result<()> + Send + Sync>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Plugin {
    pub fn new(identifier: impl Into<String>, version: impl Into<String>) -> Self {
        let identifier = identifier.into();
        Self {
            assembly: Assembly {
                identifier: identifier.clone(),
                requirements: Vec::new(),
                samplers: Vec::new(),
                inner: plugin::Plugin::new(identifier, version),
                suites: Vec::new(),
                settings: Vec::new(),
                secrets: Vec::new(),
                registrations: Vec::new(),
                errors: Vec::new(),
                stop: Vec::new(),
            },
        }
    }
}

impl Plugin {
    fn with_requirements(self) -> Self {
        if !self.assembly.requirements.is_empty() {
            return self;
        }
        [
            HostOperation::BackendEnvironment,
            HostOperation::ContentArtifactsCreate,
            HostOperation::DocumentsAppendParts,
            HostOperation::EffectsCreate,
            HostOperation::EffectsGet,
            HostOperation::EffectsSet,
            HostOperation::EvidenceLogsAppend,
            HostOperation::EvidenceRecordsList,
            HostOperation::EvidenceRecordsAppend,
            HostOperation::EvidenceTraceabilityGet,
            HostOperation::EvidenceTraceabilitySet,
            HostOperation::EvidenceStatistics,
            HostOperation::ExecutionsProgressGet,
            HostOperation::ExecutionsProgressSet,
            HostOperation::FilesCapturesStart,
            HostOperation::FilesCapturesStop,
            HostOperation::FilesUploadsCreate,
            HostOperation::FilesUploadsGet,
            HostOperation::FilesUploadsResume,
            HostOperation::FilesUploadsAppend,
            HostOperation::FilesUploadsPublish,
            HostOperation::FilesUploadsDelete,
            HostOperation::FilesGet,
            HostOperation::FilesRead,
            HostOperation::FilesWatch,
            HostOperation::FilesDelete,
            HostOperation::FilesCapture,
            HostOperation::PromptsCreate,
            HostOperation::SecretsGet,
            HostOperation::InvocationsGet,
            HostOperation::InvocationsCancel,
            HostOperation::InvocationsWatch,
            HostOperation::OperationsList,
            HostOperation::OperationsGet,
            HostOperation::PluginsSettingsList,
            HostOperation::TimeGet,
            HostOperation::TimeSleep,
        ]
        .into_iter()
        .fold(self, |plugin, operation| plugin.requires_host(operation))
    }

    pub fn suite(mut self, suite: Suite) -> Self {
        self.assembly.suites.push(suite);
        self
    }
    pub fn operation(mut self, operation: Operation) -> Self {
        self.assembly.registrations.push(operation.registration());
        self
    }
    pub fn requires_operation<Q: OperationRequest>(mut self) -> Self {
        if self.assembly.requirements.contains(&Q::ID) {
            return self;
        }
        self.assembly.requirements.push(Q::ID);
        self.assembly.inner = self.assembly.inner.requires::<Q>();
        self
    }
    pub(crate) fn requires<Q: crate::host::Request>(self) -> Self {
        self.requires_operation::<Wire<Q>>()
    }
    pub fn setting(mut self, setting: SettingDeclaration) -> Self {
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
    pub fn secret(mut self, secret: SecretDeclaration) -> Self {
        self.assembly.secrets.push(secret);
        self
    }
    pub fn stream<T: Message>(mut self, stream: Stream<T>) -> Self {
        self.assembly.registrations.push(stream.registration());
        self
    }
    pub fn observer(mut self, observer: Observer) -> Self {
        self.assembly.samplers.push(observer.sampler.clone());
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

    // Select One Template and What an In-Process Host Needs to Run It.
    pub(crate) fn into_testing(self, template: &str) -> Result<crate::testing::Selected> {
        let this = self.with_requirements();
        this.select_testing(template)
    }

    fn select_testing(self, template: &str) -> Result<crate::testing::Selected> {
        if !self.assembly.errors.is_empty() {
            return Err(Error::Invalid(self.assembly.errors.join("; ")));
        }
        let mut selected = Vec::new();
        for suite in self.assembly.suites {
            selected.extend(suite.templates.into_iter().filter(|candidate| candidate.identifier() == template));
        }
        if selected.len() != 1 {
            return Err(Error::Invalid(format!("Test Template: Declare Exactly One Template {template}")));
        }
        let template = selected.remove(0);
        let prompts = template.prompts.clone();
        Ok(crate::testing::Selected {
            plugin: self.assembly.identifier,
            template: template.parts()?,
            prompts,
            requirements: self.assembly.requirements,
            samplers: self.assembly.samplers,
        })
    }

    // Finish Composition Without Opening a Socket or Running Author Callbacks.
    fn into_harmos(self) -> Result<plugin::Plugin> {
        self.with_requirements().assemble()
    }

    fn assemble(self) -> Result<plugin::Plugin> {
        let Assembly {
            identifier: _,
            requirements: _,
            samplers: _,
            mut inner,
            suites,
            settings,
            secrets,
            registrations,
            errors,
            stop,
        } = self.assembly;
        if !errors.is_empty() {
            return Err(Error::Invalid(errors.join("; ")));
        }
        let mut records = Vec::new();
        let mut prompts = Vec::new();
        for suite in &suites {
            for template in &suite.templates {
                records.extend(template.records.iter().cloned());
                prompts.extend(template.prompts.iter().cloned());
            }
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
        validate_names(settings.iter().map(|setting| setting.name.as_str()), "Setting")?;
        for prompt in &prompts {
            validate_names(
                prompt.fields.iter().map(|field| field.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str())),
                "Prompt Field",
            )?;
            validate_names(prompt.actions.iter().map(|action| action.identifier.as_str()), "Prompt Action")?;
        }
        validate_names(
            secrets.iter().map(|secret| secret.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str())),
            "Secret",
        )?;
        if !secrets.is_empty() {
            inner = inner.section(Wire(SecretsDeclarations { secrets }));
        }
        let executions: Executions = Arc::new(Mutex::new(HashMap::new()));
        let mut definitions = Vec::new();
        for mut suite in suites {
            for template in suite.templates {
                let (definition, registered) = template.register(inner, executions.clone())?;
                inner = registered;
                suite.definition.templates.push(definition);
            }
            definitions.push(suite.definition);
        }
        if !definitions.is_empty() {
            inner = inner.section(Wire(Suites { suites: definitions }));
        }
        if !records.is_empty() {
            inner = inner.section(Wire(RecordContracts { records }));
        }
        if !prompts.is_empty() {
            inner = inner.section(Wire(PromptDefinitions { prompts }));
        }
        if !settings.is_empty() {
            inner = inner.settings(SettingsDeclarations { values: settings });
        }
        for register in registrations {
            inner = register(inner, executions.clone(), "");
        }
        inner = inner.on_stop(move || crate::context::clear(&executions).map_err(harmos::Error::from));
        for cleanup in stop {
            inner = inner.on_stop(move || cleanup().map_err(harmos::Error::from));
        }
        inner.declarations()?;
        Ok(inner)
    }
    pub fn declarations(self) -> Result<crate::Declarations> {
        self.into_harmos()?.declarations().map_err(Error::from)
    }
    pub fn serve(self) -> Result<()> {
        self.into_harmos()?.serve().map_err(Error::from)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
