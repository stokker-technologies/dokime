/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod command;
pub(crate) mod effects;
mod embedded;
pub mod error;
pub(crate) mod handle;
mod native_telemetry;
pub(crate) mod operations;
mod plugins;
pub mod progress;
pub mod query;
pub mod rollback;
pub(crate) mod sidecars;
pub mod state;
pub(crate) mod storage;

/* === Re-Exports === */

pub use crate::execution::context::ExecutionCycleContext;
pub(crate) use embedded::Templates as EmbeddedTemplates;
pub use embedded::{EmbeddedHost, EmbeddedObserver, EmbeddedProvider};
pub use error::{Error, Result};
pub use handle::{CapturedRecord, CapturedRecords, RuntimeHandle};
pub(crate) use native_telemetry::Admission as ExecutionPublicationAdmission;
pub use state::RuntimeState;
pub use storage::artifacts::{ArtifactReader, ArtifactUpload};
pub use storage::report_open::{OpenedReport, ReportOrigin};

// Reachable only from this crate's test targets. Kept at the root so those
// targets name one path, not a private module tree.
pub use handle::RecordSubscription;
pub use storage::{
    context_records::ContextRecord,
    execution::ExecutionView,
    records::RecordReader,
    telemetry::{TelemetryPage, TelemetryReader},
};

/* === Imports === */

// Standard Library Imports
// Named by this crate's own domain modules, by nobody outside it.
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use futures_util::future::BoxFuture;

// Local Imports
use crate::app::HarmosRuntime;
use crate::{config::RuntimeConfig, execution_runtime::provider::ExecutionFixture};
use query::RuntimeOverviewSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Runtime {
    handle: RuntimeHandle,
    harmos: HarmosRuntime,
    background: Vec<Box<dyn Send>>,
    background_tasks: Vec<BackgroundTask>,
    host: Arc<tokio::sync::Mutex<plugins::PluginHost>>,
    shutdown: Shutdown,
}

pub(crate) struct BackgroundTask(pub(crate) tokio::task::JoinHandle<()>);

struct Starting(Option<Runtime>);

#[derive(Default)]
struct Shutdown {
    // What the stop is doing now, for a host that bounds it.
    stage: &'static str,
    providers: Vec<crate::runtime::operations::provider::ProviderTask>,
    prepared: bool,
    participant: Option<harmos::runtime::StopParticipant>,
    overview: Option<RuntimeOverviewSnapshot>,
    stopped: bool,
    complete: bool,
    task_failure: Option<Arc<tokio::task::JoinError>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for BackgroundTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl Drop for Starting {
    fn drop(&mut self) {
        if let Some(mut runtime) = self.0.take() {
            tokio::spawn(async move {
                if let Err(error) = runtime.shutdown().await {
                    tracing::error!(%error, "cancelled plugin initialization cleanup failed");
                }
            });
        }
    }
}

impl Runtime {
    pub async fn start(config: RuntimeConfig) -> Result<Self> {
        let starting: BoxFuture<'_, Result<Self>> = Box::pin(Self::start_inner(config));
        starting.await
    }

    async fn start_inner(mut config: RuntimeConfig) -> Result<Self> {
        let mut providers = BTreeMap::new();
        for provider in std::mem::take(&mut config.embedded_providers) {
            let provider = embedded::Active::prepare(provider, &config.schema)?;
            let source = provider.definition.id.clone();
            if providers.insert(source.clone(), provider).is_some() {
                return Err(crate::runtime::Error::Operation(format!("duplicate embedded provider `{source}`")));
            }
        }

        let runtime = Self::from_parts(RuntimeHandle::boot(config, BTreeMap::new(), true).await?);
        runtime.host.lock().await.embedded = providers;
        runtime.finish_start().await
    }

    pub async fn with_execution_fixture(
        config: RuntimeConfig, source: impl Into<String>, fixture: Arc<dyn ExecutionFixture>,
    ) -> Result<Self> {
        Self::with_execution_fixtures(config, [(source.into(), fixture)]).await
    }

    pub async fn with_execution_fixtures(
        config: RuntimeConfig, fixtures: impl IntoIterator<Item = (String, Arc<dyn ExecutionFixture>)>,
    ) -> Result<Self> {
        Self::with_execution_fixtures_inner(config, fixtures, true).await
    }

    pub async fn with_execution_fixtures_without_builtins(
        config: RuntimeConfig, fixtures: impl IntoIterator<Item = (String, Arc<dyn ExecutionFixture>)>,
    ) -> Result<Self> {
        Self::with_execution_fixtures_inner(config, fixtures, false).await
    }

    #[cfg(feature = "fault-injection")]
    pub async fn with_fixture_backends(
        config: RuntimeConfig, fixtures: impl IntoIterator<Item = (String, Arc<dyn ExecutionFixture>)>,
        decorate: impl FnOnce(harmos::storage::StorageBackends) -> harmos::storage::StorageBackends + Send + 'static,
    ) -> Result<Self> {
        let fixtures = fixtures.into_iter().collect();
        let parts = RuntimeHandle::boot_with_backends(config, fixtures, true, Box::new(decorate)).await?;
        let runtime = Self::from_parts(parts);
        runtime.finish_start().await
    }

    async fn with_execution_fixtures_inner(
        config: RuntimeConfig, fixtures: impl IntoIterator<Item = (String, Arc<dyn ExecutionFixture>)>, install_builtins: bool,
    ) -> Result<Self> {
        let fixtures = fixtures.into_iter().collect();
        Self::start_with_fixtures(config, fixtures, install_builtins).await
    }

    fn start_with_fixtures(
        config: RuntimeConfig, fixtures: BTreeMap<String, Arc<dyn ExecutionFixture>>, install_builtins: bool,
    ) -> BoxFuture<'static, Result<Self>> {
        Box::pin(async move {
            let runtime = Self::from_parts(RuntimeHandle::boot(config, fixtures, install_builtins).await?);
            runtime.finish_start().await
        })
    }

    async fn finish_start(self) -> Result<Self> {
        let mut starting = Starting(Some(self));
        let runtime = starting.0.as_mut().expect("startup owns its runtime");
        let plugins_started = std::time::Instant::now();
        let outcome = runtime.host.lock().await.install_plugins().await;
        tracing::info!(phase = "plugins", elapsed_ms = plugins_started.elapsed().as_millis() as u64, "plugin installation complete");
        if let Err(source) = outcome {
            if let Err(cleanup) = runtime.shutdown().await {
                let runtime = starting.0.take().expect("startup owns its runtime");
                return Err(crate::runtime::Error::StartupCleanup {
                    source: Box::new(source),
                    cleanup: Box::new(cleanup),
                    rollback: rollback::Receipt::new(runtime),
                });
            }
            starting.0 = None;
            return Err(source);
        }
        let restarter = plugins::restarts::spawn(Arc::clone(&runtime.host), runtime.handle.clone());
        runtime.background_tasks.push(restarter);
        Ok(starting.0.take().expect("startup owns its runtime"))
    }

    fn from_parts((harmos, handle): (HarmosRuntime, RuntimeHandle)) -> Self {
        let (publications, ingest) = sidecars::spawn_publication_ingest(handle.clone());
        assert!(handle.publication_ingest.set(publications.clone()).is_ok(), "publication ingest initializes once");
        let forwarding = sidecars::spawn_record_forwarding(handle.clone());
        handle.record_ingress.attach(handle.clone());
        let lease_gaps = sidecars::spawn_lease_gap_watch(handle.clone(), harmos.operations.watch_residents());
        Self {
            harmos,
            background: Vec::new(),
            background_tasks: vec![ingest, forwarding, lease_gaps],
            host: Arc::new(tokio::sync::Mutex::new(plugins::PluginHost::new(handle.clone(), publications))),
            handle,
            shutdown: Shutdown::default(),
        }
    }

    pub fn fixture_operation_caller(&self, owner: &str) -> Result<harmos::operations::Caller> {
        let host = self.host.try_lock().map_err(|_| crate::runtime::Error::Operation("plugin host is busy".into()))?;
        let manager =
            host.plugin_manager.as_ref().ok_or_else(|| crate::runtime::Error::Operation("plugin manager is not installed".into()))?;
        let source = manager.caller(owner)?;
        self.harmos.operations.caller(&source).map_err(Into::into)
    }

    // The Running Generation Of `owner`, With The Definition It Was Prepared From.
    pub async fn fixture_plugin_status(&self, owner: &str) -> Option<harmos::plugin::PluginStatus> {
        self.host.lock().await.plugin_manager.as_ref()?.status(owner)
    }

    pub async fn register(&mut self, provider: Arc<dyn EmbeddedProvider>) -> Result<()> {
        self.host.lock().await.register(provider).await
    }

    pub async fn reinitialize_plugin(&mut self, plugin: &str) -> Result<()> {
        self.host.lock().await.reinitialize_plugin(plugin).await
    }

    pub async fn reload_plugin(&mut self, input: crate::plugins::PluginBundle) -> Result<()> {
        self.host.lock().await.reload_plugin(input).await
    }

    pub fn retain<T: Send + 'static>(&mut self, integration: T) {
        self.background.push(Box::new(integration));
    }

    pub fn handle(&self) -> RuntimeHandle {
        self.handle.clone()
    }

    pub async fn shutdown(&mut self) -> Result<RuntimeOverviewSnapshot> {
        let stopping: BoxFuture<'_, Result<RuntimeOverviewSnapshot>> = Box::pin(self.shutdown_inner());
        stopping.await
    }

    async fn shutdown_inner(&mut self) -> Result<RuntimeOverviewSnapshot> {
        if self.shutdown.complete {
            self.handle.execution_storage.capture_failure_result()?;
            self.handle.provider_completion_result()?;
            if let Some(source) = &self.shutdown.task_failure {
                return Err(crate::runtime::Error::Harmos(harmos::Error::TaskFailed {
                    domain: "Dokime shutdown task",
                    source: source.clone(),
                }));
            }
            return Ok(self.shutdown.overview.as_ref().expect("completed checkpoint").clone());
        }

        let shutdown_started = std::time::Instant::now();
        self.shutdown.stage = "settling running executions";
        self.handle.shutdown_requested.send_replace(true);
        let report_ids = self
            .handle
            .state_read()
            .executions
            .values()
            .filter(|execution| execution.saved.is_none() && execution.finished_at.is_none())
            .map(|execution| execution.test_execution_id.clone())
            .collect::<Vec<_>>();
        self.handle.sampling().fence_executions(&report_ids).await;
        self.handle.owned_work.close();
        self.handle.owned_work.wait().await;

        if !self.shutdown.prepared {
            let admission = self.handle.plugin_admission.lock().await;
            let shutdown_admission = if !self.handle.state_read().shutting_down {
                self.handle
                    .commit(crate::transactions::StartShutdown)
                    .await
                    .map(|_| ())
                    .map_err(|source| crate::runtime::Error::ShutdownStart { source })
            } else {
                Ok(())
            };
            drop(admission);

            loop {
                self.shutdown.providers.extend(std::mem::take(&mut *self.handle.provider_tasks.lock().unwrap()));
                if self.shutdown.providers.is_empty() {
                    break;
                }
                while let Some(provider) = self.shutdown.providers.last_mut() {
                    let outcome = (&mut provider.task).await;
                    self.shutdown.providers.pop();
                    if let Err(source) = outcome {
                        let source = Arc::new(source);
                        self.shutdown.task_failure.get_or_insert(source);
                    }
                }
            }
            // Monitors Settle Their Invocations; Orphans Still Settle Before Plugins Stop.
            self.handle.provider_invocations.settle_all().await;
            self.handle.stop_report_jobs().await;

            // Cancellation Must Settle Even If Native Shutdown Admission Failed.
            shutdown_admission?;

            // Active Cycles Stay Active: The Next Start Seals Them Without Replaying Device Work,
            // So A Stop Never Renders A Cycle Report Its Members Already Wrote.

            self.shutdown.stage = "stopping plugins";
            let mut host = self.host.lock().await;
            let host = &mut *host;
            if let Some(manager) = &mut host.plugin_manager {
                let mut lifecycle = plugins::lifecycle::Adapter {
                    handle: &host.handle,
                    publications: &host.publications,
                    packages: &mut host.plugins,
                    embedded: &mut host.embedded,
                };
                // A plugin that outstays its grace loses its sidecar: dropping the manager ends it.
                let grace_ms = 5000;
                let grace = std::time::Duration::from_millis(grace_ms);
                match tokio::time::timeout(grace, manager.shutdown(&mut lifecycle)).await {
                    Ok(stopped) => stopped?,
                    Err(_) => tracing::warn!(grace_ms, "plugins did not stop in time; ending their sidecars"),
                }
            }
            host.embedded.clear();
            host.plugin_manager = None;
            self.background.clear();
            self.handle.record_ingress.detach();

            for task in &self.background_tasks {
                task.0.abort();
            }
            while let Some(task) = self.background_tasks.last_mut() {
                let outcome = (&mut task.0).await;
                self.background_tasks.pop();
                if let Err(source) = outcome
                    && !source.is_cancelled()
                {
                    let source = Arc::new(source);
                    self.shutdown.task_failure.get_or_insert(source);
                }
            }
            for package in host.plugins.values_mut() {
                package.release_dispatchers(&self.handle);
            }
            self.shutdown.prepared = true;
        }

        self.shutdown.stage = "writing the final checkpoint";
        if !self.shutdown.stopped {
            if self.shutdown.overview.is_none() {
                let participant = self.shutdown.participant.get_or_insert_with(|| self.harmos.stop_participant());
                let stopping = self.harmos.shutdown();
                tokio::pin!(stopping);
                let checkpoint = async {
                    participant.stopping().await;
                    let _evidence = self.handle.evidence_admission.lock().await;
                    let _admission = self.handle.telemetry_admission.lock().await;
                    self.handle.execution_storage.checkpoint_shutdown().await?;
                    self.handle.runtime_overview().await
                };
                let overview = tokio::select! {
                    overview = checkpoint => overview?,
                    stopped = &mut stopping => {
                        stopped?;
                        return Err(crate::runtime::Error::Operation(
                            "runtime stopped before its final checkpoint".into(),
                        ));
                    }
                };
                self.shutdown.overview = Some(overview);
                self.shutdown.participant.take().expect("checkpoint participant").finish();
                stopping.await?;
            } else {
                self.harmos.shutdown().await?;
            }
            self.shutdown.stopped = true;
        }

        let mut host = self.host.lock().await;
        while let Some((name, package)) = host.plugins.iter_mut().next() {
            package
                .close()
                .await
                .map_err(|error| crate::runtime::Error::Operation(format!("failed to remove extracted plugin `{name}`: {error}",)))?;
            host.plugins.pop_first();
        }
        host.pending_plugins
            .close()
            .await
            .map_err(|error| crate::runtime::Error::Operation(format!("failed to remove staged plugin extraction: {error}",)))?;

        self.handle.local_storage.close().await?;

        self.shutdown.complete = true;
        self.handle.execution_storage.capture_failure_result()?;
        self.handle.provider_completion_result()?;
        if let Some(source) = &self.shutdown.task_failure {
            return Err(crate::runtime::Error::Harmos(harmos::Error::TaskFailed {
                domain: "Dokime shutdown task",
                source: source.clone(),
            }));
        }
        let overview = self.shutdown.overview.as_ref().expect("completed checkpoint").clone();
        tracing::info!(
            revision = overview.revision,
            elapsed_ms = shutdown_started.elapsed().as_millis() as u64,
            "runtime shutdown complete"
        );
        Ok(overview)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
