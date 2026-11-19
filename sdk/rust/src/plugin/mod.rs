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

use crate::operation::Operation;
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
use std::{any::Any, collections::HashSet, env, sync::Arc};

// Third Party Imports

// Local Imports
use crate::SecretDeclaration;
use crate::SecretsDeclarations;
use crate::SettingDeclaration;
use crate::SettingsDeclarations;
use crate::context::{self, Executions};
use crate::data;
use crate::host::Request;
use crate::message::Wire;
use crate::stream::{Registration, Sampler};
use crate::template::validate_names;
use crate::testing::Selected;
use crate::{Declarations, Error, Message, OperationRequest, Result};
use crate::{PromptDefinitions, RecordContracts, Suites};

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
    version: String,
    label: String,
    state: Arc<dyn Any + Send + Sync>,
    requirements: Vec<&'static str>,
    samplers: Vec<Sampler>,
    inner: plugin::Plugin,
    suites: Vec<Suite>,
    settings: Vec<SettingDeclaration>,
    secrets: Vec<SecretDeclaration>,
    registrations: Vec<Registration>,
    errors: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Plugin {
    pub fn of<T>(version: impl Into<String>) -> Self {
        let identifier = crate::authoring::code_name::<T>(true);
        let version = version.into();
        Self {
            assembly: Assembly {
                identifier: identifier.clone(),
                label: crate::authoring::type_label::<T>(),
                version: version.clone(),
                state: Arc::new(()),
                requirements: Vec::new(),
                samplers: Vec::new(),
                inner: plugin::Plugin::new(identifier, version),
                suites: Vec::new(),
                settings: Vec::new(),
                secrets: Vec::new(),
                registrations: Vec::new(),
                errors: Vec::new(),
            },
        }
    }
}

impl Plugin {
    fn with_requirements(self) -> Self {
        if !self.assembly.requirements.is_empty() {
            return self;
        }
        HostOperation::ALL.into_iter().fold(self, |plugin, operation| plugin.requires_host(operation))
    }

    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.assembly.identifier = identifier.into();
        self.assembly.inner = plugin::Plugin::new(&self.assembly.identifier, &self.assembly.version);
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.assembly.label = label.into();
        self
    }
    pub fn state<T: Send + Sync + 'static>(mut self, state: T) -> Self {
        self.assembly.state = Arc::new(state);
        self
    }
    pub fn suite(mut self, suite: Suite) -> Self {
        self.assembly.suites.push(suite.bind());
        self
    }
    pub fn operation(mut self, operation: Operation) -> Self {
        match operation.registration() {
            Ok(registration) => self.assembly.registrations.push(registration),
            Err(error) => self.assembly.errors.push(error.to_string()),
        }
        self
    }
    pub(crate) fn requires_operation<Q: OperationRequest>(mut self) -> Self {
        if self.assembly.requirements.contains(&Q::ID) {
            return self;
        }
        self.assembly.requirements.push(Q::ID);
        self.assembly.inner = self.assembly.inner.requires::<Q>();
        self
    }
    pub(crate) fn requires<Q: Request>(self) -> Self {
        self.requires_operation::<Wire<Q>>()
    }
    pub fn setting(mut self, setting: SettingDeclaration) -> Self {
        self.assembly.settings.push(setting);
        self
    }
    pub fn settings<T: Message>(mut self, defaults: T) -> Self {
        match data::settings(defaults) {
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
    // Select One Template and What an In-Process Host Needs to Run It.
    pub(crate) fn into_testing(self, template: &str) -> Result<Selected> {
        let this = self.with_requirements();
        this.select_testing(template)
    }

    fn select_testing(self, template: &str) -> Result<Selected> {
        if !self.assembly.errors.is_empty() {
            return Err(Error::Invalid(self.assembly.errors.join("; ")));
        }
        let mut selected = Vec::new();
        for suite in self.assembly.suites {
            selected.extend(suite.templates.into_iter().filter(|candidate| candidate.id() == template));
        }
        if selected.len() != 1 {
            return Err(Error::Invalid(format!("Test Template: Declare Exactly One Template {template}")));
        }
        let template = selected.remove(0);
        let prompts = template.prompts.clone();
        Ok(Selected {
            plugin: self.assembly.identifier,
            template: template.parts()?,
            prompts,
            requirements: self.assembly.requirements,
            state: self.assembly.state,
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
            version: _,
            label: _,
            state,
            requirements: _,
            samplers: _,
            mut inner,
            suites,
            settings,
            secrets,
            registrations,
            errors,
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
        validate_names(suites.iter().flat_map(|suite| suite.templates.iter().map(|template| template.id())), "Template")?;
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
        let secret_names = secrets
            .iter()
            .filter_map(|secret| secret.metadata.as_ref().map(|metadata| metadata.identifier.clone()))
            .collect::<HashSet<_>>();
        if !secrets.is_empty() {
            inner = inner.section(Wire(SecretsDeclarations { secrets }));
        }
        let executions = Executions::new(state, secret_names);
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
        inner = inner.on_stop(move || context::clear(&executions).map_err(harmos::Error::from));
        inner.declarations()?;
        Ok(inner)
    }
    pub fn declarations(self) -> Result<Declarations> {
        self.into_harmos()?.declarations().map_err(Error::from)
    }
    pub fn run(self) -> Result<()> {
        // The Host Supplies Managed Listener Coordinates Through Its Private Launch Protocol.
        let managed = env::var_os("HARMOS_ADDRESS").is_some();
        let plugin = self.into_harmos()?;
        let plugin = if managed { plugin } else { plugin.remote() };
        plugin.serve().map_err(Error::from)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
