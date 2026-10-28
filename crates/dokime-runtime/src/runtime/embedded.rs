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
use async_trait::async_trait;
use harmos::sidecar::StreamDeclaration;

// Local Imports
use super::{RuntimeHandle, native_telemetry};
use crate::execution::{EmbeddedPublisher, Runnable, SampleOutput};
use crate::transport::TransportFuture;
use crate::schema::graph::Schema;
use crate::state::ProviderCatalog;
use crate::{InstalledPlugin, runtime::Error, runtime::Result};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Active {
    pub(super) provider: Arc<dyn EmbeddedProvider>,
    pub(super) catalog: ProviderCatalog,
    pub(super) definition: harmos::plugin::PluginDefinition,
    pub(super) streams: Vec<StreamDeclaration>,
    observers: native_telemetry::Subscriptions,
}

pub(super) struct Instance {
    pub(super) publications: super::sidecars::PublicationIngest,
    provider: Arc<dyn EmbeddedProvider>,
    owner: String,
    streams: Vec<StreamDeclaration>,
    running: Option<Running>,
    templates: Option<Arc<Templates>>,
}

struct Running {
    instance: harmos::plugin::PluginEndpoint,
    publisher: Option<Arc<Publisher>>,
    templates: Vec<Arc<dyn Runnable>>,
    provider_started: bool,
    registered: bool,
}

// The host-level registry behind `dokime.template.run` and
// `dokime.template.signal`: every started embedded provider files its
// templates here, a run routes by its template identifier and a signal by the
// execution token its run admitted.
pub(crate) struct Templates {
    sessions: Arc<crate::domains::execution::run::Sessions>,
    state: std::sync::Mutex<Registry>,
}

#[derive(Default)]
struct Registry {
    templates: BTreeMap<String, Served>,
    executions: BTreeMap<String, String>,
}

#[derive(Clone)]
struct Served {
    owner: String,
    template: Arc<dyn Runnable>,
    publisher: Arc<Publisher>,
}

// Removes a finished run's token route, including on cancellation.
struct Admitted<'a> {
    templates: &'a Templates,
    token: String,
}

struct Publisher {
    schema_settings: harmos::config::SchemaSettings,
    owner: String,
    executions: native_telemetry::Executions,
    streams: BTreeMap<Identifier, Publication>,
    _registration: harmos::streams::StreamRegistration,
}

struct Publication {
    lease: Arc<harmos::streams::PublicationLease>,
    schema: Schema,
}

/* === Enums === */

/* === Traits === */

#[async_trait]
pub trait EmbeddedProvider: Send + Sync {
    fn catalog(&self) -> InstalledPlugin;
    fn streams(&self) -> Vec<StreamDeclaration>;

    async fn start(&self) -> Result<Vec<Arc<dyn Runnable>>>;

    async fn stop(&self) -> Result<()>;
}

/* === Implementations === */

impl Active {
    pub(super) fn prepare(provider: Arc<dyn EmbeddedProvider>, settings: &harmos::config::SchemaSettings) -> Result<Self> {
        let catalog = provider.catalog();
        Identifier::new(&catalog.source).map_err(operation)?;
        let streams = provider.streams();
        let generation = uuid::Uuid::now_v7().to_string();
        let definition = harmos::plugin::PluginDefinition {
            id: catalog.source.clone(),
            version: catalog.version.parse().map_err(operation)?,
            artifact: harmos::plugin::PluginArtifact::Embedded { revision: generation.clone() },
            requires: Vec::new(),
            configuration: b"null".to_vec(),
            contributions: harmos::plugin::PluginContributions {
                // EmbeddedProvider Declares No SDK Hooks Or Plugin Listeners.
                lifecycle: None,
                // Templates Run Through The Host-Level `dokime.template.*` Operations.
                operations: Vec::new(),
                wire_contracts: Vec::new(),
                required_contracts: record_requirement(&catalog, settings)?,
                observers: Vec::new(),
                listeners: Vec::new(),
                streams: streams.clone(),
                registrations: registrations(&catalog)?,
                sections: Vec::new(),
            },
        };
        native_telemetry::validate(&catalog, &streams, settings).map_err(operation)?;

        Ok(Self {
            provider,
            catalog: ProviderCatalog::Embedded { plugin: Box::new(catalog), generation },
            definition,
            streams,
            observers: Vec::new(),
        })
    }

    pub(super) fn instance(&self, publications: super::sidecars::PublicationIngest) -> Instance {
        Instance {
            publications,
            provider: Arc::clone(&self.provider),
            owner: self.definition.id.clone(),
            streams: self.streams.clone(),
            running: None,
            templates: None,
        }
    }

    pub(super) fn expose(&mut self, handle: &RuntimeHandle) {
        if self.observers.is_empty() {
            self.observers = native_telemetry::observe(handle, &self.definition.id);
        }
    }

    pub(super) fn release_observers(&mut self) {
        self.observers.clear();
    }

    pub(super) fn retain_observers(&mut self, previous: Self) {
        self.observers = previous.observers;
    }
}

impl Instance {
    pub(super) async fn start(
        &mut self, handle: &RuntimeHandle, router: &harmos::plugin::Router, authority: &harmos::plugin::PluginLifecycleAuthority<'_>,
    ) -> Result<()> {
        let instance = router.enter(&self.owner).map_err(operation)?;
        self.running = Some(Running { instance, publisher: None, templates: Vec::new(), provider_started: false, registered: false });

        let scope = self
            .running
            .as_ref()
            .expect("endpoint retained")
            .instance
            .scope()
            .cloned()
            .ok_or_else(|| operation("embedded endpoint has no managed activation scope"))?;
        if !authority.matches_scope(&scope, harmos::plugin::PluginKind::Embedded, harmos::plugin::PluginLifecyclePhase::Initialize) {
            return Err(operation("embedded initialization generation differs"));
        }
        let publisher = Arc::new(Publisher::stage(handle, &scope, self.streams.clone()).await?);
        let running = self.running.as_mut().expect("endpoint retained before admission");
        running.publisher = Some(publisher.clone());
        running.provider_started = true;

        let templates = self.provider.start().await?;
        let running = self.running.as_mut().expect("partial initialization remains owned");
        running.templates = templates.clone();

        handle.embedded_templates.insert(&self.owner, &templates, &publisher)?;
        self.running.as_mut().expect("provider retains its engine").registered = true;
        self.templates = Some(Arc::clone(&handle.embedded_templates));
        Ok(())
    }

    pub(super) async fn stop(&mut self, authority: &harmos::plugin::PluginLifecycleAuthority<'_>) -> Result<()> {
        if self.running.is_none() {
            return Ok(());
        }
        if let Some(running) = &mut self.running {
            let scope = running.instance.scope().ok_or_else(|| operation("embedded shutdown has no retained activation scope"))?;
            if !authority.matches_scope(scope, harmos::plugin::PluginKind::Embedded, harmos::plugin::PluginLifecyclePhase::Shutdown) {
                return Err(operation("embedded shutdown generation differs"));
            }
            if running.registered
                && let Some(templates) = &self.templates
            {
                templates.remove(&self.owner);
            }
            running.registered = false;
            running.instance.quiesce();
            for template in std::mem::take(&mut running.templates) {
                template.shutdown().await;
            }
        }

        // Keep both partial provider state and leases owned across cancellation.
        if self.running.as_ref().is_some_and(|running| running.provider_started) {
            self.provider.stop().await?;
            self.running.as_mut().expect("cleanup remains owned").provider_started = false;
        }
        if let Some(running) = self.running.take()
            && let Some(publisher) = running.publisher
        {
            publisher.revoke();
        }
        Ok(())
    }
}

impl Templates {
    pub(crate) fn new(sessions: Arc<crate::domains::execution::run::Sessions>) -> Self {
        Self { sessions, state: std::sync::Mutex::new(Registry::default()) }
    }

    pub(crate) fn contains(&self, template: &str) -> bool {
        self.lock().templates.contains_key(template)
    }

    // One provider owns each template identity; a second provider naming it is refused.
    fn insert(&self, owner: &str, templates: &[Arc<dyn Runnable>], publisher: &Arc<Publisher>) -> Result<()> {
        let mut registry = self.lock();
        if templates.iter().any(|template| registry.templates.contains_key(template.template_identifier())) {
            return Err(operation("an embedded template identity is already served"));
        }
        for template in templates {
            let served = Served { owner: owner.to_owned(), template: Arc::clone(template), publisher: Arc::clone(publisher) };
            registry.templates.insert(template.template_identifier().to_owned(), served);
        }
        Ok(())
    }

    fn remove(&self, owner: &str) {
        let mut registry = self.lock();
        let Registry { templates, executions } = &mut *registry;
        templates.retain(|_, served| served.owner != owner);
        executions.retain(|_, template| templates.contains_key(template));
    }

    fn template(&self, template: &str) -> std::result::Result<Served, harmos::sidecar::Refusal> {
        self.lock().templates.get(template).cloned().ok_or_else(|| harmos::sidecar::Refusal::of("the embedded template is not served"))
    }

    fn admit(&self, token: &str, template: &str) -> Admitted<'_> {
        self.lock().executions.insert(token.to_owned(), template.to_owned());
        Admitted { templates: self, token: token.to_owned() }
    }

    fn signalled(&self, token: &str) -> std::result::Result<Served, harmos::sidecar::Refusal> {
        let template = self.lock().executions.get(token).cloned();
        let template = template.ok_or_else(|| harmos::sidecar::Refusal::of("the execution is not running on an embedded template"))?;
        self.template(&template)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Registry> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Drop for Admitted<'_> {
    fn drop(&mut self) {
        self.templates.lock().executions.remove(&self.token);
    }
}

impl crate::execution::EmbeddedRegistry for Templates {
    fn run(
        &self, request: crate::transport::RunExecution,
    ) -> TransportFuture<'_, std::result::Result<crate::transport::SidecarExecutionOutcome, harmos::sidecar::Refusal>> {
        Box::pin(async move {
            let served = self.template(&request.template_identifier)?;
            let _admitted = self.admit(&request.execution_token, &request.template_identifier);
            let host =
                Arc::new(crate::domains::execution::run::host::Embedded::new(Arc::clone(&self.sessions), request.execution_token.clone()));
            served.template.run(request, host, Some(SampleOutput::Embedded(served.publisher.clone())), crate::execution::Caller::default()).await
        })
    }

    fn signal(
        &self, signal: crate::transport::ExecutionSignal,
    ) -> TransportFuture<'_, std::result::Result<crate::transport::SignalReply, harmos::sidecar::Refusal>> {
        Box::pin(async move { self.signalled(&signal.execution_token)?.template.signal(signal).await })
    }
}

impl Publisher {
    async fn stage(handle: &RuntimeHandle, scope: &harmos::plugin::PluginScope, declarations: Vec<StreamDeclaration>) -> Result<Self> {
        let registration = handle.streams.stage_plugin(scope, declarations.clone()).await.map_err(operation)?;
        let streams = declarations
            .into_iter()
            .zip(registration.leases().iter().cloned())
            .map(|(declaration, lease)| {
                Ok((Identifier::new(declaration.id).map_err(operation)?, Publication { lease, schema: declaration.row }))
            })
            .collect::<Result<_>>()?;

        Ok(Self {
            schema_settings: *handle.schema_settings(),
            owner: scope.owner().into(),
            executions: handle.native_executions.clone(),
            streams,
            _registration: registration,
        })
    }

    fn revoke(&self) {
        for publication in self.streams.values() {
            publication.lease.revoke();
        }
    }
}

impl EmbeddedPublisher for Publisher {
    fn publish(
        &self, token: &str, stream: &Identifier, _values: BTreeMap<String, Value>, payload: Vec<u8>,
    ) -> std::result::Result<(), String> {
        self.executions.scope(&self.owner, token, stream)?;
        let publication = self.streams.get(stream).ok_or("stream was not admitted for this provider")?;
        let validator =
            harmos::schema::Validator::from_schema(&publication.schema, &self.schema_settings).map_err(|error| error.to_string())?;
        validator.validate(&payload).map_err(|error| error.to_string())?;
        let published_unix_micros = i64::try_from(time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000)
            .map_err(|_| "publication time exceeds the protocol range")?;
        publication
            .lease
            .publish_row(
                harmos::streams::PublicationRow {
                    payload,
                    published_unix_micros: Some(published_unix_micros),
                    context: Some(token.into()),
                },
                0,
            )
            .map_err(|error| error.to_string())
    }
}

/* === Public Functions === */

// Registers the host-level template run and signal every embedded provider serves through.
pub(crate) async fn register(operations: &crate::app::OperationManager, templates: &Arc<Templates>) -> harmos::Result<()> {
    use harmos::RegistrationExt;
    let mut dependencies = harmos::Dependencies::new();
    dependencies.insert(Arc::new(crate::execution::EmbeddedTemplates(Arc::clone(templates) as Arc<dyn crate::execution::EmbeddedRegistry>)))?;
    let batch =
        harmos::RegistrationBatch::new(dependencies).operation::<crate::execution::Run<()>>().operation::<crate::execution::Signal<()>>();
    operations.register(batch.lane(crate::app::Lanes::Open)).await
}

/* === Private Functions === */

fn registrations(catalog: &InstalledPlugin) -> Result<Vec<harmos::sidecar::Registration>> {
    use crate::plugin::registration::{
        APPEARANCE_NAMESPACE, AppearanceRegistration, LISTENERS_NAMESPACE, OPTIONS_NAMESPACE, PERMISSIONS_NAMESPACE, PROMPTS_NAMESPACE,
        RECORDS_NAMESPACE, SETTINGS_NAMESPACE, SUITES_NAMESPACE, SettingsRegistration, TELEMETRY_NAMESPACE, domain_schema,
    };

    let settings = SettingsRegistration {
        status_entrypoint: catalog.settings_status.clone(),
        catalog: catalog.settings.clone(),
        discoveries: catalog.settings_discoveries.clone(),
    };
    let appearance = AppearanceRegistration { customizations: catalog.app_customizations.clone(), themes: catalog.app_themes.clone() };
    let values = [
        (SUITES_NAMESPACE, serde_json::to_vec(&catalog.suites)),
        (SETTINGS_NAMESPACE, serde_json::to_vec(&settings)),
        (APPEARANCE_NAMESPACE, serde_json::to_vec(&appearance)),
        (PERMISSIONS_NAMESPACE, serde_json::to_vec(&catalog.permissions)),
        (RECORDS_NAMESPACE, serde_json::to_vec(&catalog.record_contracts)),
        (LISTENERS_NAMESPACE, serde_json::to_vec(&catalog.listeners)),
        (PROMPTS_NAMESPACE, serde_json::to_vec(&catalog.prompts)),
        (TELEMETRY_NAMESPACE, serde_json::to_vec(&catalog.telemetry_contracts)),
        (OPTIONS_NAMESPACE, serde_json::to_vec(&catalog.option_providers)),
    ];
    values
        .into_iter()
        .map(|(namespace, payload)| {
            Ok(harmos::sidecar::Registration {
                namespace: namespace.into(),
                schema: domain_schema(namespace),
                payload: payload.map_err(operation)?,
            })
        })
        .collect()
}

// A provider that declares records may file them through the host's record
// publication, as a sidecar contributing records requires it.
fn record_requirement(
    catalog: &InstalledPlugin, settings: &harmos::config::SchemaSettings,
) -> Result<Option<Vec<harmos::plugin::inventory::contracts::Required>>> {
    if catalog.record_contracts.is_empty() {
        return Ok(None);
    }
    let contract = harmos::operations::WireDeclaration::of::<crate::records::RecordPublication>()
        .contract_with_settings(settings)
        .map_err(operation)?;
    Ok(Some(vec![harmos::plugin::inventory::contracts::Required { contract, grants: Vec::new() }]))
}

fn operation(error: impl std::fmt::Display) -> Error {
    Error::Operation(error.to_string())
}

/* === Tests === */
