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

pub use crate::prompt::Prompt;
pub use crate::record::Record;
pub use crate::stream::{Observer, Stream};
pub use crate::suite::Suite;
pub use crate::template::{Criterion, Step, Template};
pub use ::harmos::plugin::{Collection, CollectionOutput, Publisher, Window};
pub use types::*;
include!("host_exports.rs");

/* === Imports === */

// Standard Library Imports
use std::{
    any::{Any, TypeId},
    collections::HashSet,
    env,
    marker::PhantomData,
    sync::Arc,
};

// Third Party Imports
use ::harmos::{Visibility, plugin};
use tokio::{runtime::Builder, signal};

// Local Imports
use crate::SecretDeclaration;
use crate::SecretsDeclarations;
use crate::SettingDeclaration;
use crate::SettingsDeclarations;
use crate::context::{self, Executions};
use crate::host::Request;
use crate::message::Wire;
use crate::operation::{self, Operation};
use crate::operations;
use crate::stream::{self, Sampler};
use crate::template::validate_names;
use crate::testing::Selected;
use crate::{Declarations, Error, Lane, Message, Result};
use crate::{PromptDefinitions, RecordContracts, Suites};
use crate::{authoring, client, configuration};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type DependencyRegistration = Box<dyn FnOnce(&mut plugin::Registrar<'_>) -> ::harmos::Result<()> + Send>;

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
    collections: bool,
    dependencies: Vec<DependencyRegistration>,
    suites: Vec<Suite>,
    settings: Vec<SettingDeclaration>,
    secrets: Vec<SecretDeclaration>,
    lanes: Vec<Lane>,
    registrations: Vec<stream::Registration>,
    errors: Vec<String>,
    initialize: stream::Registration,
    finalize: stream::Registration,
}

/// Product Registration Context for Generated Host Dependencies.
pub struct Registrar<'a> {
    dependencies: &'a mut Vec<DependencyRegistration>,
    requirements: &'a mut Vec<&'static str>,
}

pub struct Dependency<T>(PhantomData<fn() -> T>);

/* === Enums === */

/* === Traits === */

pub trait Registration {
    fn register(self, registrar: &mut Registrar<'_>) -> Result<()>;
}

/* === Implementations === */

impl Registrar<'_> {
    pub fn register<T: Registration>(&mut self, item: T) -> Result<()> {
        item.register(self)
    }
}

impl<T> Dependency<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for Dependency<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Request> Registration for Dependency<T> {
    fn register(self, registrar: &mut Registrar<'_>) -> Result<()> {
        if !registrar.requirements.contains(&T::ID) {
            registrar.requirements.push(T::ID);
            registrar.dependencies.push(Box::new(|registrar| registrar.register(plugin::Dependency::<Wire<T>>::new())));
        }
        Ok(())
    }
}

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
        let identifier = authoring::code_name::<T>(true);
        let version = version.into();
        Self {
            assembly: Assembly {
                identifier,
                label: authoring::type_label::<T>(),
                version,
                state: Arc::new(()),
                requirements: Vec::new(),
                samplers: Vec::new(),
                collections: false,
                dependencies: Vec::new(),
                suites: Vec::new(),
                settings: Vec::new(),
                secrets: Vec::new(),
                lanes: Vec::new(),
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
        H: for<'a> AsyncFn(&'a operations::Context) -> Result<()> + Send + Sync + 'static,
    {
        self.assembly.initialize = lifecycle("initialize", handler);
        self
    }
    pub fn finalize<H>(mut self, handler: H) -> Self
    where
        H: for<'a> AsyncFn(&'a operations::Context) -> Result<()> + Send + Sync + 'static,
    {
        self.assembly.finalize = lifecycle("finalize", handler);
        self
    }
}

impl Plugin {
    fn with_requirements(mut self) -> Self {
        if !self.assembly.requirements.is_empty() {
            return self;
        }
        let mut registrar = Registrar {
            dependencies: &mut self.assembly.dependencies,
            requirements: &mut self.assembly.requirements,
        };
        for dependency in HostOperation::ALL {
            if let Err(error) = registrar.register(dependency) {
                self.assembly.errors.push(error.to_string());
            }
        }
        self
    }

    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.assembly.identifier = identifier.into();
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
    pub fn lane(mut self, lane: Lane) -> Self {
        self.assembly.lanes.push(lane);
        self
    }
    pub fn operation(mut self, operation: Operation) -> Self {
        match operation.registration() {
            Ok(registration) => self.assembly.registrations.push(registration),
            Err(error) => self.assembly.errors.push(error.to_string()),
        }
        self
    }
    pub fn setting(mut self, setting: SettingDeclaration) -> Self {
        self.assembly.settings.push(setting);
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
    /// Feed Several Streams From One Measurement per Tick, Only While One of Them Has a Listener.
    pub fn collection(mut self, collection: Collection) -> Self {
        self.assembly.collections = true;
        self.assembly
            .registrations
            .push(Box::new(move |plugin, _, _| plugin.register(|registrar| registrar.register(collection)).map_err(Error::from)));
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
        let template = template.parts()?;
        let needs_collection = template
            .case
            .streams
            .iter()
            .any(|stream| !template.samplers.iter().chain(&self.assembly.samplers).any(|sampler| &sampler.stream == stream));
        if self.assembly.collections && needs_collection {
            return Err(Error::Invalid(
                "Test Collection Streams: Register the Plugin With a Product Host to Use Library-Owned Windows".into(),
            ));
        }
        Ok(Selected {
            plugin: self.assembly.identifier,
            template,
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
            version,
            label: _,
            state,
            requirements: _,
            samplers: _,
            collections: _,
            dependencies,
            suites,
            settings,
            secrets,
            lanes,
            registrations,
            errors,
            initialize,
            finalize,
        } = self.assembly;
        if !errors.is_empty() {
            return Err(Error::Invalid(errors.join("; ")));
        }
        let mut inner = plugin::Plugin::new(&identifier, version).register(|registrar| {
            for dependency in dependencies {
                dependency(registrar)?;
            }
            Ok(())
        })?;
        for lane in lanes {
            inner = inner.lane(lane);
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
            inner = inner.register(|registrar| registrar.register(plugin::Contribution(Wire(SecretsDeclarations { secrets }))))?;
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
            inner = inner.register(|registrar| registrar.register(plugin::Contribution(Wire(Suites { suites: definitions }))))?;
        }
        if !records.is_empty() {
            inner = inner.register(|registrar| registrar.register(plugin::Contribution(Wire(RecordContracts { records }))))?;
        }
        if !prompts.is_empty() {
            inner = inner.register(|registrar| registrar.register(plugin::Contribution(Wire(PromptDefinitions { prompts }))))?;
        }
        if !settings.is_empty() {
            inner = inner.register(|registrar| registrar.register(SettingsDeclarations { values: settings }))?;
        }
        for register in registrations {
            inner = register(inner, executions.clone(), "")?;
        }
        inner = initialize(inner, executions.clone(), &identifier)?;
        inner = finalize(inner, executions, &identifier)?;
        inner.declarations()?;
        Ok(inner)
    }
    pub fn declarations(self) -> Result<Declarations> {
        self.into_harmos()?.declarations().map_err(Error::from)
    }
    pub fn run(self) -> Result<()> {
        if env::var_os("HARMOS_HOST_ENDPOINT").is_some() || env::var_os("HARMOS_LAUNCH_TOKEN").is_some() {
            return self.into_harmos()?.run().map_err(Error::from);
        }
        let values = configuration::load()?;
        let configured = env::var_os("DOKIME_REMOTE")
            .map(|value| !value.is_empty())
            .unwrap_or_else(|| values.get("DOKIME_REMOTE").is_some_and(|value| !value.is_empty()));
        let address = "127.0.0.1:0";
        // The Listener Owns Its Runtime; This Thread Only Connects and Waits for Shutdown.
        let runtime = Builder::new_current_thread().enable_all().build()?;
        if !configured {
            return runtime.block_on(async move {
                let mut listener = self.into_harmos()?.remote().start(address).await?;
                signal::ctrl_c().await?;
                listener.close().await?;
                Ok(())
            });
        }
        let remote = configuration::resolve("DOKIME_REMOTE", None, &values)?;
        runtime.block_on(async move {
            let client = client::connect_with(client::ConnectionSettings {
                target: Some(remote),
                ..Default::default()
            })
            .await?;
            let options = client::PluginRegisterOptions {
                address: Some(address.into()),
                ..Default::default()
            };
            let result = async {
                client.plugins().register_with(self, options).await?;
                signal::ctrl_c().await?;
                Ok(())
            }
            .await;
            let closed = client.close().await;
            match (result, closed) {
                (Err(error), Err(cleanup)) => {
                    Err(Error::Invalid(format!("Run Plugin: Resolve the Run and Cleanup Failures: {error:?}; {cleanup:?}")))
                }
                (result, Ok(())) => result,
                (Ok(()), closed) => closed,
            }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn lifecycle<H>(name: &'static str, handler: H) -> stream::Registration
where
    H: for<'a> AsyncFn(&'a operations::Context) -> Result<()> + Send + Sync + 'static,
{
    Box::new(move |plugin, executions, identifier| {
        let operation =
            operation::wire(format!("{identifier}.{name}"), Visibility::Protected, async move |transport: &plugin::Context, (): ()| {
                let _cleanup = (name == "finalize").then(|| Finalize(executions.clone()));
                let context = operations::Context::invocation(transport, &executions)?;
                handler(&context).await
            });
        plugin.register(|registrar| registrar.register(operation)).map_err(Error::from)
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{HostOperation, Plugin, Visibility};
    use crate::operations::{Context, Operation};
    use crate::{Error, Lane, Result};
    use dokime_protocol::descriptor_pool;
    use harmos_protocol::{
        model::{Protobuf, lane_declaration},
        schema,
    };

    #[derive(crate::Row)]
    struct CollectionReading {
        value: f64,
    }

    #[derive(crate::Row)]
    struct CollectionLink {
        value: u64,
    }

    #[test]
    fn native_collections_register_grouped_members_without_product_producers() -> Result<()> {
        let first = ::harmos::plugin::Stream::<CollectionReading>::new("first");
        let second = ::harmos::plugin::Stream::<CollectionLink>::new("second");
        let collection = ::harmos::plugin::Collection::operation(async |_, _| Ok(()))
            .stream(first)
            .stream(second)
            .every(prost_types::Duration { seconds: 0, nanos: 25_000_000 });
        let declarations = Plugin::of::<()>("1.0.0").identifier("native").collection(collection).declarations()?;
        assert_eq!(declarations.streams.len(), 2);
        let group = &declarations.streams[0].collection;
        assert!(!group.is_empty());
        assert!(declarations.streams.iter().all(|member| &member.collection == group));
        Ok(())
    }

    #[test]
    fn lanes_preserve_declarations_and_operation_assignments() -> Result<()> {
        let instrument = Lane::serial("instrument");
        let operation = Operation::new("example.measure", Visibility::Public, async |_: &Context, value: f64| Ok(value)).lane(&instrument);
        let declarations = Plugin::of::<()>("1.0.0").lane(instrument).identifier("example").operation(operation).declarations()?;
        assert_eq!(declarations.lanes.len(), 1);
        assert_eq!(declarations.lanes[0].name, "instrument");
        assert_eq!(declarations.lanes[0].capacity, Some(lane_declaration::Capacity::Serial(true)));
        let operation = declarations.operations.iter().find(|operation| operation.id == "example.measure").unwrap();
        assert_eq!(operation.lane, "instrument");

        let missing = Lane::serial("missing");
        let operation = Operation::new("example.measure", Visibility::Public, async |_: &Context, value: f64| Ok(value)).lane(&missing);
        assert!(Plugin::of::<()>("1.0.0").operation(operation).declarations().is_err());
        Ok(())
    }

    #[test]
    fn lifecycle_hooks_are_inert_protected_operations_with_final_identity() -> Result<()> {
        let declarations = Plugin::of::<()>("1.0.0")
            .initialize(async |_| panic!("Inspecting Declarations Entered Initialize"))
            .finalize(async |_| panic!("Inspecting Declarations Entered Finalize"))
            .identifier("author")
            .declarations()?;
        for name in ["author.initialize", "author.finalize"] {
            let operation = declarations.operations.iter().find(|operation| operation.id == name).expect("Lifecycle Operation");
            assert_eq!(operation.visibility, Visibility::Protected as i32);
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
