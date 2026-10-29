/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod preparation;
mod records;
mod settings;

/* === Re-Exports === */

pub(crate) use preparation::Prepared;

pub use records::{CapturedRecord, CapturedRecords, RecordSubscription};

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, RwLock},
};

// Third Party Imports

// Local Imports
use crate::InstalledPlugin;
use crate::app::{Dokime, HarmosRuntime};
use crate::domains::ActionDispatcher;
use crate::runtime::state::RuntimeState;
use crate::transactions::Transactions;
use crate::{
    config::RuntimeConfig,
    execution_runtime::provider::{ExecutionFixture, ProviderExecutorRegistry},
    installed_plugins::module_preferences::PluginPreferences,
    runtime::error::{Error, Result},
    secrets::SecretStore,
    telemetry_runtime::Sampling,
};
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::SessionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

#[cfg(feature = "fault-injection")]
pub(crate) type BackendDecorator = Box<dyn FnOnce(harmos::storage::StorageBackends) -> harmos::storage::StorageBackends + Send>;
pub(crate) type InstalledPlugins = BTreeMap<String, InstalledPlugin>;

/* === Structs === */

// The Opened Owners A Boot Hands To Handle Assembly.
pub(crate) struct BootParts {
    pub(crate) config: RuntimeConfig,
    pub(crate) record_namespace: SessionId,
    installed_plugins: InstalledPlugins,
    pub(crate) storage: crate::storage::Manager,
    pub(crate) local_state: crate::storage::sqlite::State,
    pub(crate) record_source: Arc<crate::runtime::storage::record_capture::Source>,
    pub(crate) resources: crate::app::Resources,
    node_identity: dokime_domain::model::identity::DokimeNodeIdentity,
    application_settings: crate::config::Settings,
    plugin_preferences: PluginPreferences,
    pub(crate) secrets: SecretStore,
    sampling: Sampling,
    providers: ProviderExecutorRegistry,
    boot_started: std::time::Instant,
}

#[derive(Clone)]
pub struct RuntimeHandle {
    storage: crate::storage::Manager,
    pub(crate) local_storage: crate::storage::sqlite::State,
    pub(crate) request_storage: Arc<crate::runtime::storage::requests::RequestStorage>,
    pub(crate) execution_storage: Arc<crate::runtime::storage::executions::ExecutionStorage>,
    pub(crate) plugin_boot: Arc<RwLock<BTreeMap<uuid::Uuid, bool>>>,
    pub(crate) state_manager: harmos::state::StateManager<Dokime>,
    pub(crate) record_source: Arc<crate::runtime::storage::record_capture::Source>,
    pub(crate) streams: crate::app::Streams,
    pub(crate) operations: crate::app::OperationManager,
    grpc: harmos::grpc::GrpcManager,
    pub(crate) record_ingress: Arc<super::sidecars::core::RecordIngress>,
    pub(crate) execution_sessions: Arc<crate::domains::execution::run::Sessions>,
    pub(crate) embedded_templates: Arc<crate::runtime::EmbeddedTemplates>,

    pub(crate) sampling: Arc<Sampling>,
    pub(crate) stream_leases: Arc<crate::telemetry_runtime::StreamLeases>,
    pub(crate) evidence_admission: Arc<tokio::sync::Mutex<()>>,
    pub(crate) evidence_exports: crate::domains::exports::evidence::Manager,
    pub(crate) telemetry_admission: Arc<tokio::sync::Mutex<()>>,
    pub(crate) telemetry_cursors: Arc<Mutex<crate::domains::telemetry::TelemetryCursors>>,
    pub(crate) providers: Arc<ProviderExecutorRegistry>,
    pub(crate) progress: Arc<crate::domains::execution::progress::ProgressRegistry>,
    pub(crate) operation_progress: Arc<crate::runtime::progress::OperationRegistry>,
    pub(crate) native_executions: super::native_telemetry::Executions,
    pub(crate) record_demands: Arc<Mutex<super::sidecars::RecordDemands>>,
    pub(crate) publication_ingest: Arc<std::sync::OnceLock<super::sidecars::PublicationIngest>>,
    pub(crate) shutdown_requested: tokio::sync::watch::Sender<bool>,
    pub(crate) provider_tasks: Arc<Mutex<Vec<crate::runtime::operations::provider::ProviderTask>>>,
    pub(crate) provider_invocations: Arc<crate::runtime::operations::provider::ProviderInvocations>,
    pub(crate) provider_completion_failure: Arc<Mutex<Option<crate::runtime::operations::provider::PublicationFailure>>>,
    // Detached work shutdown drains before the runtime closes.
    pub(crate) owned_work: tokio_util::task::TaskTracker,
    pub(crate) plugin_admission: Arc<tokio::sync::Mutex<()>>,
    pub(crate) unavailable_plugins: Arc<RwLock<std::collections::BTreeSet<String>>>,
    pub(crate) state: Arc<RwLock<Arc<RuntimeState>>>,
    pub(crate) state_watch: tokio::sync::watch::Sender<crate::queries::RuntimeStateSnapshot>,
    pub(crate) action_dispatchers: Arc<RwLock<BTreeMap<Identifier, Arc<dyn ActionDispatcher>>>>,
    pub(crate) routing: Arc<std::sync::OnceLock<harmos::plugin::Router>>,
    config: Arc<RuntimeConfig>,
    settings: crate::config::Settings,
    plugin_preferences: Arc<PluginPreferences>,
    secrets: Arc<SecretStore>,
    pub(crate) brand: Arc<RwLock<Option<crate::installed_plugins::brand::ResolvedBrand>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub(crate) fn admit_owned_work(&self) -> Option<tokio_util::task::task_tracker::TaskTrackerToken> {
        // Take The Token First So A Concurrent Close Either Waits For It Or Is Seen.
        let token = self.owned_work.token();
        (!self.owned_work.is_closed()).then_some(token)
    }

    pub fn storage(&self) -> &crate::storage::Manager {
        &self.storage
    }

    pub fn streams(&self) -> &harmos::streams::StreamManager<Dokime> {
        &self.streams
    }

    pub fn operation_progress(&self) -> &Arc<crate::runtime::progress::OperationRegistry> {
        &self.operation_progress
    }

    pub fn watch_execution_progress(&self) -> crate::domains::execution::progress::ProgressWatch {
        self.progress.watch()
    }

    pub(crate) async fn commit<T>(&self, transaction: T) -> std::result::Result<harmos::state::Receipt, harmos::Error<T::Error>>
    where
        T: harmos::state::Transaction<RuntimeState> + Into<Transactions>,
        T::Error: From<String>,
    {
        let receipt = self.state_manager.commit(transaction).await?;
        self.refresh_state().await.map_err(|_| harmos::Error::Unavailable)?;
        Ok(receipt)
    }

    pub(crate) async fn commit_keyed<T>(
        &self, key: harmos::state::RequestKey, transaction: T,
    ) -> std::result::Result<harmos::state::Receipt, harmos::Error<T::Error>>
    where
        T: harmos::state::Transaction<RuntimeState> + Into<Transactions>,
        T::Error: From<String>,
    {
        let receipt = self.state_manager.commit_keyed(key, transaction).await?;
        self.refresh_state().await.map_err(|_| harmos::Error::Unavailable)?;
        Ok(receipt)
    }

    pub(crate) async fn commit_owned(
        &self, transaction: harmos::operations::Response<Transactions>,
    ) -> std::result::Result<harmos::state::Receipt, harmos::Error<String>> {
        let receipt = self.state_manager.commit_owned(transaction).await?;
        self.refresh_state().await.map_err(|_| harmos::Error::Unavailable)?;
        Ok(receipt)
    }

    #[tracing::instrument(
        level = "info",
        name = "runtime.boot",
        skip_all,
        fields(session_id = %config.session_id),
    )]
    pub(crate) async fn boot(
        config: RuntimeConfig, providers: BTreeMap<String, Arc<dyn ExecutionFixture>>, install_builtins: bool,
    ) -> Result<(HarmosRuntime, Self)> {
        #[cfg(feature = "fault-injection")]
        {
            Box::pin(Self::boot_inner(config, providers, install_builtins, None)).await
        }
        #[cfg(not(feature = "fault-injection"))]
        {
            Box::pin(Self::boot_inner(config, providers, install_builtins)).await
        }
    }

    #[cfg(feature = "fault-injection")]
    pub(crate) async fn boot_with_backends(
        config: RuntimeConfig, providers: BTreeMap<String, Arc<dyn ExecutionFixture>>, install_builtins: bool, decorate: BackendDecorator,
    ) -> Result<(HarmosRuntime, Self)> {
        Box::pin(Self::boot_inner(config, providers, install_builtins, Some(decorate))).await
    }

    async fn boot_inner(
        config: RuntimeConfig, providers: BTreeMap<String, Arc<dyn ExecutionFixture>>, install_builtins: bool,
        #[cfg(feature = "fault-injection")] decorate: Option<BackendDecorator>,
    ) -> Result<(HarmosRuntime, Self)> {
        let mut state = RuntimeState::new(config.session_id.clone());
        state.record_namespace = crate::model::identity::new_id::<dokime_domain::model::identity::SessionId>();
        if install_builtins {
            state.install_builtin_domains().map_err(|error| Error::Operation(format!("failed to install built-in domains: {error}")))?;
        }

        let live = crate::app::LiveRuntime::default();
        let boot = Arc::new(std::sync::Mutex::new(super::rollback::Boot::default()));
        let receipt = super::rollback::Receipt::boot(boot.clone());
        let mut preparing = super::rollback::Preparing(Some(receipt.clone()));
        let launch = crate::app::LaunchConfig {
            record_namespace: state.process_epoch().clone(),
            config,
            providers,
            live: live.clone(),
            boot,
            #[cfg(feature = "fault-injection")]
            decorate,
        };
        let outcome = HarmosRuntime::launch(state, Some(launch)).await;
        let harmos = match outcome {
            Ok(runtime) => {
                preparing.0.take();
                runtime
            }
            Err(source) => {
                let source = Error::RuntimeBoot { source: Box::new(source) };
                let cleanup = receipt.retry().await;
                preparing.0.take();
                return match cleanup {
                    Ok(()) => Err(source),
                    Err(cleanup) => Err(Error::StartupCleanup {
                        source: Box::new(source),
                        cleanup: Box::new(cleanup),
                        rollback: receipt,
                    }),
                };
            }
        };
        let handle = live
            .try_read()
            .expect("application initialization releases its runtime slot")
            .clone()
            .expect("application initialization publishes its runtime handle");
        Ok((harmos, handle))
    }

    pub(crate) async fn prepare(launch: crate::app::LaunchConfig) -> Result<BootParts> {
        let config = launch.config;
        let providers = launch.providers;
        #[cfg(feature = "fault-injection")]
        let decorate = launch.decorate;
        let boot_started = std::time::Instant::now();
        let installed_plugins = check_boot_config(&config)?;
        config.storage.validate()?;
        harmos::filesystem::publication::create_dir_all(&config.storage_root)?;
        let backend = Arc::new(crate::storage::Backend::primary_with(config.storage_root.clone(), config.local_storage).await?);
        let local_state = backend.local_state();
        launch.boot.lock().expect("startup owners are healthy").local = Some(local_state.clone());
        let application_settings = crate::config::Settings::open(config.application.clone(), local_state.clone()).await?;
        let node_identity = application_settings.read(|document| document.identity.clone()).await?;
        let plugin_preferences = PluginPreferences::new(application_settings.clone());
        let secrets =
            SecretStore::from_config(&config, application_settings.clone()).await.map_err(|error| Error::Operation(error.to_string()))?;
        let sampling = Sampling::new();
        let providers = ProviderExecutorRegistry::new(providers);
        let settings = config.storage.clone();
        let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
            config.storage_root.clone(),
            harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
            config.storage_state,
        ));
        let backends = harmos::storage::StorageBackends {
            primary: backend.clone(),
            state: recovery,
        };
        #[cfg(feature = "fault-injection")]
        let backends = match decorate {
            Some(decorate) => decorate(backends),
            None => backends,
        };
        let storage = crate::storage::Manager::new(backends, settings)?;
        launch.boot.lock().expect("startup owners are healthy").storage = Some(storage.clone());
        let record_source =
            Arc::new(crate::runtime::storage::record_capture::Source::new(&launch.record_namespace, storage.clone(), config.schema)?);
        let resources = crate::app::Resources::new(config.schema);
        *resources.analysis.dispatchers.write().expect("analysis dispatcher registry is healthy") = config.action_dispatchers.clone();
        Ok(BootParts {
            record_namespace: launch.record_namespace,
            config,
            installed_plugins,
            storage: storage.clone(),
            local_state: local_state.clone(),
            record_source,
            resources,
            node_identity,
            application_settings,
            plugin_preferences,
            secrets,
            sampling,
            providers,
            boot_started,
        })
    }

    // Activate Prepared Resources, Recover Receipts And Assemble The Booted Handle.
    pub(crate) async fn initialize(harmos_runtime: &harmos::runtime::Handle<Dokime>, parts: BootParts, prepared: Prepared) -> Result<Self> {
        let BootParts {
            record_namespace: _,
            config,
            installed_plugins,
            storage,
            local_state,
            record_source,
            resources,
            node_identity,
            application_settings,
            plugin_preferences,
            secrets,
            sampling,
            providers,
            boot_started,
        } = parts;
        let Prepared {
            provider_invocations,
            execution_sessions,
            embedded_templates,
            step_builtins,
            record_ingress,
        } = prepared;

        let initial = harmos_runtime
            .state
            .state()
            .read(Clone::clone)
            .await
            .map_err(|error| Error::Operation(format!("failed to read initial Harmos state: {error}")))?;
        let initial_revision = initial.position.0;
        let initial_state = initial.value;
        tracing::info!(
            phase = "storage",
            initial_revision,
            elapsed_ms = boot_started.elapsed().as_millis() as u64,
            "runtime boot complete"
        );

        let initial_snapshot = crate::queries::RuntimeStateSnapshot::new(Arc::new(initial_state.clone()), initial_revision);
        let (state_watch, _) = tokio::sync::watch::channel(initial_snapshot);
        let brand = Arc::new(RwLock::new(None));
        let process_id = crate::model::identity::new_id::<SessionId>();
        let requests_started = std::time::Instant::now();
        let request_storage = crate::runtime::storage::requests::RequestStorage::new(storage.clone(), &node_identity.node_id)?;
        request_storage
            .recover(|id| {
                initial_state.run_requests.get(id).map(|request| &request.summary).filter(|summary| {
                    summary.phase.terminal() && summary.phase != dokime_domain::model::execution::request::RunPhase::Interrupted
                })
            })
            .await?;
        let request_storage = Arc::new(request_storage);
        tracing::info!(phase = "requests", elapsed_ms = requests_started.elapsed().as_millis() as u64, "request receipt recovery complete");
        let execution_storage = Arc::new(
            crate::runtime::storage::executions::ExecutionStorage::new(
                storage.clone(),
                harmos_runtime.state.clone(),
                process_id,
                brand.clone(),
                config.schema,
            )
            .await?,
        );
        let runtime = Self {
            local_storage: local_state.clone(),
            request_storage,
            storage: storage.clone(),
            execution_storage,
            plugin_boot: Arc::new(RwLock::new(BTreeMap::new())),
            state_manager: harmos_runtime.state.clone(),
            record_source,
            streams: harmos_runtime.streams.clone(),
            operations: harmos_runtime.operations.clone(),
            grpc: harmos_runtime.grpc().clone(),
            record_ingress,
            execution_sessions,
            embedded_templates,
            sampling: Arc::new(sampling),
            evidence_admission: Default::default(),
            evidence_exports: crate::domains::exports::evidence::Manager::new(config.application.cache().join("exports")),
            telemetry_admission: Arc::new(tokio::sync::Mutex::new(())),
            telemetry_cursors: Arc::new(Mutex::new(crate::domains::telemetry::TelemetryCursors::default())),
            stream_leases: Arc::new(crate::telemetry_runtime::StreamLeases::new()),
            providers: Arc::new(providers),
            progress: Arc::new(crate::domains::execution::progress::ProgressRegistry::new()),
            operation_progress: crate::runtime::progress::OperationRegistry::new(),
            native_executions: Default::default(),
            record_demands: Default::default(),
            publication_ingest: Default::default(),
            shutdown_requested: tokio::sync::watch::channel(false).0,
            provider_tasks: Default::default(),
            provider_invocations,
            provider_completion_failure: Default::default(),
            owned_work: Default::default(),
            plugin_admission: Default::default(),
            unavailable_plugins: resources.analysis.unavailable.clone(),
            state: Arc::new(RwLock::new(Arc::new(initial_state))),
            state_watch,
            action_dispatchers: resources.analysis.dispatchers.clone(),
            routing: Arc::new(std::sync::OnceLock::new()),
            config: Arc::new(config.clone()),
            settings: application_settings,
            plugin_preferences: Arc::new(plugin_preferences),
            secrets: Arc::new(secrets),
            brand,
        };

        runtime.settle_boot(node_identity, installed_plugins, &step_builtins).await?;
        Ok(runtime)
    }

    // Establish Identity And Activate Prepared Host Resources.
    async fn settle_boot(
        &self, node_identity: dokime_domain::model::identity::DokimeNodeIdentity, installed_plugins: InstalledPlugins,
        step_builtins: &crate::domains::files::StepBuiltins,
    ) -> Result<()> {
        self.commit_keyed(
            harmos::state::RequestKey::new("dokime.node.identity.v2"),
            crate::transactions::EstablishNodeIdentity { candidate: node_identity },
        )
        .await
        .map_err(|error| Error::Operation(format!("failed to establish node identity: {error}")))?;

        step_builtins.bind(self);
        for plugin in installed_plugins.into_values() {
            if self.state_read().host_plugin_current(&plugin) {
                continue;
            }
            crate::prompts::admission::validate_installation(&plugin).map_err(Error::Operation)?;
            self.commit(crate::transactions::InstallHostPlugin { installation: plugin })
                .await
                .map_err(|error| Error::Operation(error.to_string()))?;
        }
        let contracts = self.state_read().telemetry_contracts();
        self.execution_storage.register_schemas(&contracts).await?;
        Ok(())
    }

    // The Listener Shares The Booted Runtime Registry And Shutdown Owner.
    pub fn grpc(&self) -> &harmos::grpc::GrpcManager {
        &self.grpc
    }

    pub fn operations(&self) -> &crate::app::OperationManager {
        &self.operations
    }

    pub fn router(&self) -> &harmos::plugin::Router {
        self.routing.get().expect("plugin session initialized before runtime exposure")
    }

    pub(crate) fn plugin_available(&self, source: &str) -> bool {
        !self.unavailable_plugins.read().expect("plugin availability lock poisoned").contains(source)
    }

    pub(crate) fn remove_action_dispatcher(&self, id: &Identifier) {
        self.action_dispatchers.write().expect("action dispatcher lock poisoned").remove(id);
    }

    pub(crate) fn action_dispatcher(&self, plugin_id: &Identifier) -> Option<Arc<dyn ActionDispatcher>> {
        self.action_dispatchers.read().expect("action dispatcher lock was poisoned").get(plugin_id).cloned()
    }

    pub(crate) async fn analyze_with_dispatcher(
        &self, source: &Identifier, expected: &Arc<dyn ActionDispatcher>, operation: &str, request: crate::analysis::Request,
    ) -> crate::runtime::Result<crate::analysis::Response> {
        let invocation = {
            let dispatchers = self.action_dispatchers.read().expect("action dispatcher lock poisoned");
            if !dispatchers.get(source).is_some_and(|current| Arc::ptr_eq(current, expected)) {
                return Err(Error::Admission(AdmissionFailure::Unavailable {
                    code: "analysis_owner_changed".into(),
                    message: "analysis owner changed while its evidence was being selected".into(),
                }));
            }
            // Admission Is Synchronous; Release The Registry Before Awaiting Work.
            expected.analyze(operation, request)
        };
        invocation.await.map_err(|error| Error::Operation(error.to_string()))
    }

    pub(crate) fn set_action_dispatcher(&self, plugin_id: Identifier, dispatcher: Arc<dyn ActionDispatcher>) {
        self.action_dispatchers.write().expect("action dispatcher lock was poisoned").insert(plugin_id, dispatcher);
    }

    pub(crate) fn config(&self) -> Arc<RuntimeConfig> {
        Arc::clone(&self.config)
    }

    pub fn settings(&self) -> &crate::config::Settings {
        &self.settings
    }

    pub fn schema_settings(&self) -> &harmos::config::SchemaSettings {
        &self.config.schema
    }

    pub(crate) fn plugin_preferences(&self) -> Arc<PluginPreferences> {
        Arc::clone(&self.plugin_preferences)
    }

    pub(crate) fn secret_store(&self) -> Arc<SecretStore> {
        Arc::clone(&self.secrets)
    }

    pub fn subscribe_records(&self) -> RecordSubscription {
        RecordSubscription::new(&self.record_source, &self.state_manager, self.state_manager.applied())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn check_boot_config(config: &RuntimeConfig) -> Result<InstalledPlugins> {
    let mut installed_plugins = BTreeMap::new();
    for plugin in &config.installed_plugins {
        if installed_plugins.insert(plugin.source.clone(), plugin.clone()).is_some() {
            return Err(Error::Operation(format!("installed plugin `{}` is duplicated", plugin.source)));
        }
    }
    Ok(installed_plugins)
}

/* === Tests === */
