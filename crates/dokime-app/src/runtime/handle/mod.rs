/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod records;
mod settings;

/* === Re-Exports === */
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
use crate::runtime::state::RuntimeState;
use crate::transactions::Transactions;
use crate::{
    config::RuntimeConfig,
    credentials::CredentialStore,
    error::{Error, Result},
    execution_runtime::provider::{ExecutionFixture, ProviderExecutorRegistry},
    installed_plugins::module_preferences::PluginPreferences,
    telemetry_runtime::Sampling,
};
use dokime::core::{Identifier, SessionId};
use dokime::domains::ActionDispatcher;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
#[cfg(test)]
type BackendDecorator = Box<
    dyn FnOnce(
            harmos::storage::StorageBackends<dokime::storage::objects::Objects>,
        ) -> harmos::storage::StorageBackends<dokime::storage::objects::Objects>
        + Send,
>;

pub(crate) type InstalledPlugins = BTreeMap<String, InstalledPlugin>;

/* === Structs === */
// The Opened Owners A Boot Hands To Handle Assembly.
struct BootParts {
    config: RuntimeConfig,
    installed_plugins: InstalledPlugins,
    storage: dokime::storage::Manager,
    local_state: dokime::storage::sqlite::State,
    storage_policy: Arc<crate::config::policy::PolicyStore>,
    record_source: Arc<crate::runtime::storage::record_capture::Source>,
    resources: crate::app::Resources,
    node_identity: dokime::core::DokimeNodeIdentity,
    application_settings: crate::config::Settings,
    plugin_preferences: PluginPreferences,
    credentials: CredentialStore,
    sampling: Sampling,
    providers: ProviderExecutorRegistry,
    boot_started: std::time::Instant,
}
struct BootRuntime(Option<HarmosRuntime>);

pub(crate) struct HostTraceabilityIndex {
    pub(crate) generation: u64,
    pub(crate) samples: BTreeMap<String, dokime::telemetry::TelemetrySample>,
}

#[derive(Clone)]
pub struct RuntimeHandle {
    storage: dokime::storage::Manager,
    pub(crate) local_storage: dokime::storage::sqlite::State,
    pub(crate) storage_policy: Arc<crate::config::policy::PolicyStore>,
    pub(crate) request_storage: Arc<crate::runtime::storage::requests::RequestStorage>,
    pub(crate) execution_storage: Arc<crate::runtime::storage::executions::ExecutionStorage>,
    pub(crate) plugin_boot: Arc<RwLock<BTreeMap<uuid::Uuid, bool>>>,
    pub(crate) state_manager: harmos::state::StateManager<Dokime>,
    pub(crate) record_source: Arc<crate::runtime::storage::record_capture::Source>,
    pub(crate) streams: crate::app::Streams,
    pub(crate) operations: harmos::operations::OperationManager,
    grpc: harmos::grpc::GrpcManager,
    pub(crate) publication_core: Arc<super::sidecars::core::Publications>,
    pub(crate) record_ingress: Arc<super::sidecars::core::RecordIngress>,
    pub(crate) execution_sessions: Arc<crate::domains::execution::run::Sessions>,

    pub(crate) sampling: Arc<Sampling>,
    pub(crate) stream_leases: Arc<crate::telemetry_runtime::StreamLeases>,
    pub(crate) evidence_admission: Arc<tokio::sync::Mutex<()>>,
    pub(crate) telemetry_admission: Arc<tokio::sync::Mutex<()>>,
    pub(crate) telemetry_cursors: Arc<Mutex<BTreeMap<String, u64>>>,
    pub(crate) host_traceability: Arc<Mutex<HostTraceabilityIndex>>,
    pub(crate) host_telemetry_source:
        Arc<RwLock<Option<Arc<crate::telemetry_runtime::HostTelemetrySource>>>>,
    pub(crate) providers: Arc<ProviderExecutorRegistry>,
    pub(crate) progress: Arc<crate::domains::execution::progress::ProgressRegistry>,
    pub(crate) operation_progress: Arc<crate::runtime::progress::OperationRegistry>,
    pub(crate) native_executions: super::native_telemetry::Executions,
    pub(crate) record_demands: Arc<Mutex<super::sidecars::RecordDemands>>,
    pub(crate) publication_ingest: Arc<std::sync::OnceLock<super::sidecars::PublicationIngest>>,
    pub(crate) shutdown_requested: tokio::sync::watch::Sender<bool>,
    pub(crate) provider_tasks: Arc<Mutex<Vec<crate::operations::provider::ProviderTask>>>,
    pub(crate) provider_completion_failure:
        Arc<Mutex<Option<crate::operations::provider::PublicationFailure>>>,
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
    credentials: Arc<CredentialStore>,
    pub(crate) brand: Arc<RwLock<Option<crate::installed_plugins::brand::ResolvedBrand>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Drop for BootRuntime {
    fn drop(&mut self) {
        if let Some(runtime) = self.0.take() {
            tokio::spawn(async move {
                if let Err(error) = runtime.stop().await {
                    tracing::error!(%error, "cancelled runtime initialization cleanup failed");
                }
            });
        }
    }
}

impl RuntimeHandle {
    /// Admit detached work that shutdown drains, or `None` once shutdown closed admission.
    pub(crate) fn admit_owned_work(
        &self,
    ) -> Option<tokio_util::task::task_tracker::TaskTrackerToken> {
        // Take The Token First So A Concurrent Close Either Waits For It Or Is Seen.
        let token = self.owned_work.token();
        (!self.owned_work.is_closed()).then_some(token)
    }

    pub fn storage(&self) -> &dokime::storage::Manager {
        &self.storage
    }

    pub fn streams(&self) -> &harmos::streams::StreamManager<Dokime> {
        &self.streams
    }

    /// The slow operations this runtime is performing; `begin` one around
    /// work that may keep an operator waiting.
    pub fn operation_progress(&self) -> &Arc<crate::runtime::progress::OperationRegistry> {
        &self.operation_progress
    }

    pub fn watch_execution_progress(&self) -> crate::domains::execution::progress::ProgressWatch {
        self.progress.watch()
    }

    pub(crate) async fn commit<T>(
        &self,
        transaction: T,
    ) -> std::result::Result<harmos::state::Receipt, harmos::Error<T::Error>>
    where
        T: harmos::state::Transaction<RuntimeState> + Into<Transactions>,
        T::Error: From<String>,
    {
        let receipt = self.state_manager.commit(transaction).await?;
        self.refresh_state()
            .await
            .map_err(|_| harmos::Error::Unavailable)?;
        Ok(receipt)
    }

    pub(crate) async fn commit_keyed<T>(
        &self,
        key: harmos::state::RequestKey,
        transaction: T,
    ) -> std::result::Result<harmos::state::Receipt, harmos::Error<T::Error>>
    where
        T: harmos::state::Transaction<RuntimeState> + Into<Transactions>,
        T::Error: From<String>,
    {
        let receipt = self.state_manager.commit_keyed(key, transaction).await?;
        self.refresh_state()
            .await
            .map_err(|_| harmos::Error::Unavailable)?;
        Ok(receipt)
    }

    #[tracing::instrument(
        level = "info",
        name = "runtime.boot",
        skip_all,
        fields(session_id = %config.session_id),
    )]
    pub(crate) async fn boot(
        config: RuntimeConfig,
        providers: BTreeMap<String, Arc<dyn ExecutionFixture>>,
        install_builtins: bool,
    ) -> Result<(HarmosRuntime, Self)> {
        #[cfg(test)]
        {
            Self::boot_inner(config, providers, install_builtins, None).await
        }
        #[cfg(not(test))]
        {
            Self::boot_inner(config, providers, install_builtins).await
        }
    }

    #[cfg(test)]
    pub(crate) async fn boot_with_backends(
        config: RuntimeConfig,
        providers: BTreeMap<String, Arc<dyn ExecutionFixture>>,
        install_builtins: bool,
        decorate: BackendDecorator,
    ) -> Result<(HarmosRuntime, Self)> {
        Self::boot_inner(config, providers, install_builtins, Some(decorate)).await
    }

    async fn boot_inner(
        config: RuntimeConfig,
        providers: BTreeMap<String, Arc<dyn ExecutionFixture>>,
        install_builtins: bool,
        #[cfg(test)] decorate: Option<BackendDecorator>,
    ) -> Result<(HarmosRuntime, Self)> {
        let boot_started = std::time::Instant::now();
        let installed_plugins = check_boot_config(&config, boot_started)?;
        config.storage.validate()?;
        dokime_filesystem::publication::create_dir_all(&config.storage_root)?;
        let backend =
            Arc::new(dokime::storage::Backend::primary(config.storage_root.clone()).await?);
        let local_state = backend.local_state();
        let (config, application_settings, storage_policy) =
            tokio::task::spawn_blocking(move || {
                let mut config = config;
                let settings = crate::config::Settings::load(&config.application_root)?;
                let policy =
                    crate::config::policy::PolicyStore::open(&mut config, settings.clone())?;
                Ok::<_, Error>((config, settings, Arc::new(policy)))
            })
            .await??;
        let node_identity = application_settings.read(|document| document.identity.clone());
        let plugin_preferences = PluginPreferences::new(application_settings.clone());
        let credentials = CredentialStore::from_config(&config.credential_backend)
            .map_err(|error| Error::Operation(error.to_string()))?;
        let mut state = RuntimeState::new(config.session_id.clone());
        state.record_namespace = dokime::core::SessionId::new();
        if install_builtins {
            state.install_builtin_domains().map_err(|error| {
                Error::Operation(format!("failed to install built-in domains: {error}"))
            })?;
        }

        let sampling = Sampling::default();
        let providers = ProviderExecutorRegistry::new(providers);
        let settings = config.storage.clone();
        let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
            config.storage_root.clone(),
            harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
            dokime::storage::state_settings(&settings),
        ));
        tracing::info!(
            read_pool_bytes = settings.budget.read_memory_bytes.get(),
            "storage read pool sized"
        );
        let backends = harmos::storage::StorageBackends {
            primary: backend.clone(),
            state: recovery,
        };
        #[cfg(test)]
        let backends = match decorate {
            Some(decorate) => decorate(backends),
            None => backends,
        };
        let storage = dokime::storage::Manager::new(backends, settings)?;
        let record_source = Arc::new(crate::runtime::storage::record_capture::Source::new(
            state.process_epoch(),
            storage.clone(),
        )?);
        let resources = crate::app::Resources::new(config.storage.clone());
        *resources
            .analysis
            .dispatchers
            .write()
            .expect("analysis dispatcher registry is healthy") = config.action_dispatchers.clone();
        let harmos_runtime = crate::runtime::storage::build(
            state,
            record_source.clone(),
            &storage,
            resources.clone(),
        )
        .await;
        let harmos_runtime = match harmos_runtime {
            Ok(runtime) => runtime,
            Err(mut source) => {
                // Settle The Original Bootstrap Owner Before Closing Its Control Lease
                if source.retry_cleanup().await.is_err() {
                    return Err(Error::RuntimeBoot {
                        source: Box::new(source),
                    });
                }
                let source = Error::RuntimeBoot {
                    source: Box::new(source),
                };
                let rollback = crate::runtime::rollback::Receipt::boot(None, storage, local_state);
                return Err(match rollback.retry().await {
                    Ok(()) => source,
                    Err(cleanup) => Error::StartupCleanup {
                        source: Box::new(source),
                        cleanup: Box::new(cleanup),
                        rollback,
                    },
                });
            }
        };
        let mut boot = BootRuntime(Some(harmos_runtime));
        let harmos_runtime = boot.0.as_ref().expect("boot owns the runtime");
        let parts = BootParts {
            config,
            installed_plugins,
            storage: storage.clone(),
            local_state: local_state.clone(),
            storage_policy,
            record_source,
            resources,
            node_identity,
            application_settings,
            plugin_preferences,
            credentials,
            sampling,
            providers,
            boot_started,
        };
        let initialized = Self::initialize(harmos_runtime, parts).await;
        match initialized {
            Ok(runtime) => Ok((boot.0.take().expect("boot owns the runtime"), runtime)),
            Err(source) => {
                // Preserve the existing cancellation guard until physical cleanup
                // settles. A failed cleanup transfers every remaining owner into
                // the returned receipt rather than logging and discarding it.
                let cleanup = async {
                    harmos_runtime.stop().await?;
                    storage.close().await?;
                    local_state.close().await?;
                    Ok::<_, Error>(())
                }
                .await;
                match cleanup {
                    Ok(()) => {
                        drop(boot.0.take());
                        Err(source)
                    }
                    Err(cleanup) => Err(Error::StartupCleanup {
                        source: Box::new(source),
                        cleanup: Box::new(cleanup),
                        rollback: crate::runtime::rollback::Receipt::boot(
                            boot.0.take(),
                            storage,
                            local_state,
                        ),
                    }),
                }
            }
        }
    }

    // Register Operations, Recover Receipts And Assemble The Booted Handle.
    async fn initialize(harmos_runtime: &HarmosRuntime, parts: BootParts) -> Result<Self> {
        let BootParts {
            config,
            installed_plugins,
            storage,
            local_state,
            storage_policy,
            record_source,
            resources,
            node_identity,
            application_settings,
            plugin_preferences,
            credentials,
            sampling,
            providers,
            boot_started,
        } = parts;
        harmos::storage::operations::Operations::register(
            &harmos_runtime.operations,
            "dokime.storage",
            crate::app::Lanes::Open,
            Arc::new(storage.clone()),
        )?;
        crate::operations::provider::register(&harmos_runtime.operations)?;
        let execution_sessions = Arc::new(crate::domains::execution::run::Sessions::default());
        crate::domains::execution::run::host::register(
            &harmos_runtime.operations,
            &execution_sessions,
        )?;
        crate::plot::builtins::register(&harmos_runtime.operations)?;
        let step_builtins = crate::domains::files::register(&harmos_runtime.operations)?;
        let publication_core =
            super::sidecars::core::Publications::register(&harmos_runtime.operations)?;
        let record_ingress =
            super::sidecars::core::RecordIngress::register(&harmos_runtime.operations)?;

        let initial = harmos_runtime
            .state
            .state()
            .read(Clone::clone)
            .await
            .map_err(|error| {
                Error::Operation(format!("failed to read initial Harmos state: {error}"))
            })?;
        let initial_revision = initial.position.0;
        let initial_state = initial.value;
        tracing::info!(
            phase = "storage",
            initial_revision,
            elapsed_ms = boot_started.elapsed().as_millis() as u64,
            "runtime boot complete"
        );

        let initial_snapshot = crate::queries::RuntimeStateSnapshot::new(
            Arc::new(initial_state.clone()),
            initial_revision,
        );
        let (state_watch, _) = tokio::sync::watch::channel(initial_snapshot);
        let brand = Arc::new(RwLock::new(None));
        let process_id = SessionId::new();
        let requests_started = std::time::Instant::now();
        let request_storage = crate::runtime::storage::requests::RequestStorage::new(
            storage.clone(),
            node_identity.node_id.clone(),
        );
        request_storage
            .recover(|id| {
                initial_state
                    .run_requests
                    .get(id)
                    .map(|request| &request.summary)
                    .filter(|summary| {
                        summary.phase.terminal()
                            && summary.phase != dokime::execution::request::RunPhase::Interrupted
                    })
            })
            .await?;
        let request_storage = Arc::new(request_storage);
        tracing::info!(
            phase = "requests",
            elapsed_ms = requests_started.elapsed().as_millis() as u64,
            "request receipt recovery complete"
        );
        let execution_storage = Arc::new(
            crate::runtime::storage::executions::ExecutionStorage::new(
                storage.clone(),
                harmos_runtime.state.clone(),
                process_id,
                brand.clone(),
            )
            .await?,
        );
        let runtime = Self {
            local_storage: local_state.clone(),
            storage_policy,
            request_storage,
            storage: storage.clone(),
            execution_storage,
            plugin_boot: Arc::new(RwLock::new(BTreeMap::new())),
            state_manager: harmos_runtime.state.clone(),
            record_source,
            streams: harmos_runtime.streams.clone(),
            operations: harmos_runtime.operations.clone(),
            grpc: harmos_runtime.grpc().clone(),
            publication_core,
            record_ingress,
            execution_sessions,
            sampling: Arc::new(sampling),
            evidence_admission: Default::default(),
            telemetry_admission: Arc::new(tokio::sync::Mutex::new(())),
            telemetry_cursors: Arc::new(Mutex::new(BTreeMap::new())),
            host_traceability: Arc::new(Mutex::new(HostTraceabilityIndex {
                generation: 0,
                samples: BTreeMap::new(),
            })),
            stream_leases: Arc::new(crate::telemetry_runtime::StreamLeases::default()),
            host_telemetry_source: Arc::new(RwLock::new(None)),
            providers: Arc::new(providers),
            progress: Arc::new(crate::domains::execution::progress::ProgressRegistry::new()),
            operation_progress: crate::runtime::progress::OperationRegistry::new(),
            native_executions: Default::default(),
            record_demands: Default::default(),
            publication_ingest: Default::default(),
            shutdown_requested: tokio::sync::watch::channel(false).0,
            provider_tasks: Default::default(),
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
            credentials: Arc::new(credentials),
            brand,
        };

        runtime
            .settle_boot(node_identity, installed_plugins, &step_builtins)
            .await?;
        Ok(runtime)
    }

    // Establish Identity, Install Host Plugins And Register Late Operations.
    async fn settle_boot(
        &self,
        node_identity: dokime::core::DokimeNodeIdentity,
        installed_plugins: InstalledPlugins,
        step_builtins: &crate::domains::files::StepBuiltins,
    ) -> Result<()> {
        self.commit_keyed(
            harmos::state::RequestKey::new("dokime.node.identity.v2"),
            crate::transactions::EstablishNodeIdentity {
                candidate: node_identity,
            },
        )
        .await
        .map_err(|error| Error::Operation(format!("failed to establish node identity: {error}")))?;

        step_builtins.bind(self);
        for plugin in installed_plugins.into_values() {
            if self.state_read().host_plugin_current(&plugin) {
                continue;
            }
            self.commit(crate::transactions::InstallHostPlugin {
                installation: plugin,
            })
            .await
            .map_err(|error| Error::Operation(error.to_string()))?;
        }
        let contracts = self.state_read().telemetry_contracts();
        crate::runtime::storage::telemetry::register_schemas(self.storage(), &contracts).await?;
        crate::operations::runtime::register(self)?;
        crate::operations::reports::register(self)?;
        Ok(())
    }

    pub(crate) fn installed_plugins(&self) -> Arc<InstalledPlugins> {
        Arc::new(self.state_read().installed_plugins.clone())
    }

    // The Listener Shares The Booted Runtime Registry And Shutdown Owner.
    pub fn grpc(&self) -> &harmos::grpc::GrpcManager {
        &self.grpc
    }

    pub fn operations(&self) -> &harmos::operations::OperationManager {
        &self.operations
    }

    pub fn router(&self) -> &harmos::plugin::Router {
        self.routing
            .get()
            .expect("plugin session initialized before runtime exposure")
    }

    pub(crate) fn plugin_available(&self, source: &str) -> bool {
        !self
            .unavailable_plugins
            .read()
            .expect("plugin availability lock poisoned")
            .contains(source)
    }

    pub(crate) fn remove_action_dispatcher(&self, id: &Identifier) {
        self.action_dispatchers
            .write()
            .expect("action dispatcher lock poisoned")
            .remove(id);
    }

    pub(crate) fn action_dispatcher(
        &self,
        plugin_id: &Identifier,
    ) -> Option<Arc<dyn ActionDispatcher>> {
        self.action_dispatchers
            .read()
            .expect("action dispatcher lock was poisoned")
            .get(plugin_id)
            .cloned()
    }

    pub(crate) async fn analyze_with_dispatcher(
        &self,
        source: &Identifier,
        expected: &Arc<dyn ActionDispatcher>,
        operation: &str,
        request: dokime::analysis::Request,
    ) -> crate::Result<dokime::analysis::Response> {
        let invocation = {
            let dispatchers = self
                .action_dispatchers
                .read()
                .expect("action dispatcher lock poisoned");
            if !dispatchers
                .get(source)
                .is_some_and(|current| Arc::ptr_eq(current, expected))
            {
                return Err(Error::Unavailable {
                    code: "analysis_owner_changed".into(),
                    message: "analysis owner changed while its evidence was being selected".into(),
                });
            }
            // Admission Is Synchronous; Release The Registry Before Awaiting Work.
            expected.analyze(operation, request)
        };
        invocation
            .await
            .map_err(|error| Error::Operation(error.to_string()))
    }

    pub(crate) fn set_action_dispatcher(
        &self,
        plugin_id: Identifier,
        dispatcher: Arc<dyn ActionDispatcher>,
    ) {
        self.action_dispatchers
            .write()
            .expect("action dispatcher lock was poisoned")
            .insert(plugin_id, dispatcher);
    }

    pub(crate) fn config(&self) -> Arc<RuntimeConfig> {
        Arc::clone(&self.config)
    }

    pub fn settings(&self) -> &crate::config::Settings {
        &self.settings
    }

    pub(crate) fn plugin_preferences(&self) -> Arc<PluginPreferences> {
        Arc::clone(&self.plugin_preferences)
    }

    pub(crate) fn credential_store(&self) -> Arc<CredentialStore> {
        Arc::clone(&self.credentials)
    }

    pub fn subscribe_records(&self) -> RecordSubscription {
        RecordSubscription::new(
            &self.record_source,
            &self.state_manager,
            self.state_manager.applied(),
        )
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn check_boot_config(
    config: &RuntimeConfig,
    boot_started: std::time::Instant,
) -> Result<InstalledPlugins> {
    if config.compaction_interval.is_zero()
        || boot_started
            .checked_add(config.compaction_interval)
            .is_none()
    {
        return Err(Error::Operation(
            "compaction interval must be positive and representable".into(),
        ));
    }
    let mut installed_plugins = BTreeMap::new();
    if config
        .local_retention
        .is_some_and(|duration| duration.is_zero() || i64::try_from(duration.as_micros()).is_err())
    {
        return Err(Error::Operation(
            "local retention must be positive and fit the timestamp range".into(),
        ));
    }
    for plugin in &config.installed_plugins {
        if installed_plugins
            .insert(plugin.source.clone(), plugin.clone())
            .is_some()
        {
            return Err(Error::Operation(format!(
                "installed plugin `{}` is duplicated",
                plugin.source
            )));
        }
    }
    Ok(installed_plugins)
}

/* === Tests === */
