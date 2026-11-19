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
#[allow(dead_code, reason = "The Shared Generator Emits the Complete Internal Host Transport")]
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
use std::{
    any::{Any, TypeId},
    collections::HashSet,
    env,
    sync::Arc,
};

// Third Party Imports
use harmos::plugin;

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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Finalize(Executions);

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
    initialize: Registration,
    finalize: Registration,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for Finalize {
    fn drop(&mut self) {
        context::clear(&self.0).expect("Finalize Plugin Resources");
    }
}

impl Plugin {
    pub(crate) fn registration_identifier(&self) -> &str {
        &self.assembly.identifier
    }

    pub(crate) fn template_bindings(&self) -> Vec<(TypeId, String, String)> {
        self.assembly
            .suites
            .iter()
            .flat_map(|suite| {
                let identifier = suite.definition.metadata.as_ref().expect("Suite Metadata").identifier.clone();
                suite.templates.iter().map(move |template| (template.marker, identifier.clone(), template.id().into()))
            })
            .collect()
    }

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
                initialize: lifecycle("initialize", async |_| Ok(())),
                finalize: lifecycle("finalize", async |_| Ok(())),
            },
        }
    }
}

impl Plugin {
    pub fn initialize<H>(mut self, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a crate::operations::Context) -> Result<()> + Send + Sync + 'static,
    {
        self.assembly.initialize = lifecycle("initialize", handler);
        self
    }
    pub fn finalize<H>(mut self, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a crate::operations::Context) -> Result<()> + Send + Sync + 'static,
    {
        self.assembly.finalize = lifecycle("finalize", handler);
        self
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
        if observer.execution {
            self.assembly
                .errors
                .push("Declare Plugin Observer: Use Observer::operation or Observer::operation_window With an Operation Context".into());
        }
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
    pub(crate) fn into_harmos(self) -> Result<plugin::Plugin> {
        self.with_requirements().assemble()
    }

    fn assemble(self) -> Result<plugin::Plugin> {
        let Assembly {
            identifier,
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
            initialize,
            finalize,
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
        inner = initialize(inner, executions.clone(), &identifier);
        inner = finalize(inner, executions, &identifier);
        inner.declarations()?;
        Ok(inner)
    }
    pub fn declarations(self) -> Result<Declarations> {
        self.into_harmos()?.declarations().map_err(Error::from)
    }
    pub fn run(self) -> Result<()> {
        // The Host Supplies Managed Listener Coordinates Through Its Private Launch Protocol.
        let managed = env::var_os("HARMOS_ADDRESS").is_some();
        if managed {
            return self.into_harmos()?.serve().map_err(Error::from);
        }
        let values = crate::configuration::load()?;
        let address = crate::configuration::resolve("DOKIME_PLUGIN_ADDRESS", None, &values)?;
        self.run_with(address)
    }

    pub fn run_with(self, address: impl Into<String>) -> Result<()> {
        // The Listener Owns Its Runtime; This Thread Only Waits for Shutdown.
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        runtime.block_on(async move {
            let mut listener = self.into_harmos()?.remote().start(address).await?;
            println!("harmos listening {}", listener.address());
            tokio::signal::ctrl_c().await?;
            listener.close().await?;
            Ok(())
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn lifecycle<H>(name: &'static str, handler: H) -> Registration
where
    H: for<'a> AsyncFn(&'a crate::operations::Context) -> Result<()> + Send + Sync + 'static,
{
    Box::new(move |plugin, executions, identifier| {
        let operation = crate::operation::wire(
            format!("{identifier}.{name}"),
            harmos::Visibility::Protected,
            async move |transport: &plugin::Context, (): ()| {
                let _cleanup = (name == "finalize").then(|| Finalize(executions.clone()));
                let context = crate::operations::Context::invocation(transport, &executions)?;
                handler(&context).await
            },
        );
        plugin.operation(operation)
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{HostOperation, Plugin};
    use crate::{Error, Result};
    use dokime_protocol::descriptor_pool;
    use harmos_protocol::{model::Protobuf, schema};

    #[test]
    fn lifecycle_hooks_are_inert_protected_operations_with_final_identity() -> Result<()> {
        let declarations = Plugin::of::<()>("1.0.0")
            .initialize(async |_| panic!("Inspecting Declarations Entered Initialize"))
            .finalize(async |_| panic!("Inspecting Declarations Entered Finalize"))
            .identifier("author")
            .declarations()?;
        for name in ["author.initialize", "author.finalize"] {
            let operation = declarations.operations.iter().find(|operation| operation.id == name).expect("Lifecycle Operation");
            assert_eq!(operation.visibility, harmos::Visibility::Protected as i32);
            assert_eq!(operation.request, "google.protobuf.Empty");
            assert_eq!(operation.response, "google.protobuf.Empty");
        }
        Ok(())
    }

    #[test]
    fn default_host_requirements_preserve_protocol_operation_kinds() -> Result<()> {
        // Compose the Complete Default Inventory Without Starting a Host.
        let declarations = Plugin::of::<()>("1.0.0").identifier("requirements").declarations()?;
        assert_eq!(declarations.requires.len(), HostOperation::ALL.len());

        // Fingerprint Each Requirement From the Authoritative Protocol Kind.
        let pool = descriptor_pool();
        for requirement in declarations.requires {
            let (service, name) = requirement.id.split_once('/').expect("Host Operation Identity");
            let service = pool.get_service_by_name(service).expect("Host Service Descriptor");
            let method = service.methods().find(|method| method.name() == name).expect("Host Method Descriptor");
            let contract = Protobuf {
                descriptors: pool.encode_to_vec(),
                request: method.input().full_name().into(),
                response: method.output().full_name().into(),
                input: method.is_client_streaming().then(|| method.input().full_name().into()),
                output: method.is_server_streaming().then(|| method.output().full_name().into()),
            };
            let fingerprint = schema::host_fingerprint(&requirement.id, &contract).map_err(|error| Error::Invalid(error.to_string()))?;
            assert_eq!(requirement.fingerprint, fingerprint, "{}", requirement.id);
        }
        Ok(())
    }
}
