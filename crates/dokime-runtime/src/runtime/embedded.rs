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
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

// Third Party Imports
use async_trait::async_trait;
use harmos::sidecar::StreamDeclaration;

// Local Imports
use super::{RuntimeHandle, native_telemetry};
use crate::execution::{EmbeddedPublisher, Runnable, SampleOutput};
use crate::state::ProviderCatalog;
use crate::transport::TransportFuture;
use crate::{InstalledPlugin, runtime::Error, runtime::Result};
use dokime_domain::model::identity::Identifier;

/* === Constants === */

// Finished embedded runs whose token still routes a late signal, such as failure cleanup.
const FINISHED_ROUTES: usize = 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Active {
    pub(super) provider: Arc<dyn EmbeddedProvider>,
    pub(super) catalog: ProviderCatalog,
    pub(super) definition: harmos::plugin::PluginDefinition,
    pub(super) streams: Vec<StreamDeclaration>,
    pub(super) exposed: bool,
    bindings: crate::plugins::dispatch::Bindings,
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
    operations: Option<harmos::operations::PluginRegistration>,
    observers: Option<tokio::task::JoinHandle<harmos::Result<Vec<harmos::operations::Running<harmos::operations::resident::Activation>>>>>,
}

#[derive(Clone)]
pub struct EmbeddedHost {
    pub caller: harmos::operations::Caller,
    pub configuration: crate::plugin::Configuration,
    scope: harmos::plugin::PluginScope,
    operations: crate::app::OperationManager,
    publisher: Arc<Publisher>,
}

#[derive(Clone)]
pub struct EmbeddedObserver {
    pub definition: harmos::observers::ObserverDefinition,
    pub description: String,
    pub run: Arc<dyn Fn(EmbeddedHost, harmos::observers::ObserverActivation) -> TransportFuture<'static, harmos::Result<()>> + Send + Sync>,
}

struct Observer {
    observer: EmbeddedObserver,
    host: EmbeddedHost,
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
    finished: VecDeque<String>,
}

#[derive(Clone)]
struct Served {
    owner: String,
    template: Arc<dyn Runnable>,
    publisher: Arc<Publisher>,
}

// Retires a finished run's token route, including on cancellation. The route
// outlives the run so the host's failure cleanup can still cancel the template
// that ran it; only the most recent finished routes are kept.
struct Admitted<'a> {
    templates: &'a Templates,
    token: String,
}

struct Publisher {
    owner: String,
    executions: native_telemetry::Executions,
    streams: BTreeMap<Identifier, Publication>,
    _registration: harmos::streams::StreamRegistration,
}

struct Publication {
    lease: Arc<harmos::streams::PublicationLease>,
    sequence: tokio::sync::Mutex<u64>,
}

/* === Enums === */

#[harmos::catalog]
enum PreparedOperations {
    Operation0(crate::execution::Run<()>),
    Operation1(crate::execution::Signal<()>),
}

/* === Traits === */

#[async_trait]
pub trait EmbeddedProvider: Send + Sync {
    fn catalog(&self) -> InstalledPlugin;
    fn streams(&self) -> Vec<StreamDeclaration>;

    fn operations(&self) -> Vec<harmos::operations::EncodedBinding> {
        Vec::new()
    }

    fn requirements(&self) -> Vec<harmos::plugin::inventory::contracts::Required> {
        Vec::new()
    }

    fn observers(&self) -> Vec<EmbeddedObserver> {
        Vec::new()
    }

    async fn initialize(&self, _host: EmbeddedHost) -> Result<()> {
        Ok(())
    }

    async fn start(&self) -> Result<Vec<Arc<dyn Runnable>>>;

    async fn stop(&self) -> Result<()>;
}

/* === Implementations === */

impl Active {
    pub(super) fn prepare(provider: Arc<dyn EmbeddedProvider>, settings: &harmos::config::SchemaSettings) -> Result<Self> {
        let catalog = provider.catalog();
        Identifier::new(&catalog.source).map_err(operation)?;
        let streams = provider.streams();
        catalog.settings.validate().map_err(operation)?;
        let operations = provider.operations();
        let definitions = operations.iter().map(|binding| binding.contract().operation().clone()).collect::<Vec<_>>();
        let bindings = crate::plugins::dispatch::Bindings::inspect(&catalog, &definitions).map_err(operation)?;
        let observers = provider.observers();
        let mut required = record_requirement(&catalog, settings)?.unwrap_or_default();
        required.extend(provider.requirements());
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
                operations: operations.iter().map(|binding| binding.contract().operation().clone()).collect(),
                wire_contracts: operations.iter().map(|binding| binding.contract().clone()).collect(),
                required_contracts: Some(required),
                observers: observers
                    .iter()
                    .map(|observer| harmos::plugin::inventory::observers::Declaration {
                        id: observer.definition.id.clone(),
                        description: observer.description.clone(),
                        activation: harmos::plugin::inventory::observers::Activation::Plugin,
                        streams: observer.definition.streams.iter().map(|stream| stream.id.clone()).collect(),
                        demand_keys: observer.definition.demand_keys.clone(),
                    })
                    .collect(),
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
            exposed: false,
            bindings,
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
        if !self.exposed {
            let id = Identifier::new(&self.definition.id).expect("embedded identity was validated");
            let dispatcher = Arc::new(self.bindings.clone().attach(&self.definition.id, handle.operations.clone()));
            handle.set_action_dispatcher(id.clone(), Arc::clone(&dispatcher) as _);
            let handle = handle.clone();
            tokio::spawn(async move {
                handle.discover_plugin_settings(&id, &dispatcher).await;
            });
            self.exposed = true;
        }
        handle.engage_owned_demand(&self.definition.id);
    }
}

impl Instance {
    pub(super) async fn start(
        &mut self, handle: &RuntimeHandle, router: &harmos::plugin::Router, authority: &harmos::plugin::PluginLifecycleAuthority<'_>,
    ) -> Result<()> {
        let instance = router.enter(&self.owner).map_err(operation)?;
        self.running = Some(Running {
            instance,
            publisher: None,
            templates: Vec::new(),
            provider_started: false,
            registered: false,
            operations: None,
            observers: None,
        });

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
        let operations = self.provider.operations();
        let registration = handle
            .operations
            .stage_plugin(&scope, |bindings| {
                for binding in operations {
                    bindings.encoded(binding, crate::app::Lanes::Open)?;
                }
                Ok(())
            })
            .map_err(operation)?;
        self.running.as_mut().expect("endpoint retained").operations = Some(registration);
        let catalog = self.provider.catalog();
        let configuration = handle.plugin_initialization(&catalog, &serde_json::json!({})).await?;
        let caller = handle.operations.caller(&self.running.as_ref().expect("endpoint retained").instance.caller()).map_err(operation)?;
        let host = EmbeddedHost {
            caller,
            configuration: configuration.clone(),
            scope: scope.clone(),
            operations: handle.operations.clone(),
            publisher: publisher.clone(),
        };
        let running = self.running.as_mut().expect("endpoint retained before admission");
        running.publisher = Some(publisher.clone());
        running.provider_started = true;

        self.provider.initialize(host.clone()).await?;
        handle.record_plugin_settings(&self.owner, configuration.settings, BTreeMap::new());
        let observers = self.provider.observers();
        if !observers.is_empty() {
            let ids: Vec<_> = observers.iter().map(|observer| observer.definition.id.clone()).collect();
            let resources = Arc::new(handle.operations.context());
            let backends = observers.into_iter().map(|observer| Observer { observer, host: host.clone() });
            handle.operations.stage_plugin_observers(&resources, &scope, backends).map_err(operation)?.commit().map_err(operation)?;
            let operations = handle.operations.clone();
            self.running.as_mut().expect("observer belongs to retained generation").observers = Some(tokio::spawn(async move {
                if scope.ready().await.is_err() {
                    return Ok(Vec::new());
                }
                let mut observers = Vec::new();
                for id in ids {
                    observers.push(operations.submit_resident_in(&id, resources.clone()).await?);
                }
                Ok(observers)
            }));
        }

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
        if let Some(running) = &mut self.running {
            if let Some(registration) = &mut running.operations {
                registration.shutdown().await.map_err(resident_error)?;
            }
            running.operations.take();
            if let Some(task) = running.observers.take() {
                if !task.is_finished() {
                    task.abort();
                }
                match task.await {
                    Ok(Ok(observers)) => {
                        for observer in observers {
                            observer.release().await.map_err(resident_error)?;
                        }
                    }
                    Ok(Err(error)) => return Err(resident_error(error)),
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => return Err(operation(error)),
                }
            }
        }
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
        Self {
            sessions,
            state: std::sync::Mutex::new(Registry::default()),
        }
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
            let served = Served {
                owner: owner.to_owned(),
                template: Arc::clone(template),
                publisher: Arc::clone(publisher),
            };
            registry.templates.insert(template.template_identifier().to_owned(), served);
        }
        Ok(())
    }

    fn remove(&self, owner: &str) {
        let mut registry = self.lock();
        let Registry { templates, executions, finished } = &mut *registry;
        templates.retain(|_, served| served.owner != owner);
        executions.retain(|_, template| templates.contains_key(template));
        finished.retain(|token| executions.contains_key(token));
    }

    // A template no started provider serves answers the not-registered code, distinct from a served template's refusal.
    fn template(&self, template: &str) -> std::result::Result<Served, harmos::sidecar::Refusal> {
        self.lock()
            .templates
            .get(template)
            .cloned()
            .ok_or_else(|| harmos::sidecar::Refusal::new(harmos::domain::UNKNOWN, "the embedded template is not served"))
    }

    fn admit(&self, token: &str, template: &str) -> Admitted<'_> {
        let mut registry = self.lock();
        registry.finished.retain(|finished| finished != token);
        registry.executions.insert(token.to_owned(), template.to_owned());
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
        let mut registry = self.templates.lock();
        if !registry.executions.contains_key(&self.token) {
            return;
        }
        registry.finished.push_back(self.token.clone());
        while registry.finished.len() > FINISHED_ROUTES {
            if let Some(token) = registry.finished.pop_front() {
                registry.executions.remove(&token);
            }
        }
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
            served
                .template
                .run(request, host, Some(SampleOutput::Embedded(served.publisher.clone())), crate::execution::Caller::default())
                .await
        })
    }

    fn signal(
        &self, signal: crate::transport::ExecutionSignal,
    ) -> TransportFuture<'_, std::result::Result<crate::transport::SignalReply, harmos::sidecar::Refusal>> {
        Box::pin(async move { self.signalled(&signal.execution_token)?.template.signal(signal).await })
    }
}

impl Publisher {
    async fn publish_rows(
        &self, token: Option<&str>, stream: &Identifier, rows: Vec<crate::execution::StampedRow>,
    ) -> std::result::Result<(), String> {
        if let Some(token) = token {
            self.executions.scope(&self.owner, token, stream)?;
        }
        let publication = self.streams.get(stream).ok_or("stream was not admitted for this provider")?;
        let rows = rows
            .into_iter()
            .map(|row| {
                Ok(harmos::streams::PublicationRow {
                    payload: row.storage_payload.ok_or("typed publication requires original row bytes")?,
                    published_unix_micros: Some(row.published_unix_micros),
                    context: token.map(str::to_owned),
                })
            })
            .collect::<std::result::Result<Vec<_>, String>>()?;
        let mut sequence = publication.sequence.lock().await;
        let first = sequence.checked_add(1).ok_or("publication sequence overflow")?;
        let batch = publication.lease.stage_rows(rows, first, 0);
        *sequence = sequence.checked_add(batch.staged.len() as u64).ok_or("publication sequence overflow")?;
        for staged in batch.staged {
            staged.committed().await.map_err(|error| error.to_string())?;
        }
        publication.lease.ingested().await.map_err(|error| error.to_string())?;
        batch.refused.map_or(Ok(()), |error| Err(error.to_string()))
    }
    async fn stage(handle: &RuntimeHandle, scope: &harmos::plugin::PluginScope, declarations: Vec<StreamDeclaration>) -> Result<Self> {
        let registration = handle.streams.stage_plugin(scope, declarations.clone()).await.map_err(operation)?;
        let streams = declarations
            .into_iter()
            .zip(registration.leases().iter().cloned())
            .map(|(declaration, lease)| {
                Ok((
                    Identifier::new(declaration.id).map_err(operation)?,
                    Publication {
                        lease,
                        sequence: tokio::sync::Mutex::new(0),
                    },
                ))
            })
            .collect::<Result<_>>()?;

        Ok(Self {
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
    fn values_required(&self) -> bool {
        false
    }

    fn publish<'a>(
        &'a self, token: &'a str, stream: &'a Identifier, rows: Vec<crate::execution::StampedRow>,
    ) -> TransportFuture<'a, std::result::Result<(), String>> {
        Box::pin(self.publish_rows(Some(token), stream, rows))
    }
}

impl EmbeddedHost {
    pub fn operation_definition(&self, id: &str, local: bool) -> harmos::Result<(String, harmos::OperationDefinition)> {
        let owner = if local { self.scope.owner().to_owned() } else { self.operations.owner(id)?.to_string() };
        let definition = self.operations.definition_from(&owner, id)?;
        Ok((format!("{owner}.{id}"), definition))
    }

    pub async fn publish(&self, execution: Option<&str>, stream: &Identifier, rows: Vec<crate::execution::StampedRow>) -> Result<()> {
        self.scope.ready().await.map_err(operation)?;
        self.publisher.publish_rows(execution, stream, rows).await.map_err(operation)
    }
}

impl harmos::observers::ObserverBackend for Observer {
    fn definition(&self) -> harmos::Result<harmos::observers::ObserverDefinition> {
        Ok(self.observer.definition.clone())
    }
    async fn run(
        &self, _resources: Arc<harmos::ExecutionContext>, activation: harmos::observers::ObserverActivation,
    ) -> harmos::Result<()> {
        self.host.scope.ready().await?;
        (self.observer.run)(self.host.clone(), activation).await
    }
}

/* === Public Functions === */

// Registers the host-level template run and signal every embedded provider serves through.
pub(crate) fn prepare(setup: &mut harmos::Setup<crate::app::Dokime>, templates: &Arc<Templates>) -> harmos::Result<()> {
    setup.resource::<crate::execution::EmbeddedTemplates>(std::sync::Arc::new(crate::execution::EmbeddedTemplates(
        Arc::clone(templates) as Arc<dyn crate::execution::EmbeddedRegistry>
    )));

    {
        setup.operations::<PreparedOperations>(crate::app::Lanes::Open);
        Ok(())
    }
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
    let appearance = AppearanceRegistration {
        customizations: catalog.app_customizations.clone(),
        themes: catalog.app_themes.clone(),
    };
    let mut values = vec![
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
    if let Some(capture) = &catalog.run_start {
        values.push(("dokime.run-start", serde_json::to_vec(capture)));
    }
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

fn resident_error(error: harmos::Error) -> Error {
    let mut message = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(error) = source {
        message.push_str(": ");
        message.push_str(&error.to_string());
        source = error.source();
    }
    Error::Operation(message)
}

/* === Tests === */
