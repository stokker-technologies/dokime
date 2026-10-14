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
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
// Third Party Imports

// Local Imports
use super::{ActivePackage, Extraction, Program, Running, operation};
use crate::runtime::sidecars::{PublicationIngest, flush_publications};
use crate::{Error, InstalledSidecar, Result, RuntimeHandle};

/* === Constants === */
const FENCE_TIMEOUT: Duration = Duration::from_secs(10);

/* === Macros === */

/* === Type Aliases === */
pub(in crate::runtime) type Manager =
    harmos::plugin::PluginManager<Infallible, Sidecar, crate::runtime::embedded::Instance>;
type Backend =
    harmos::plugin::PluginBackend<Infallible, Sidecar, crate::runtime::embedded::Instance>;

/* === Structs === */
pub(in crate::runtime) struct Adapter<'a> {
    pub(in crate::runtime) handle: &'a RuntimeHandle,
    pub(in crate::runtime) publications: &'a PublicationIngest,
    pub(in crate::runtime) packages: &'a mut BTreeMap<String, ActivePackage>,
    pub(in crate::runtime) embedded: &'a mut BTreeMap<String, crate::runtime::embedded::Active>,
}

pub(in crate::runtime) struct Sidecar {
    services: harmos::operations::OperationManager,
    publications: PublicationIngest,
    program: Program,
    package: String,
    definition: harmos::plugin::PluginDefinition,
    registered: bool,
    publication_core: Arc<crate::runtime::sidecars::core::Publications>,
    publication_registered: bool,
    lease: Option<crate::services::Lease>,
    operations: Option<harmos::operations::PluginRegistration>,
    scope: Option<harmos::plugin::PluginScope>,
    sdk: Option<harmos::operations::PluginSdkRegistration>,
    initializing: Option<harmos::operations::EncodedInvocation>,
    shutting_down: Option<harmos::operations::EncodedInvocation>,
    initialized: bool,
    shutdown_attempted: bool,
    // Shared across the package's programs, but owned rather than a path lookup.
    directory: Arc<Extraction>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::plugin::PluginLifecycle for Adapter<'_> {
    type Guest = Infallible;
    type Sidecar = Sidecar;
    type Embedded = crate::runtime::embedded::Instance;

    fn preflight(&mut self, candidate: &harmos::plugin::PluginDefinitions) -> harmos::Result<()> {
        self.handle.operations.validate_requirements(candidate)
    }

    fn prepare(&mut self, artifact: &harmos::plugin::PluginDefinition) -> harmos::Result<Backend> {
        self.prepare_artifact(artifact).map_err(lifecycle_error)
    }

    async fn fence(
        &mut self,
        transition: harmos::plugin::PluginTransition,
        affected: &[harmos::plugin::PluginDefinition],
    ) -> harmos::Result<()> {
        self.fence_work(transition, affected)
            .await
            .map_err(lifecycle_error)
    }

    async fn start(
        &mut self,
        artifact: &harmos::plugin::PluginDefinition,
        router: &harmos::plugin::Router,
        authority: &harmos::plugin::PluginLifecycleAuthority<'_>,
        backend: &mut Backend,
    ) -> harmos::Result<()> {
        if authority.owner() != artifact.id
            || authority.backend_kind() != artifact.kind()
            || authority.phase() != harmos::plugin::PluginLifecyclePhase::Initialize
        {
            return Err(lifecycle_error(operation(
                "plugin initialization authority differs",
            )));
        }
        self.start_artifact(artifact, router, authority, backend)
            .await
            .map_err(lifecycle_error)
    }

    async fn stop(
        artifact: &harmos::plugin::PluginDefinition,
        authority: &harmos::plugin::PluginLifecycleAuthority<'_>,
        backend: &mut Backend,
    ) -> harmos::Result<()> {
        if authority.owner() != artifact.id
            || authority.backend_kind() != artifact.kind()
            || authority.phase() != harmos::plugin::PluginLifecyclePhase::Shutdown
        {
            return Err(lifecycle_error(operation(
                "plugin shutdown authority differs",
            )));
        }
        if let Backend::Sidecar(sidecar) = backend
            && sidecar.scope.as_ref().is_some_and(|scope| {
                !authority.matches_scope(
                    scope,
                    artifact.kind(),
                    harmos::plugin::PluginLifecyclePhase::Shutdown,
                )
            })
        {
            return Err(lifecycle_error(operation(
                "plugin shutdown generation differs",
            )));
        }
        let publications = match backend {
            Backend::Guest(guest) => match *guest {},
            Backend::Embedded(provider) => {
                provider.stop(authority).await.map_err(lifecycle_error)?;
                &provider.publications
            }
            Backend::Sidecar(sidecar) => {
                sidecar.stop(authority).await.map_err(lifecycle_error)?;
                &sidecar.publications
            }
        };
        flush_publications(publications)
            .await
            .map_err(lifecycle_error)
    }
}

impl Adapter<'_> {
    async fn fence_work(
        &mut self,
        transition: harmos::plugin::PluginTransition,
        affected: &[harmos::plugin::PluginDefinition],
    ) -> Result<()> {
        let sources: BTreeSet<_> = affected
            .iter()
            .map(|artifact| artifact.id.clone())
            .collect();
        let cycles: BTreeSet<_> = {
            let _admission = self.handle.plugin_admission.lock().await;
            self.handle
                .unavailable_plugins
                .write()
                .expect("plugin availability lock poisoned")
                .extend(sources.clone());
            self.handle
                .provider_tasks
                .lock()
                .expect("provider task lock poisoned")
                .iter()
                .filter(|task| sources.contains(&task.source) && !task.task.is_finished())
                .map(|task| {
                    if transition == harmos::plugin::PluginTransition::Shutdown {
                        task.cancel_queued();
                    }
                    task.cycle.clone()
                })
                .collect()
        };
        for cycle in cycles {
            if transition != harmos::plugin::PluginTransition::Shutdown {
                break;
            }
            let lifecycle = self.handle.cycle_lifecycle(&cycle).await;
            if matches!(
                lifecycle,
                Some(
                    crate::execution::CycleStatus::Running | crate::execution::CycleStatus::Paused
                )
            ) && let Err(error) = self
                .handle
                .stop_cycle(cycle.clone(), crate::CommandContext::default())
                .await
            {
                // A concurrent normal completion may win before Stop commits.
                if matches!(
                    self.handle.cycle_lifecycle(&cycle).await,
                    Some(
                        crate::execution::CycleStatus::Running
                            | crate::execution::CycleStatus::Paused
                    )
                ) {
                    return Err(error);
                }
            }
        }
        // Keep every monitor registered until it has finished. Dropping a reload
        // future or failing cancellation must never detach shutdown-owned work.
        let deadline = tokio::time::Instant::now() + FENCE_TIMEOUT;
        loop {
            let finished = self
                .handle
                .provider_tasks
                .lock()
                .expect("provider task lock poisoned")
                .iter()
                .filter(|task| sources.contains(&task.source))
                .all(|task| task.task.is_finished());
            if finished {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(Error::timeout(
                    "plugin_drain_timeout",
                    "affected executions did not settle within the plugin drain bound; existing owners remain registered",
                ));
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        self.handle.reconcile_listener_record_demand()?;

        let tasks = {
            let mut tasks = self
                .handle
                .provider_tasks
                .lock()
                .expect("provider task lock poisoned");
            let (owned, remaining): (Vec<_>, Vec<_>) = std::mem::take(&mut *tasks)
                .into_iter()
                .partition(|task| sources.contains(&task.source));
            *tasks = remaining;
            owned
        };
        let mut completion_error = None;
        for task in tasks {
            if let Err(error) = task.task.await {
                completion_error.get_or_insert(Error::Join { source: error });
            }
        }
        if let Some(error) = completion_error {
            return Err(error);
        }
        // A normal reload must not replace an owner whose admitted requests
        // failed publication. Root shutdown still releases resources and
        // reports this retained diagnostic after physical cleanup.
        if !self.handle.is_shutting_down() {
            self.handle.provider_completion_result()?;
        }
        // Revoke Domain Exposure After Admitted Work Has Settled.
        // Physical Shutdown Is Retained By Each Backend, Independent Of This Adapter.
        for source in &sources {
            self.handle.remove_record_demand_source(source);
            if let Some(provider) = self.embedded.get_mut(source) {
                provider.release_observers();
            }
            for package in self.packages.values_mut() {
                package.telemetry.remove(source);
                package.running.retain(|live| &live.source != source);
            }
            self.handle
                .remove_action_dispatcher(&dokime::Identifier::new(source).map_err(operation)?);
        }
        Ok(())
    }

    fn prepare_artifact(&self, artifact: &harmos::plugin::PluginDefinition) -> Result<Backend> {
        if let Some(provider) = self.embedded.get(&artifact.id) {
            return Ok(Backend::Embedded(
                provider.instance(self.publications.clone()),
            ));
        }

        let (name, package, program) = self
            .packages
            .iter()
            .find_map(|(name, package)| {
                package
                    .package
                    .programs
                    .iter()
                    .find(|program| program.executable.id() == artifact.id)
                    .map(|program| (name, &package.package, program))
            })
            .ok_or_else(|| operation(format!("no prepared plugin owns {}", artifact.id)))?;

        if program.definition.artifact != artifact.artifact {
            return Err(operation(
                "prepared executable differs from the admitted artifact descriptor",
            ));
        }

        Ok(Backend::Sidecar(Sidecar {
            services: self.handle.operations.clone(),
            publications: self.publications.clone(),
            program: program.clone(),
            package: name.clone(),
            definition: artifact.clone(),
            registered: false,
            publication_core: self.handle.publication_core.clone(),
            publication_registered: false,
            lease: None,
            operations: None,
            scope: None,
            sdk: None,
            initializing: None,
            shutting_down: None,
            initialized: false,
            shutdown_attempted: false,
            directory: Arc::clone(&package._directory),
        }))
    }

    async fn start_artifact(
        &mut self,
        artifact: &harmos::plugin::PluginDefinition,
        router: &harmos::plugin::Router,
        authority: &harmos::plugin::PluginLifecycleAuthority<'_>,
        backend: &mut Backend,
    ) -> Result<()> {
        let sidecar = match backend {
            Backend::Guest(guest) => match *guest {},
            Backend::Embedded(provider) => {
                return provider.start(self.handle, router, authority).await;
            }
            Backend::Sidecar(sidecar) => sidecar,
        };
        let program = &sidecar.program;
        let bindings = program.bindings.clone();
        sidecar.start(self.handle, router, authority).await?;
        self.handle.capture_suite_objects(&artifact.id).await?;
        self.packages
            .get_mut(&sidecar.package)
            .expect("the prepared package is retained")
            .running
            .push(Running {
                source: artifact.id.clone(),
                exposed: false,
                bindings,
            });
        Ok(())
    }
}

impl Sidecar {
    async fn start(
        &mut self,
        handle: &RuntimeHandle,
        router: &harmos::plugin::Router,
        authority: &harmos::plugin::PluginLifecycleAuthority<'_>,
    ) -> Result<()> {
        let started = std::time::Instant::now();
        let source = self.program.executable.id().to_owned();
        let outcome = self.start_attempt(handle, router, authority).await;
        tracing::info!(
            plugin = %source,
            elapsed_ms = started.elapsed().as_millis() as u64,
            ok = outcome.is_ok(),
            "plugin start attempt complete"
        );
        outcome
    }

    async fn start_attempt(
        &mut self,
        handle: &RuntimeHandle,
        router: &harmos::plugin::Router,
        authority: &harmos::plugin::PluginLifecycleAuthority<'_>,
    ) -> Result<()> {
        let program = &self.program;
        let executable = &program.executable.0;
        let source = program.executable.id().to_owned();
        let logged_by = source.clone();
        let sidecar: InstalledSidecar = harmos::plugin::sidecar::Supervised::new(
            executable.clone(),
            Box::new(
                move |_context: &harmos::services::Service<crate::Dokime>, value| {
                    if let harmos::plugin::sidecar::Notice::Logged { line } = &value {
                        forward_plugin_line(&logged_by, line);
                    }
                },
            ) as crate::SidecarNoticeHandler,
        )
        .demand_keys(program.plugin.record_contracts.iter().map(|contract| {
            crate::runtime::sidecars::record_demand_key(contract.record_type.as_str())
        }))
        .routed(router)
        .expected_registrations(&self.definition.contributions.registrations);
        self.publication_core.insert(&source, sidecar.clone())?;
        self.publication_registered = true;
        let serving = sidecar.clone();
        let record_demands = sidecar.demands();
        let ready = sidecar.ready();
        let policy = executable
            .declaration()
            .restart()
            .ok_or_else(|| operation("sidecar declaration has no authored restart policy"))?
            .bounded();
        // A declaration without restart credit deactivates on its first fault.
        let restart = if policy.enabled() {
            harmos::operations::RestartPolicy::OnTransient {
                credit: usize::try_from(policy.credit).map_err(operation)?,
                sustained_health: policy.sustained_health(),
                backoff: harmos::operations::Backoff::Exponential {
                    initial: policy.backoff_base(),
                    maximum: policy.backoff_cap(),
                    jitter: harmos::operations::Jitter::Deterministic,
                },
            }
        } else {
            harmos::operations::RestartPolicy::Never
        };
        self.services
            .register_service(
                harmos::services::ServiceDefinition {
                    id: source.clone(),
                    policy: harmos::operations::OperationPolicy {
                        restart,
                        ..harmos::services::POLICY
                    },
                },
                move |_: Arc<()>| sidecar.clone(),
            )
            .map_err(Error::from)?;
        self.registered = true;
        self.lease = Some(self.services.submit_resident(&source).await?);
        let native = Arc::new(
            tokio::time::timeout(Duration::from_secs(10), serving.staged_operations())
                .await
                .map_err(|_| Error::PluginInitialization {
                    plugin: source.clone(),
                    reason: "native endpoint did not stage within 10 seconds".to_owned(),
                })?
                .map_err(operation)?,
        );
        let scope = native.scope().ok_or_else(|| {
            operation("initialized native endpoint has no admitted plugin generation")
        })?;
        if !authority.matches_scope(
            scope,
            self.definition.kind(),
            harmos::plugin::PluginLifecyclePhase::Initialize,
        ) {
            return Err(operation("native initialization generation differs"));
        }
        self.scope = Some(scope.clone());
        let capacity = NonZeroUsize::new(128).expect("native invocation capacity is positive");
        self.operations = Some(handle.operations.stage_plugin(scope, |bindings| {
            bindings.register_native(crate::app::Lanes::Open, Arc::clone(&native), capacity)
        })?);
        self.sdk = Some(native.stage_sdk(&handle.operations, crate::app::Lanes::Open)?);
        self.initializing = Some(self.sdk.as_ref().expect("SDK staged").submit_initialize(
            authority,
            &self.definition.configuration,
            NonZeroUsize::new(4 * 1024 * 1024).expect("bounded initialization bytes"),
        )?);
        let outcome = self
            .initializing
            .as_mut()
            .expect("initialization retained")
            .await;
        self.initializing.take();
        outcome?;
        self.initialized = true;
        handle.register_record_demand_source(source, record_demands, ready);
        Ok(())
    }

    async fn stop(
        &mut self,
        authority: &harmos::plugin::PluginLifecycleAuthority<'_>,
    ) -> Result<()> {
        if let Some(sdk) = &self.sdk {
            sdk.settle().await?;
            if let Some(initializing) = &mut self.initializing {
                let outcome = initializing.await;
                self.initializing.take();
                outcome?;
                self.initialized = true;
            }
            if self.initialized && !self.shutdown_attempted {
                self.shutting_down = Some(sdk.submit_shutdown(authority)?);
                self.shutdown_attempted = true;
            }
            if let Some(shutdown) = &mut self.shutting_down {
                let outcome = shutdown.await;
                self.shutting_down.take();
                outcome?;
                self.initialized = false;
            }
        }
        if let Some(operations) = self.operations.as_mut() {
            operations.shutdown().await?;
        }
        self.operations = None;
        if self.registered {
            let source = self.program.executable.id();
            let stopped = self.services.unregister_resident(source).await;
            // Settlement errors can still complete administrative revocation.
            // Cancellation before return leaves this stage owned for retry.
            self.registered = self.services.resident_definition(source).is_some();
            stopped?;
        }
        self.lease = None;
        if let Some(sdk) = &mut self.sdk {
            sdk.close().await?;
        }
        self.sdk.take();
        if self.publication_registered {
            self.publication_core.remove(self.program.executable.id());
            self.publication_registered = false;
        }
        self.scope = None;

        // Replaced packages release their preparation reference. The last old
        // generation owns deletion and keeps its receipt when deletion fails.
        if let Some(directory) = Arc::get_mut(&mut self.directory) {
            directory.close().await.map_err(operation)?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn lifecycle_error(error: Error) -> harmos::Error {
    match error {
        Error::Harmos(error) => error,
        error => harmos::Error::TaskFailed {
            domain: "dokime plugin lifecycle",
            source: Arc::new(error),
        },
    }
}

fn forward_plugin_line(plugin: &str, line: &str) {
    macro_rules! forward {
        ($level:expr, $message:expr) => {
            tracing::event!(
                target: "sidecar::stderr",
                $level,
                plugin = %plugin,
                line = %$message,
                "sidecar stderr"
            )
        };
    }

    match severity(line) {
        Some(("ERROR", message)) => forward!(tracing::Level::ERROR, message),
        Some(("WARN", message)) => forward!(tracing::Level::WARN, message),
        Some(("DEBUG", message)) => forward!(tracing::Level::DEBUG, message),
        Some(("TRACE", message)) => forward!(tracing::Level::TRACE, message),
        Some((_, message)) => forward!(tracing::Level::INFO, message),
        None => forward!(tracing::Level::INFO, line),
    }
}

fn severity(line: &str) -> Option<(&'static str, &str)> {
    let (token, rest) = line.trim_start().split_once(' ')?;
    let level = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"]
        .into_iter()
        .find(|level| *level == token)?;

    Some((level, rest.trim_start()))
}

/* === Tests === */
