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
pub use embedded::EmbeddedProvider;
pub use error::{Error, Result};
pub use handle::{CapturedRecord, CapturedRecords, RuntimeHandle};
pub(crate) use native_telemetry::Admission as ExecutionPublicationAdmission;
pub use state::RuntimeState;
pub use storage::artifacts::{ArtifactReader, ArtifactUpload};
pub use storage::report_open::{OpenedReport, ReportOrigin};

// Reachable only from this crate's test targets. Kept at the root so those
// targets name one path, not a private module tree.

pub use handle::RecordSubscription;

pub use storage::{context_records::ContextRecord, execution::ExecutionView, records::RecordReader, telemetry::TelemetryReader};

// Named by this crate's own domain modules, by nobody outside it.

/* === Imports === */
// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};
// Third Party Imports

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
    plugins: BTreeMap<String, plugins::ActivePackage>,
    embedded: BTreeMap<String, embedded::Active>,
    pending_plugins: plugins::StagedPackages,
    plugin_manager: Option<plugins::lifecycle::Manager>,
    publications: sidecars::PublicationIngest,
    shutdown: Shutdown,
}

pub(crate) struct BackgroundTask(pub(crate) tokio::task::JoinHandle<()>);

#[derive(Default)]
struct Shutdown {
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

impl Runtime {
    pub async fn start(mut config: RuntimeConfig) -> Result<Self> {
        let mut providers = BTreeMap::new();
        for provider in std::mem::take(&mut config.embedded_providers) {
            let provider = embedded::Active::prepare(provider)?;
            let source = provider.definition.id.clone();
            if providers.insert(source.clone(), provider).is_some() {
                return Err(crate::runtime::Error::Operation(format!("duplicate embedded provider `{source}`")));
            }
        }

        let mut runtime = Self::from_parts(RuntimeHandle::boot(config, BTreeMap::new(), true).await?);
        runtime.embedded = providers;
        let plugins_started = std::time::Instant::now();
        let outcome = runtime.install_plugins().await;
        tracing::info!(phase = "plugins", elapsed_ms = plugins_started.elapsed().as_millis() as u64, "plugin installation complete");
        if let Err(error) = outcome {
            return Err(runtime.rollback(error).await);
        }
        Ok(runtime)
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

    async fn with_execution_fixtures_inner(
        config: RuntimeConfig, fixtures: impl IntoIterator<Item = (String, Arc<dyn ExecutionFixture>)>, install_builtins: bool,
    ) -> Result<Self> {
        let fixtures = fixtures.into_iter().collect();
        let mut runtime = Self::from_parts(RuntimeHandle::boot(config, fixtures, install_builtins).await?);
        let plugins_started = std::time::Instant::now();
        let outcome = runtime.install_plugins().await;
        tracing::info!(phase = "plugins", elapsed_ms = plugins_started.elapsed().as_millis() as u64, "plugin installation complete");
        if let Err(error) = outcome {
            return Err(runtime.rollback(error).await);
        }
        Ok(runtime)
    }

    async fn rollback(mut self, source: crate::runtime::Error) -> crate::runtime::Error {
        match self.shutdown().await {
            Ok(_) => source,
            Err(cleanup) => crate::runtime::Error::StartupCleanup {
                source: Box::new(source),
                cleanup: Box::new(cleanup),
                rollback: rollback::Receipt::new(self),
            },
        }
    }

    fn from_parts((harmos, handle): (HarmosRuntime, RuntimeHandle)) -> Self {
        let (publications, ingest) = sidecars::spawn_publication_ingest(handle.clone());
        assert!(handle.publication_ingest.set(publications.clone()).is_ok(), "publication ingest initializes once");
        let forwarding = sidecars::spawn_record_forwarding(handle.clone());
        handle.record_ingress.attach(handle.clone());
        let lease_gaps = sidecars::spawn_lease_gap_watch(handle.clone(), harmos.operations.watch_residents());
        Self {
            handle,
            harmos,
            background: Vec::new(),
            background_tasks: vec![ingest, forwarding, lease_gaps],
            publications,
            plugins: BTreeMap::new(),
            embedded: BTreeMap::new(),
            pending_plugins: plugins::StagedPackages::default(),
            plugin_manager: None,
            shutdown: Shutdown::default(),
        }
    }

    pub fn fixture_operation_caller(&self, owner: &str) -> Result<harmos::operations::Caller> {
        let manager =
            self.plugin_manager.as_ref().ok_or_else(|| crate::runtime::Error::Operation("plugin manager is not installed".into()))?;
        let source = manager.caller(owner)?;
        self.harmos.operations.caller(&source).map_err(Into::into)
    }

    pub fn retain<T: Send + 'static>(&mut self, integration: T) {
        self.background.push(Box::new(integration));
    }

    pub fn handle(&self) -> RuntimeHandle {
        self.handle.clone()
    }

    pub async fn shutdown(&mut self) -> Result<RuntimeOverviewSnapshot> {
        if self.shutdown.complete {
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
        self.handle.shutdown_requested.send_replace(true);
        let report_ids = self
            .handle
            .state_read()
            .executions
            .values()
            .filter(|execution| execution.saved.is_none() && execution.finished_at.is_none())
            .map(|execution| execution.test_execution_id.clone())
            .collect();
        let report_inputs = self.handle.freeze_report_inputs(report_ids).await;
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

            // Cancellation Must Settle Even If Capturing The Report Boundary Failed.
            report_inputs?;
            // Cancellation Must Settle Even If Native Shutdown Admission Failed.
            shutdown_admission?;

            if let Some(manager) = &mut self.plugin_manager {
                let mut lifecycle = plugins::lifecycle::Adapter {
                    handle: &self.handle,
                    publications: &self.publications,
                    packages: &mut self.plugins,
                    embedded: &mut self.embedded,
                };
                manager.shutdown(&mut lifecycle).await?;
            }
            self.embedded.clear();
            self.plugin_manager = None;
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
            for package in self.plugins.values_mut() {
                package.release_dispatchers(&self.handle);
            }
            self.shutdown.prepared = true;
        } else {
            report_inputs?;
        }

        if !self.shutdown.stopped {
            if self.shutdown.overview.is_none() {
                let participant = self.shutdown.participant.get_or_insert_with(|| self.harmos.stop_participant());
                let stopping = self.harmos.stop();
                tokio::pin!(stopping);
                let checkpoint = async {
                    participant.stopping().await;
                    let _evidence = self.handle.evidence_admission.lock().await;
                    let _admission = self.handle.telemetry_admission.lock().await;
                    self.handle.execution_storage.checkpoint_shutdown().await?;
                    self.handle.streams.seal::<crate::telemetry_runtime::TelemetryRow>().await;
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
                self.harmos.stop().await?;
            }
            self.shutdown.stopped = true;
        }

        while let Some((name, package)) = self.plugins.iter_mut().next() {
            package
                .close()
                .await
                .map_err(|error| crate::runtime::Error::Operation(format!("failed to remove extracted plugin `{name}`: {error}",)))?;
            self.plugins.pop_first();
        }
        self.pending_plugins
            .close()
            .await
            .map_err(|error| crate::runtime::Error::Operation(format!("failed to remove staged plugin extraction: {error}",)))?;

        self.handle.local_storage.close().await?;

        self.shutdown.complete = true;
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
