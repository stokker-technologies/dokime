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
}

struct Running {
    instance: harmos::plugin::PluginEndpoint,
    publisher: Option<Arc<Publisher>>,
    templates: Vec<Arc<dyn Runnable>>,
    provider_started: bool,
    operations: Option<harmos::operations::PluginRegistration>,
}

struct Served {
    template: Arc<dyn Runnable>,
    publisher: Arc<Publisher>,
    sessions: Arc<crate::domains::execution::run::Sessions>,
}

struct Publisher {
    owner: String,
    executions: native_telemetry::Executions,
    streams: BTreeMap<Identifier, Publication>,
    _registration: harmos::streams::StreamRegistration,
}

struct Publication {
    lease: Arc<harmos::streams::PublicationLease>,
    schema: Schema,
}

struct EmbeddedRun(Arc<Served>);

struct EmbeddedSignal(Arc<Served>);

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
    pub(super) fn prepare(provider: Arc<dyn EmbeddedProvider>) -> Result<Self> {
        let catalog = provider.catalog();
        Identifier::new(&catalog.source).map_err(operation)?;
        let streams = provider.streams();
        let generation = uuid::Uuid::now_v7().to_string();
        let wire_contracts = template_contracts(&catalog)?;
        let operations = wire_contracts.iter().map(|contract| contract.operation().clone()).collect();
        let definition = harmos::plugin::PluginDefinition {
            id: catalog.source.clone(),
            version: catalog.version.parse().map_err(operation)?,
            artifact: harmos::plugin::PluginArtifact::Embedded { revision: generation.clone() },
            requires: Vec::new(),
            configuration: b"null".to_vec(),
            contributions: harmos::plugin::PluginContributions {
                // EmbeddedProvider Declares No SDK Hooks Or Plugin Listeners.
                lifecycle: None,
                operations,
                wire_contracts,
                required_contracts: record_requirement(&catalog)?,
                observers: Vec::new(),
                listeners: Vec::new(),
                streams: streams.clone(),
                registrations: registrations(&catalog)?,
                sections: Vec::new(),
            },
        };
        native_telemetry::validate(&catalog, &streams).map_err(operation)?;

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
        self.running = Some(Running { instance, publisher: None, templates: Vec::new(), provider_started: false, operations: None });

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
        handle.capture_suite_objects(&self.owner).await?;

        let registration = handle.operations.stage_plugin(&scope, |bindings| {
            for template in &templates {
                let [run, signal] = crate::execution::contracts(template.template_identifier()).map_err(|_| {
                    harmos::Error::InvalidInput { field: "embedded template", reason: "the template id is not an operation identity" }
                })?;
                let served = Arc::new(Served {
                    template: Arc::clone(template),
                    publisher: Arc::clone(&publisher),
                    sessions: Arc::clone(&handle.execution_sessions),
                });
                bindings.register_wire_contract::<EmbeddedRun>(&run, crate::app::Lanes::Open, Arc::clone(&served))?;
                bindings.register_wire_contract::<EmbeddedSignal>(&signal, crate::app::Lanes::Open, served)?;
            }
            Ok(())
        })?;
        self.running.as_mut().expect("provider retains its engine").operations = Some(registration);
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
            if let Some(operations) = &mut running.operations {
                operations.shutdown().await?;
            }
            running.operations = None;
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

impl harmos::operations::Operation for EmbeddedRun {
    type Declaration = crate::execution::Run<()>;
    type Resources = Served;
    type Endpoints = ();
    type Error = harmos::sidecar::Refusal;
    type Context<'a> = harmos::operations::Context<'a>;

    async fn initialize<'ctx>(served: Arc<Served>, _: Self::Context<'ctx>) -> std::result::Result<Self, Self::Error> {
        Ok(Self(served))
    }

    async fn run<'ctx>(
        &mut self, _: Self::Context<'ctx>, request: &mut crate::execution::Run<()>, _: &mut (),
    ) -> std::result::Result<harmos::operations::Response<crate::transport::SidecarExecutionOutcome>, Self::Error> {
        let request = std::mem::take(&mut request.0);
        let host =
            Arc::new(crate::domains::execution::run::host::Embedded::new(Arc::clone(&self.0.sessions), request.execution_token.clone()));
        self.0
            .template
            .run(request, host, Some(SampleOutput::Embedded(self.0.publisher.clone())), crate::execution::Caller::default())
            .await
            .map(Into::into)
    }
}

impl harmos::operations::Operation for EmbeddedSignal {
    type Declaration = crate::execution::Signal<()>;
    type Resources = Served;
    type Endpoints = ();
    type Error = harmos::sidecar::Refusal;
    type Context<'a> = harmos::operations::Context<'a>;

    async fn initialize<'ctx>(served: Arc<Served>, _: Self::Context<'ctx>) -> std::result::Result<Self, Self::Error> {
        Ok(Self(served))
    }

    async fn run<'ctx>(
        &mut self, _: Self::Context<'ctx>, signal: &mut crate::execution::Signal<()>, _: &mut (),
    ) -> std::result::Result<harmos::operations::Response<crate::transport::SignalReply>, Self::Error> {
        self.0.template.signal(std::mem::take(&mut signal.0)).await.map(Into::into)
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

        Ok(Self { owner: scope.owner().into(), executions: handle.native_executions.clone(), streams, _registration: registration })
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
        let validator = harmos::schema::Validator::from_schema(&publication.schema, &harmos::config::SchemaSettings::default())
            .map_err(|error| error.to_string())?;
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

fn template_contracts(catalog: &InstalledPlugin) -> Result<Vec<harmos::operations::Contract>> {
    let mut contracts = Vec::new();
    for suite in &catalog.suites.suites {
        for template in &suite.suite.test_case_templates {
            contracts.extend(crate::execution::contracts(template.metadata.identifier.as_str()).map_err(operation)?);
        }
    }
    Ok(contracts)
}

// A provider that declares records may file them through the host's record
// publication, as a sidecar contributing records requires it.
fn record_requirement(catalog: &InstalledPlugin) -> Result<Option<Vec<harmos::plugin::inventory::contracts::Required>>> {
    if catalog.record_contracts.is_empty() {
        return Ok(None);
    }
    let contract = harmos::operations::WireDeclaration::of::<crate::records::RecordPublication>().contract().map_err(operation)?;
    Ok(Some(vec![harmos::plugin::inventory::contracts::Required { contract, grants: Vec::new() }]))
}

fn operation(error: impl std::fmt::Display) -> Error {
    Error::Operation(error.to_string())
}

/* === Tests === */
