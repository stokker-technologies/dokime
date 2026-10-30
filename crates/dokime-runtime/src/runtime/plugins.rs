/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(super) mod lifecycle;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};

// Third Party Imports

// Local Imports
use super::{Runtime, progress::Operation};
use crate::domains::installed_plugins::contributions::decode_registrations;
use crate::plugins::{PluginBundle, PluginProvenance, PluginStore};
use crate::plugins::{
    catalog::selected_files,
    store::{digest, read_bounded},
};
use crate::{runtime::Error, runtime::Result};
use crate::{
    state::{BundleCatalog, ProviderCatalog},
    transactions::{ReconcileProviders, ReloadProviderCatalog},
};

/* === Constants === */

const PLUGIN_LOG_VARIABLE: &str = "DOKIME_PLUGIN_LOG";

// Extraction Cache
const EXTRACTION_MARKER_FILE: &str = ".extraction-marker.json";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Package {
    input: PluginBundle,
    catalog: BundleCatalog,
    programs: Vec<Program>,
    _directory: Arc<Extraction>,
}

#[derive(Default)]
pub(super) struct StagedPackages(Vec<Package>);

#[derive(Clone)]
struct Program {
    plugin: crate::InstalledPlugin,
    executable: Executable,
    definition: harmos::plugin::PluginDefinition,
    bindings: crate::plugins::dispatch::Bindings,
    configured: serde_json::Value,
}

#[derive(Clone)]
struct Executable(harmos::sidecar::Executable);

struct Running {
    source: String,
    exposed: bool,
    bindings: crate::plugins::dispatch::Bindings,
}

pub(super) struct ActivePackage {
    running: Vec<Running>,
    // Keep the extraction directory after every handle that can retain a
    // sidecar transport. On Windows, a TempDir cannot be removed while one of
    // those handles is still alive; declaration order is also drop order.
    package: Package,
}

struct ReadBundle {
    input: PluginBundle,
    bytes: Vec<u8>,
    content_digest: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct ExtractionMarker {
    sha256: String,
    size: u64,
    mtime_unix_ms: i64,
}

/* === Enums === */

enum Extraction {
    Ephemeral(tempfile::TempDir),
    Cached(std::path::PathBuf),
    Closed,
}

/* === Traits === */

/* === Implementations === */

impl Extraction {
    fn path(&self) -> &std::path::Path {
        match self {
            Self::Ephemeral(directory) => directory.path(),
            Self::Cached(path) => path,
            Self::Closed => unreachable!("closed extraction cannot prepare a plugin"),
        }
    }
}

impl Package {
    async fn close(&mut self) -> std::io::Result<()> {
        self.programs.clear();
        let directory = Arc::get_mut(&mut self._directory)
            .ok_or_else(|| std::io::Error::other("plugin backend still owns the extraction directory"))?;
        directory.close().await
    }
}

impl Extraction {
    async fn close(&mut self) -> std::io::Result<()> {
        let Self::Ephemeral(directory) = self else {
            return Ok(());
        };
        let path = directory.path().to_owned();
        let result = remove_extraction(&path).await;
        if result.is_ok() {
            *self = Self::Closed;
        }
        result
    }
}

impl StagedPackages {
    fn extend(&mut self, packages: Vec<Package>) {
        self.0.extend(packages);
    }

    pub(super) async fn close(&mut self) -> std::io::Result<()> {
        while let Some(package) = self.0.last_mut() {
            package.close().await?;
            self.0.pop();
        }
        Ok(())
    }
}

impl Executable {
    fn id(&self) -> &str {
        &self.0.declaration().id
    }
}

impl ActivePackage {
    pub(super) fn release_dispatchers(&mut self, handle: &super::RuntimeHandle) {
        for live in &self.running {
            if let Ok(id) = dokime_domain::model::identity::Identifier::new(&live.source) {
                handle.remove_action_dispatcher(&id);
            }
        }
        self.running.clear();
    }

    pub(super) async fn close(&mut self) -> std::io::Result<()> {
        self.running.clear();
        self.package.close().await
    }
}

impl Runtime {
    pub(super) async fn install_plugins(&mut self) -> Result<()> {
        let progress = self.handle.operation_progress().begin("Installing Plugins");
        let installed = self.install_plugins_with(&progress).await;
        progress.track(installed)
    }

    async fn install_plugins_with(&mut self, progress: &Operation) -> Result<()> {
        let config = self.handle.config();
        let store = self.handle.plugin_store();
        store.sweep().await.map_err(operation)?;
        let mut inputs = config.plugins.clone();
        if let Some(directory) = &config.plugin_lookup {
            inputs.extend(lookup_bundles(directory)?);
        }
        if let Some(directory) = &config.bundled_plugins {
            let configured = inputs
                .iter()
                .map(|input| {
                    let bytes = read_bounded(&input.path, store.archive_bytes())?;
                    crate::plugins::inspect_with_archive_limit(&bytes, &config.schema, store.archive_bytes()).map(|(_, metadata)| metadata)
                })
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(operation)?;
            let host_sources = self.embedded.keys().cloned().collect();
            store.install_distribution(directory, &configured, &host_sources).await.map_err(operation)?;
        }
        inputs.extend(store.installed().await.map_err(operation)?);
        for input in &inputs {
            if let PluginProvenance::Installed { install_id, .. } = input.provenance {
                self.handle.plugin_boot.write().expect("plugin boot lock poisoned").insert(install_id, false);
            }
        }
        let extraction = extraction_root(&config);
        std::fs::create_dir_all(&extraction).map_err(operation)?;
        sweep_orphaned_extractions(&extraction, &inputs);
        progress.phase("Preparing Bundles");
        let total = inputs.len() as u64;
        let mut packages = Vec::new();
        for (prepared, input) in inputs.into_iter().enumerate() {
            progress.progress(prepared as u64, total);
            let reuse = reusable_extraction(&extraction, &input);
            let cached_digest = reuse.as_ref().and_then(|_| match &input.provenance {
                PluginProvenance::Installed { sha256, .. } => Some(sha256.clone()),
                PluginProvenance::Configured => None,
            });
            let prepared = match read_bundle(
                input.clone(),
                cached_digest.as_deref(),
                usize::try_from(dokime_domain::model::plugins::wire::INSTALL_ARCHIVE_MAX_BYTES)
                    .expect("validated server plugin bundle limit fits native address space"),
            ) {
                Ok(bundle) => prepare(bundle, &extraction, reuse, &self.handle).await,
                Err(error) => Err(error),
            };
            match prepared {
                Ok(package) => packages.push(package),
                Err(error) => {
                    if let Err(failure) = failed(&store, &input, &error.to_string()).await {
                        self.pending_plugins.extend(packages);
                        return Err(failure);
                    }
                }
            }
        }
        progress.progress(total, total);
        progress.phase("Validating Catalogs");
        progress.indeterminate();
        // Program values live under their bundle's install ID; bind the
        // sources before any program is activated with them.
        self.handle.plugin_preferences().bind_installs(
            packages
                .iter()
                .filter_map(|package| match package.input.provenance {
                    PluginProvenance::Installed { install_id, .. } => Some((package, install_id)),
                    PluginProvenance::Configured => None,
                })
                .flat_map(|(package, install_id)| {
                    package.programs.iter().map(move |program| (program.executable.id().to_owned(), install_id))
                })
                .collect(),
        );
        let embedded_plugins: Vec<_> = self.embedded.values().map(|provider| provider.definition.clone()).collect();
        let embedded_catalogs: Vec<_> = self.embedded.values().map(|provider| provider.catalog.clone()).collect();
        let mut removed = match plan(&store, &packages.iter().collect::<Vec<_>>(), &embedded_plugins, &self.handle.operations).await {
            Ok(plan) => plan,
            Err(error) => {
                self.pending_plugins.extend(packages);
                return Err(error);
            }
        };
        // Validate the entire desired catalog before any process runs. An
        // installed package that conflicts is pruned together with dependents.
        loop {
            let catalogs: Vec<ProviderCatalog> = embedded_catalogs
                .iter()
                .cloned()
                .chain(
                    packages
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| !removed.contains(index))
                        .map(|(_, package)| package.catalog.clone().into()),
                )
                .collect();
            if self.handle.state_read().validate_providers(&catalogs).is_ok() {
                break;
            }
            let mut accepted = embedded_catalogs.clone();
            let mut refused = None;
            for (index, package) in packages.iter().enumerate().filter(|(index, _)| !removed.contains(index)) {
                accepted.push(package.catalog.clone().into());
                if let Err(error) = self.handle.state_read().as_ref().validate_providers(&accepted) {
                    refused = Some((index, error));
                    break;
                }
            }
            let Some((index, reason)) = refused else {
                break;
            };
            if let Err(error) = prune(
                &store,
                &packages.iter().collect::<Vec<_>>(),
                &mut removed,
                index,
                &reason,
                &embedded_plugins,
                &self.handle.operations,
            )
            .await
            {
                self.pending_plugins.extend(packages);
                return Err(error);
            }
        }
        let catalogs: Vec<ProviderCatalog> = embedded_catalogs
            .iter()
            .cloned()
            .chain(
                packages
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !removed.contains(index))
                    .map(|(_, package)| package.catalog.clone().into()),
            )
            .collect();
        self.handle.state_read().validate_providers(&catalogs).map_err(operation)?;
        for provider in self.embedded.values() {
            self.handle.streams.validate_publications(&provider.definition.id, provider.streams.clone()).map_err(operation)?;
        }

        for (index, package) in packages.into_iter().enumerate() {
            if removed.contains(&index) {
                self.pending_plugins.extend(vec![package]);
            } else {
                self.plugins.insert(package.catalog.package.clone(), ActivePackage { package, running: Vec::new() });
            }
        }
        let candidate = graph(self.plugins.values().flat_map(|active| &active.package.programs), self.embedded.values(), [])?;
        let manager = harmos::plugin::PluginManager::new(candidate)?;
        self.handle.routing.set(manager.router().clone()).map_err(|_| operation("plugin manager was already configured"))?;
        self.plugin_manager = Some(manager);

        let mut excluded = BTreeSet::new();
        let mut next = None;
        loop {
            let manager = self.plugin_manager.as_mut().expect("manager prepared before activation");
            let mut lifecycle = lifecycle::Adapter {
                handle: &self.handle,
                publications: &self.publications,
                packages: &mut self.plugins,
                embedded: &mut self.embedded,
            };
            let result = match next.take() {
                Some(candidate) => manager.reload(candidate, &mut lifecycle).await,
                None => manager.initialize(&mut lifecycle).await,
            };
            match result {
                Ok(()) => break,
                Err(error) => {
                    let Some((plugin, source, cleanup)) = start_failure(&error) else {
                        return Err(error.into());
                    };
                    if self.embedded.contains_key(plugin) || !cleanup.is_empty() {
                        return Err(error.into());
                    }
                    let packages: Vec<_> = self.plugins.values().map(|active| &active.package).collect();
                    let initial = packages
                        .iter()
                        .position(|package| package.programs.iter().any(|program| program.executable.id() == plugin))
                        .expect("failed plugin belongs to a staged package");
                    let mut removed = packages
                        .iter()
                        .enumerate()
                        .filter_map(|(index, package)| excluded.contains(&package.catalog.package).then_some(index))
                        .collect();
                    prune(
                        &self.handle.plugin_store(),
                        &packages,
                        &mut removed,
                        initial,
                        &source.to_string(),
                        &embedded_plugins,
                        &self.handle.operations,
                    )
                    .await?;
                    excluded.extend(removed.iter().map(|index| packages[*index].catalog.package.clone()));
                    next = Some(graph(
                        self.plugins.iter().filter(|(name, _)| !excluded.contains(*name)).flat_map(|(_, active)| &active.package.programs),
                        self.embedded.values(),
                        [],
                    )?);
                }
            }
        }
        for name in excluded {
            let active = self.plugins.remove(&name).expect("excluded package is staged");
            self.pending_plugins.extend(vec![active.package]);
        }
        let admitted: Vec<_> = self.plugins.values().map(|active| active.package.catalog.clone()).collect();
        let providers: Vec<_> = admitted.into_iter().map(Into::into).chain(embedded_catalogs).collect();
        let state = self.handle.state_read();
        let has_previous_providers = state.plugin_owners.values().any(|owner| !matches!(owner, crate::state::PluginOwner::Host));
        // An unchanged catalog set is not journaled again; otherwise every
        // boot appends every catalog in full to the journal and its tail.
        let reconcile = (!providers.is_empty() || has_previous_providers) && !state.providers_current(&providers);
        drop(state);
        if reconcile && let Err(error) = self.handle.commit(ReconcileProviders { providers }).await {
            return Err(operation(error));
        }
        self.capture_ready_catalogs().await?;
        self.handle.restore_effects(None).await?;
        self.recover_plugin_executions().await?;
        let inputs: Vec<_> = self.plugins.values().map(|active| active.package.input.clone()).collect();
        for input in inputs {
            if let PluginProvenance::Installed { install_id, .. } = input.provenance {
                self.handle.plugin_boot.write().expect("plugin boot lock poisoned").insert(install_id, true);
            }
            record(&store, &input, None).await;
        }
        self.expose_plugins()?;
        Ok(())
    }

    fn ready_catalogs(&self) -> Vec<ProviderCatalog> {
        let ready: BTreeSet<_> = self.plugin_manager.as_ref().expect("manager is initialized").ready().collect();
        self.plugins
            .values()
            .filter_map(|active| {
                let mut bundle = active.package.catalog.clone();
                bundle.plugins.retain(|plugin| ready.contains(plugin.source.as_str()));
                (!bundle.plugins.is_empty()).then_some(bundle.into())
            })
            .chain(
                self.embedded
                    .values()
                    .filter(|provider| ready.contains(provider.definition.id.as_str()))
                    .map(|provider| provider.catalog.clone()),
            )
            .collect()
    }

    async fn capture_ready_catalogs(&mut self) -> Result<()> {
        let ready: Vec<_> = self.plugin_manager.as_ref().expect("manager is initialized").ready().map(str::to_owned).collect();
        // Catalog Reconciliation Must Precede Schema Capture And Recovery.
        let contracts = self.handle.state_read().telemetry_contracts();
        self.handle.execution_storage.register_schemas(&contracts).await?;
        for source in ready {
            self.handle.capture_suite_objects(&source).await?;
        }
        self.handle.storage().flush().await?;
        Ok(())
    }

    fn expose_plugins(&mut self) -> Result<()> {
        let ready: BTreeSet<_> = self.plugin_manager.as_ref().expect("manager is initialized").ready().map(str::to_owned).collect();
        for active in self.plugins.values_mut() {
            for live in &mut active.running {
                if !ready.contains(&live.source) || live.exposed {
                    continue;
                }
                let id = crate::Identifier::new(&live.source).map_err(operation)?;
                let dispatcher = Arc::new(live.bindings.clone().attach(&live.source, self.handle.operations.clone()));
                self.handle.set_action_dispatcher(id.clone(), Arc::clone(&dispatcher) as _);
                let handle = self.handle.clone();
                tokio::spawn(async move {
                    handle.discover_plugin_settings(&id, &dispatcher).await;
                });
                live.exposed = true;
                self.handle.engage_owned_demand(&live.source);
            }
        }
        for (source, provider) in &mut self.embedded {
            if ready.contains(source) {
                provider.expose(&self.handle);
            }
        }
        self.handle.unavailable_plugins.write().expect("plugin availability lock poisoned").retain(|source| !ready.contains(source));
        self.handle.reconcile_listener_record_demand()?;
        Ok(())
    }

    pub async fn register(&mut self, provider: Arc<dyn super::EmbeddedProvider>) -> Result<()> {
        let candidate = super::embedded::Active::prepare(provider, self.handle.schema_settings())?;
        let source = candidate.definition.id.clone();
        let catalogs: Vec<ProviderCatalog> = self
            .plugins
            .values()
            .map(|active| active.package.catalog.clone().into())
            .chain(self.embedded.iter().filter(|(id, _)| *id != &source).map(|(_, active)| active.catalog.clone()))
            .chain(std::iter::once(candidate.catalog.clone()))
            .collect();
        self.handle.state_read().validate_provider_reload(&catalogs, &source).map_err(operation)?;
        self.handle.streams.validate_publications(&source, candidate.streams.clone()).map_err(operation)?;
        let definitions = graph(
            self.plugins.values().flat_map(|active| &active.package.programs),
            self.embedded.iter().filter(|(id, _)| *id != &source).map(|(_, active)| active).chain(std::iter::once(&candidate)),
            [source.clone()],
        )?;
        let reload = self.plugin_manager.as_ref().expect("manager initialized at startup").plan_reload(definitions)?;

        self.embedded.insert(source.clone(), candidate);
        let mut lifecycle = lifecycle::Adapter {
            handle: &self.handle,
            publications: &self.publications,
            packages: &mut self.plugins,
            embedded: &mut self.embedded,
        };
        let manager = self.plugin_manager.as_mut().expect("manager initialized at startup");
        if let Err(error) = manager.apply_reload(reload, &mut lifecycle).await {
            if start_failure(&error).is_some() {
                self.handle.commit(ReconcileProviders { providers: self.ready_catalogs() }).await.map_err(operation)?;
            }
            return Err(error.into());
        }

        self.handle.commit(ReloadProviderCatalog { providers: catalogs, provider: source }).await.map_err(operation)?;
        self.capture_ready_catalogs().await?;
        self.expose_plugins()?;
        Ok(())
    }

    pub async fn reinitialize_plugin(&mut self, plugin: &str) -> Result<()> {
        let owned = self.plugins.values().flat_map(|active| &active.package.programs).any(|program| program.executable.id() == plugin)
            || self.embedded.contains_key(plugin);
        if !owned {
            return Err(operation(format!("no admitted plugin owns `{plugin}`")));
        }

        let mut refreshed =
            self.plugins.values().flat_map(|active| &active.package.programs).find(|program| program.executable.id() == plugin).cloned();
        if let Some(program) = &mut refreshed {
            let initialization = self.handle.plugin_initialization(&program.plugin, &program.configured).await?;
            program.definition.configuration = serde_json::to_vec(&initialization).map_err(operation)?;
        }
        let programs =
            self.plugins.values().flat_map(|active| &active.package.programs).map(|program| {
                refreshed.as_ref().filter(|candidate| candidate.executable.id() == program.executable.id()).unwrap_or(program)
            });
        let definitions = graph(programs, self.embedded.values(), [plugin.to_owned()])?;
        let reload = self.plugin_manager.as_ref().expect("manager initialized at startup").plan_reload(definitions)?;

        // Desired preparation changes only after complete descriptor preflight.
        // Existing physical backends retain their original admitted bytes.
        if let Some(refreshed) = refreshed {
            let program = self
                .plugins
                .values_mut()
                .flat_map(|active| &mut active.package.programs)
                .find(|program| program.executable.id() == plugin)
                .expect("requested program remains prepared");
            program.definition = refreshed.definition;
        }

        let mut lifecycle = lifecycle::Adapter {
            handle: &self.handle,
            publications: &self.publications,
            packages: &mut self.plugins,
            embedded: &mut self.embedded,
        };
        let manager = self.plugin_manager.as_mut().expect("manager initialized at startup");
        manager.apply_reload(reload, &mut lifecycle).await.map_err(Error::from)?;

        self.expose_plugins()
    }

    pub async fn reload_plugin(&mut self, input: PluginBundle) -> Result<()> {
        let extraction = extraction_root(&self.handle.config());
        // Development reload always re-reads and re-extracts: it exists
        // specifically to pick up bytes that just changed. The extraction
        // cache only ever applies to `Installed` bundles (see
        // `reusable_extraction`), never to this path.
        let bundle = read_bundle(
            input,
            None,
            usize::try_from(dokime_domain::model::plugins::wire::INSTALL_ARCHIVE_MAX_BYTES)
                .expect("validated server plugin bundle limit fits native address space"),
        )?;
        if !matches!(bundle.input.provenance, PluginProvenance::Configured) {
            return Err(operation("only explicitly configured development plugins can be reloaded"));
        }
        // Exact bytes already admitted for this configuration need no new archive
        // inspection or staging. The digest is computed here, never supplied by
        // the caller; unavailable plugins still take the full recovery path.
        if matches!(bundle.input.provenance, PluginProvenance::Configured)
            && self.plugins.values().any(|active| {
                matches!(active.package.input.provenance, PluginProvenance::Configured)
                    && active.package.catalog.content_digest == bundle.content_digest
                    && active.package.input.programs.len() == bundle.input.programs.len()
                    && active
                        .package
                        .input
                        .programs
                        .iter()
                        .all(|(owner, config)| bundle.input.programs.get(owner).is_some_and(|candidate| candidate == config))
                    && active.package.programs.iter().all(|program| {
                        self.handle.plugin_available(program.executable.id())
                            && self
                                .plugin_manager
                                .as_ref()
                                .is_some_and(|manager| manager.ready().any(|owner| owner == program.executable.id()))
                    })
            })
        {
            return Ok(());
        }
        let candidate = prepare(bundle, &extraction, None, &self.handle).await?;
        let name = candidate.catalog.package.clone();
        if let Some(current) = self.plugins.get(&name)
            && !matches!(current.package.input.provenance, PluginProvenance::Configured)
        {
            return Err(operation("only explicitly configured development plugins can be reloaded"));
        }
        let mut catalogs: Vec<ProviderCatalog> = self
            .plugins
            .iter()
            .filter(|(key, _)| *key != &name)
            .map(|(_, active)| active.package.catalog.clone().into())
            .chain(self.embedded.values().map(|provider| provider.catalog.clone()))
            .collect();
        catalogs.push(candidate.catalog.clone().into());
        self.handle.state_read().validate_provider_reload(&catalogs, &name).map_err(operation)?;
        let redefined: Vec<_> = {
            let state = self.handle.state_read();
            candidate
                .catalog
                .plugins
                .iter()
                .flat_map(|plugin| {
                    plugin.record_contracts.iter().chain(plugin.suites.suites.iter().flat_map(|suite| suite.record_contracts.iter()))
                })
                .filter(|contract| {
                    state
                        .historical_record_contract(&contract.record_type, contract.schema_version)
                        .is_some_and(|old| old.record_schema() != contract.record_schema())
                })
                .map(|contract| (contract.record_type.to_string(), contract.schema_version))
                .collect()
        };
        // Resolve the complete dependency graph before interrupting any resource.
        let programs: Vec<_> = self
            .plugins
            .iter()
            .filter(|(key, _)| *key != &name)
            .flat_map(|(_, active)| &active.package.programs)
            .chain(candidate.programs.iter())
            .collect();
        for program in &programs {
            self.handle
                .streams
                .validate_publications(program.executable.id(), program.executable.0.declaration().streams.clone())
                .map_err(operation)?;
        }
        let rebuilt = candidate.programs.iter().map(|program| program.executable.id().to_owned());
        let definitions = graph(programs.iter().copied(), self.embedded.values(), rebuilt)?;
        let reload = self.plugin_manager.as_ref().expect("manager initialized at startup").plan_reload(definitions)?;

        let mut desired = ActivePackage { package: candidate, running: Vec::new() };
        if let Some(previous) = self.plugins.remove(&name) {
            // Manager backends retain old physical generations and their files.
            // These entries are only the domain projections for exposure.
            desired.running = previous.running;
            drop(previous.package);
        }
        self.plugins.insert(name.clone(), desired);
        let mut lifecycle = lifecycle::Adapter {
            handle: &self.handle,
            publications: &self.publications,
            packages: &mut self.plugins,
            embedded: &mut self.embedded,
        };
        let manager = self.plugin_manager.as_mut().expect("manager initialized at startup");
        if let Err(error) = manager.apply_reload(reload, &mut lifecycle).await {
            if start_failure(&error).is_some() {
                self.handle.commit(ReconcileProviders { providers: self.ready_catalogs() }).await.map_err(operation)?;
            }
            return Err(error.into());
        }
        self.handle
            .commit(ReloadProviderCatalog {
                providers: catalogs,
                provider: name.clone(),
            })
            .await
            .map_err(operation)?;
        self.capture_ready_catalogs().await?;
        self.expose_plugins()?;
        std::mem::take(&mut self.pending_plugins).close().await.map_err(operation)?;
        for (record, version) in redefined {
            tracing::warn!(
                "development reload redefined record {record} version {version}; earlier executions in this state directory now decode against the new definition"
            );
        }
        Ok(())
    }

    async fn recover_plugin_executions(&mut self) -> Result<()> {
        // Admission is complete, but no external caller has the runtime yet.
        // Invalidate obsolete identities before unfinished recovered work is sealed as interrupted.
        let previous: std::collections::BTreeSet<_> = self
            .handle
            .state_read()
            .executions
            .values()
            .filter(|execution| {
                execution.finished_at.is_none()
                    && matches!(
                        execution.status,
                        dokime_domain::model::execution::result::ExecutionStatus::Pending
                            | dokime_domain::model::execution::result::ExecutionStatus::Running
                    )
            })
            .filter_map(|execution| execution.snapshot.plugin().cloned())
            .collect();
        for previous in previous {
            let current = self.handle.state_read().execution_identity(&previous.source);
            if current.as_ref() != Some(&previous) {
                self.handle
                    .commit(crate::transactions::InvalidateExecutions {
                        previous,
                        current,
                        invalidated_at: crate::model::time::now_utc(),
                    })
                    .await
                    .map_err(operation)?;
            }
        }
        if !self.handle.state_read().cycles.is_empty() {
            self.handle
                .commit(crate::transactions::RecoverInterruptedCycles { at: crate::model::time::now_utc() })
                .await
                .map_err(operation)?;
        }
        let cycles = self.handle.state_read().cycles.keys().cloned().collect::<Vec<_>>();
        if cycles.is_empty() {
            return Ok(());
        }
        // One Unsealable Cycle Stays Retryable And Recorded; It Never Stops The Runtime From Booting.
        for cycle in cycles {
            let Err(error) = self.handle.seal_cycle_inner(cycle.clone(), crate::CommandContext::default()).await else { continue };
            if self.handle.is_shutting_down() {
                return Err(error);
            }
            tracing::error!(%cycle, %error, "cycle could not be sealed at startup; it stays open for a later seal");
            self.handle.record_startup_seal_failure(&cycle, &error).await;
        }
        // The Stop Left These Active; Once Sealed They Are Served From Native History Like Any Earlier Session's.
        self.handle.commit(crate::transactions::RecoverInterruptedCycles { at: crate::model::time::now_utc() }).await.map_err(operation)?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn remove_extraction(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(windows)]
    let delays = [0, 10, 25, 50, 100, 200, 400, 800];
    #[cfg(not(windows))]
    let delays = [0];

    for (attempt, delay) in delays.into_iter().enumerate() {
        if delay != 0 {
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }
        match tokio::fs::remove_dir_all(path).await {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied && attempt + 1 < delays.len() => {}
            Err(error) => return Err(error),
        }
    }
    unreachable!("the final removal attempt returns its result")
}

fn start_failure(error: &harmos::Error) -> Option<(&str, &harmos::Error, &[harmos::plugin::PluginCleanupFailure])> {
    match error {
        harmos::Error::PluginLifecycle { source } => match source.as_ref() {
            harmos::plugin::PluginFailure::Start { artifact: plugin, source, cleanup } => Some((plugin, source, cleanup)),
            _ => None,
        },
        _ => None,
    }
}

fn plugin_log_filter(filter: std::ffi::OsString) -> std::ffi::OsString {
    match filter.to_str() {
        Some(level @ ("debug" | "trace")) => format!("{level},h2=warn,tower=warn,hyper=warn,hyper_util=warn,tonic=warn").into(),
        _ => filter,
    }
}

fn graph<'a>(
    programs: impl IntoIterator<Item = &'a Program>, embedded: impl IntoIterator<Item = &'a super::embedded::Active>,
    rebuilt: impl IntoIterator<Item = String>,
) -> Result<harmos::plugin::PluginDefinitions> {
    let definitions = programs
        .into_iter()
        .map(|program| program.definition.clone())
        .chain(embedded.into_iter().map(|provider| provider.definition.clone()));
    harmos::plugin::PluginDefinitions::new(definitions, rebuilt).map_err(Error::from)
}

fn extraction_root(config: &crate::RuntimeConfig) -> PathBuf {
    config.application.cache().join("plugins")
}

fn lookup_bundles(directory: &std::path::Path) -> Result<Vec<PluginBundle>> {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(operation(error)),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry.map_err(operation)?.path();
        if path.extension().is_some_and(|extension| extension == "dokime") && path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths.into_iter().map(PluginBundle::new).collect())
}

fn read_bundle(input: PluginBundle, cached_digest: Option<&str>, archive_bytes: usize) -> Result<ReadBundle> {
    let bytes = read_bounded(&input.path, archive_bytes).map_err(operation)?;
    let content_digest = match cached_digest {
        Some(digest) => digest.to_owned(),
        None => digest(&bytes),
    };
    if let PluginProvenance::Installed { install_id, sha256, .. } = &input.provenance
        && content_digest != *sha256
    {
        return Err(operation(format!(
            "installed plugin `{install_id}` at `{}` no longer matches the digest it was authorized under; reinstall it",
            input.path.display()
        )));
    }
    Ok(ReadBundle { input, bytes, content_digest })
}

async fn prepare(read: ReadBundle, extraction: &std::path::Path, reuse: Option<PathBuf>, handle: &super::RuntimeHandle) -> Result<Package> {
    let ReadBundle { input, bytes, content_digest } = read;
    let (bundle, metadata) = crate::plugins::inspect_with_archive_limit(
        &bytes,
        handle.schema_settings(),
        usize::try_from(dokime_domain::model::plugins::wire::INSTALL_ARCHIVE_MAX_BYTES)
            .expect("validated server plugin bundle limit fits native address space"),
    )
    .map_err(operation)?;
    let selected = selected_files(&bundle).map_err(operation)?;
    if let Some(owner) = input.programs.keys().find(|owner| !metadata.owners.contains(owner)) {
        return Err(operation(format!("plugin `{}` configuration names unknown executable `{owner}`", input.path.display())));
    }
    let mut plugins = Vec::new();
    let directory = match reuse {
        Some(cached) => Extraction::Cached(cached),
        None => {
            let staged = bundle.extract(extraction).map_err(operation)?;
            match &input.provenance {
                PluginProvenance::Installed { install_id, .. } => {
                    let target = extraction.join(install_id.to_string());
                    persist_extraction(staged, &target, &input.path, &content_digest).map_err(operation)?;
                    Extraction::Cached(target)
                }
                PluginProvenance::Configured => Extraction::Ephemeral(staged),
            }
        }
    };
    let mut programs = Vec::new();
    for file in selected {
        let declaration = bundle.declaration(&file.path).expect("selected executable declaration");
        if !crate::plugins::catalog::native(declaration) {
            return Err(operation(crate::plugins::catalog::guest_refused(&file.path)));
        }
        let (id, version, description, operations) =
            (&declaration.id, &declaration.version, &declaration.description, &declaration.operations);
        dokime_domain::model::identity::Identifier::new(id).map_err(operation)?;
        let registrations = file.executable.as_ref().expect("selected executable").registrations.clone();
        let plugin = decode_registrations(id, &version.to_string(), description, &registrations)?;
        super::native_telemetry::validate(&plugin, &declaration.streams, handle.schema_settings()).map_err(operation)?;
        crate::plugins::dispatch::validate_analyses(&plugin, &declaration.contracts.provides, handle.schema_settings())
            .map_err(operation)?;
        let bindings = crate::plugins::dispatch::Bindings::inspect(&plugin, operations).map_err(operation)?;
        plugins.push(plugin.clone());
        let mut executable = harmos::sidecar::Executable::at(directory.path().join(&file.path)).map_err(operation)?;
        if executable.declaration() != declaration {
            return Err(operation(format!("extracted executable `{id}` has a different declaration than its verified bundle")));
        }
        if let Some(filter) = std::env::var_os(PLUGIN_LOG_VARIABLE) {
            executable = executable.variable("RUST_LOG", plugin_log_filter(filter));
        }

        if executable.sha256().iter().map(|byte| format!("{byte:02x}")).collect::<String>() != file.sha256 {
            return Err(operation(format!("extracted executable `{id}` differs from its verified bundle")));
        }
        let configured = input.programs.get(id).cloned().unwrap_or_else(crate::plugins::config::empty_object);
        let initialization = handle.plugin_initialization(&plugin, &configured).await?;
        let mut definition =
            harmos::plugin::PluginDefinition::sidecar(&executable, serde_json::to_vec(&initialization).map_err(operation)?)
                .map_err(operation)?;
        definition.contributions.registrations = registrations;

        programs.push(Program {
            plugin,
            executable: Executable(executable),
            definition,
            bindings,
            configured,
        });
    }
    let installation = match &input.provenance {
        PluginProvenance::Configured => None,
        PluginProvenance::Installed { install_id, .. } => Some(install_id.to_string()),
    };
    let path = input.path.to_string_lossy().into_owned();
    Ok(Package {
        input,
        catalog: BundleCatalog {
            package: metadata.identity.name,
            path,
            content_digest,
            installation,
            plugins,
        },
        programs,
        _directory: Arc::new(directory),
    })
}

fn reusable_extraction(extraction: &std::path::Path, input: &PluginBundle) -> Option<PathBuf> {
    let PluginProvenance::Installed { install_id, sha256, .. } = &input.provenance else {
        return None;
    };
    let target = extraction.join(install_id.to_string());
    let marker_bytes = std::fs::read(target.join(EXTRACTION_MARKER_FILE)).ok()?;
    let marker: ExtractionMarker = serde_json::from_slice(&marker_bytes).ok()?;
    let stat = std::fs::metadata(&input.path).ok()?;
    if !extraction_matches(&marker, sha256, stat.len(), mtime_unix_ms(&stat)?) {
        return None;
    }
    // A light existence check that the directory still looks intact, not a
    // full re-validation -- that would defeat the point of trusting it.
    target.join("manifest.json").is_file().then_some(target)
}

fn extraction_matches(marker: &ExtractionMarker, sha256: &str, size: u64, modified: i64) -> bool {
    marker.sha256 == sha256 && marker.size == size && marker.mtime_unix_ms == modified
}

fn persist_extraction(
    staged: tempfile::TempDir, target: &std::path::Path, bundle_path: &std::path::Path, content_digest: &str,
) -> std::io::Result<()> {
    if target.is_dir() {
        // Leftover from an interrupted extraction under this same install_id;
        // never legitimately reused directly, so it is cleared before the
        // rename lands the new one in its place.
        std::fs::remove_dir_all(target)?;
    }
    std::fs::rename(staged.path(), target)?;
    let stat = std::fs::metadata(bundle_path)?;
    let marker = ExtractionMarker {
        sha256: content_digest.to_owned(),
        size: stat.len(),
        mtime_unix_ms: mtime_unix_ms(&stat).unwrap_or(0),
    };
    harmos::filesystem::publication::write(
        &target.join(EXTRACTION_MARKER_FILE),
        &serde_json::to_vec(&marker).map_err(std::io::Error::other)?,
    )
    .map_err(Into::into)
}

fn mtime_unix_ms(stat: &std::fs::Metadata) -> Option<i64> {
    let elapsed = stat.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?;
    i64::try_from(elapsed.as_millis()).ok()
}

fn sweep_orphaned_extractions(extraction: &std::path::Path, inputs: &[PluginBundle]) {
    let live: BTreeSet<String> = inputs
        .iter()
        .filter_map(|input| match &input.provenance {
            PluginProvenance::Installed { install_id, .. } => Some(install_id.to_string()),
            PluginProvenance::Configured => None,
        })
        .collect();
    let Ok(entries) = std::fs::read_dir(extraction) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !orphaned_extraction(name, &live) {
            continue;
        }
        if let Err(error) = std::fs::remove_dir_all(entry.path()) {
            tracing::warn!(
                %error,
                directory = name,
                "could not reclaim an orphaned plugin extraction directory"
            );
        }
    }
}

fn orphaned_extraction(name: &str, live: &BTreeSet<String>) -> bool {
    !live.contains(name) && uuid::Uuid::parse_str(name).is_ok()
}

fn package_definitions(packages: &[&Package]) -> Vec<Vec<harmos::plugin::PluginDefinition>> {
    packages.iter().map(|package| package.programs.iter().map(|program| program.definition.clone()).collect()).collect()
}

async fn plan(
    store: &PluginStore, packages: &[&Package], embedded: &[harmos::plugin::PluginDefinition], operations: &crate::app::OperationManager,
) -> Result<BTreeSet<usize>> {
    let mut refusals = Vec::new();
    let result = plan_graph(
        &package_definitions(packages),
        embedded,
        |candidate| operations.validate_requirements(candidate),
        |index, reason| {
            let message = match reason.code() {
                Some(code) => format!("{code}: {reason}"),
                None => reason.to_string(),
            };
            refusals.push((index, message));
            Ok(())
        },
    );
    for (index, reason) in refusals {
        failed(store, &packages[index].input, &reason).await?;
    }
    result
}

fn plan_graph(
    packages: &[Vec<harmos::plugin::PluginDefinition>], embedded: &[harmos::plugin::PluginDefinition],
    mut validate_host: impl FnMut(&harmos::plugin::PluginDefinitions) -> harmos::Result<()>,
    mut refuse: impl FnMut(usize, &Error) -> Result<()>,
) -> Result<BTreeSet<usize>> {
    let mut removed = BTreeSet::new();
    loop {
        let mut locations = BTreeMap::new();
        let mut plugins = Vec::new();
        for (package_index, package) in packages.iter().enumerate().filter(|(index, _)| !removed.contains(index)) {
            for plugin in package {
                locations.insert(plugin.id.clone(), package_index);
                plugins.push(plugin.clone());
            }
        }
        plugins.extend_from_slice(embedded);
        match harmos::plugin::PluginGraph::resolve(plugins.clone()) {
            Ok(_) => {
                let candidate = harmos::plugin::PluginDefinitions::new(plugins, [])?;
                let Err(failure) = validate_host(&candidate) else {
                    return Ok(removed);
                };
                let mut unavailable = None;
                for (definition, required) in candidate.host_requirements() {
                    // Harmos Leaves Unprovided Contracts For Host Admission. Check
                    // Each Such Requirement Before Starting Any Plugin Generation.
                    let mut isolated = definition.clone();
                    isolated.requires.clear();
                    isolated.contributions.required_contracts = Some(vec![required.clone()]);
                    let isolated = harmos::plugin::PluginDefinitions::new([isolated], [])?;
                    if let Err(source) = validate_host(&isolated) {
                        let reason = Error::unavailable(
                            "plugin_contract_unavailable",
                            format!(
                                "plugin `{}` requires unavailable contract `{}`: {source}",
                                definition.id,
                                required.contract.operation().id,
                            ),
                        );
                        let Some(&package) = locations.get(&definition.id) else {
                            return Err(reason);
                        };
                        unavailable = Some((package, reason));
                        break;
                    }
                }
                let Some((package, reason)) = unavailable else {
                    return Err(failure.into());
                };
                for index in prune_graph(packages, &mut removed, package) {
                    refuse(index, &reason)?;
                }
            }
            Err(error) => {
                if let harmos::plugin::Error::Cycle { artifact } = &error {
                    let (members, contracts) = circular_contracts(&plugins, artifact);
                    let reason = Error::conflict(
                        "plugin_contract_cycle",
                        format!("circular plugin requirements among {members:?} through contracts {contracts:?}"),
                    );
                    let affected: BTreeSet<_> = members.iter().filter_map(|member| locations.get(member).copied()).collect();
                    if affected.is_empty() {
                        return Err(reason);
                    }
                    for package in affected {
                        for index in prune_graph(packages, &mut removed, package) {
                            refuse(index, &reason)?;
                        }
                    }
                    continue;
                }
                let owner = match &error {
                    harmos::plugin::Error::Unknown { .. } => {
                        return Err(operation(error));
                    }
                    harmos::plugin::Error::Definition { artifact: plugin, .. } => plugin,
                    harmos::plugin::Error::StreamConflict { second, .. } => second,
                    harmos::plugin::Error::Duplicate { id } => id,
                    harmos::plugin::Error::Missing { artifact: plugin, .. }
                    | harmos::plugin::Error::Incompatible { artifact: plugin, .. }
                    | harmos::plugin::Error::Cycle { artifact: plugin }
                    | harmos::plugin::Error::IncompatibleOperation { artifact: plugin, .. }
                    | harmos::plugin::Error::AmbiguousOperation { artifact: plugin, .. }
                    | harmos::plugin::Error::ProviderSelection { artifact: plugin, .. } => plugin,
                };
                let Some(&package) = locations.get(owner) else {
                    return Err(operation(error));
                };
                let reason = match error {
                    error @ (harmos::plugin::Error::IncompatibleOperation { .. }
                    | harmos::plugin::Error::AmbiguousOperation { .. }
                    | harmos::plugin::Error::ProviderSelection { .. }) => {
                        Error::unavailable("plugin_contract_unavailable", error.to_string())
                    }
                    error => operation(error),
                };
                for index in prune_graph(packages, &mut removed, package) {
                    refuse(index, &reason)?;
                }
            }
        }
    }
}

fn circular_contracts(plugins: &[harmos::plugin::PluginDefinition], owner: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut edges = BTreeMap::<&str, BTreeSet<&str>>::new();
    for plugin in plugins {
        let dependencies = edges.entry(&plugin.id).or_default();
        dependencies.extend(plugin.requires.iter().map(|required| required.id.as_str()));
        for required in plugin.contributions.required_contracts.iter().flatten() {
            dependencies.extend(
                plugins
                    .iter()
                    .filter(|provider| provider.id != plugin.id && provider.contributions.wire_contracts.contains(&required.contract))
                    .map(|provider| provider.id.as_str()),
            );
        }
    }
    let reachable = |start| {
        let mut visited = BTreeSet::new();
        let mut pending = vec![start];
        while let Some(next) = pending.pop() {
            if visited.insert(next) {
                pending.extend(edges.get(next).into_iter().flatten().copied());
            }
        }
        visited
    };
    let members: BTreeSet<String> =
        reachable(owner).into_iter().filter(|member| reachable(member).contains(owner)).map(str::to_owned).collect();
    let contracts = plugins
        .iter()
        .filter(|plugin| members.contains(&plugin.id))
        .flat_map(|plugin| plugin.contributions.required_contracts.iter().flatten())
        .filter(|required| {
            plugins
                .iter()
                .any(|provider| members.contains(&provider.id) && provider.contributions.wire_contracts.contains(&required.contract))
        })
        .map(|required| required.contract.operation().id.to_string())
        .collect();
    (members, contracts)
}

async fn prune(
    store: &PluginStore, packages: &[&Package], removed: &mut BTreeSet<usize>, initial: usize, reason: &str,
    embedded: &[harmos::plugin::PluginDefinition], operations: &crate::app::OperationManager,
) -> Result<()> {
    for index in prune_graph(&package_definitions(packages), removed, initial) {
        failed(store, &packages[index].input, reason).await?;
    }
    // Removing A Provider Can Turn A Previously Routed Contract Into A Host
    // Requirement. Replan Before Activation, Including Startup Failure Retries.
    let remaining: Vec<_> = packages.iter().enumerate().filter(|(index, _)| !removed.contains(index)).collect();
    let further = plan(store, &remaining.iter().map(|(_, package)| **package).collect::<Vec<_>>(), embedded, operations).await?;
    removed.extend(further.into_iter().map(|index| remaining[index].0));
    Ok(())
}

fn prune_graph(packages: &[Vec<harmos::plugin::PluginDefinition>], removed: &mut BTreeSet<usize>, initial: usize) -> BTreeSet<usize> {
    let mut newly_removed = BTreeSet::new();
    if removed.insert(initial) {
        newly_removed.insert(initial);
    }
    loop {
        let remaining: BTreeSet<_> = packages
            .iter()
            .enumerate()
            .filter(|(index, _)| !removed.contains(index))
            .flat_map(|(_, package)| package.iter().map(|plugin| plugin.id.as_str()))
            .collect();
        let unavailable: BTreeSet<_> = removed
            .iter()
            .flat_map(|index| packages[*index].iter().map(|plugin| plugin.id.as_str()))
            .filter(|owner| !remaining.contains(owner))
            .collect();
        let dependent = packages.iter().enumerate().find_map(|(index, package)| {
            (!removed.contains(&index)
                && package.iter().any(|plugin| plugin.requires.iter().any(|requirement| unavailable.contains(requirement.id.as_str()))))
            .then_some(index)
        });
        let Some(index) = dependent else { break };
        removed.insert(index);
        newly_removed.insert(index);
    }
    newly_removed
}

async fn failed(store: &PluginStore, input: &PluginBundle, reason: &str) -> Result<()> {
    let PluginProvenance::Installed { install_id, .. } = &input.provenance else {
        return Err(operation(format!("configured plugin `{}` failed: {reason}", input.path.display())));
    };
    tracing::warn!(
        plugin = %input.path.display(),
        install_id = %install_id,
        reason,
        "plugin was not admitted"
    );
    record(store, input, Some(reason.to_owned())).await;
    Ok(())
}

async fn record(store: &PluginStore, input: &PluginBundle, error: Option<String>) {
    if let PluginProvenance::Installed { install_id, .. } = &input.provenance
        && let Err(error) = store.record_boot(*install_id, error).await
    {
        tracing::warn!(%install_id, %error, "could not record plugin boot outcome");
    }
}

fn operation(error: impl std::fmt::Display) -> Error {
    Error::Operation(error.to_string())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use harmos::domain::inventory::contracts::Required;
    use harmos::operations::{Contract, PluginRegistration, Registration, WireDeclaration};
    use harmos::plugin::{
        PluginArtifact, PluginBackend, PluginContributions, PluginDefinition, PluginDefinitions, PluginEndpoint, PluginLifecycle,
        PluginLifecycleAuthority, PluginManager, PluginTransition, Router,
    };

    #[harmos::operation(id = "order.probe", response = (), visibility = Public, policy = dokime_operations::ORDINARY)]
    struct Probe;

    struct Host;

    struct Lifecycle {
        events: Vec<String>,
        operations: crate::app::OperationManager,
    }

    #[derive(Default)]
    struct Staged {
        endpoint: Option<PluginEndpoint>,
        operations: Option<PluginRegistration>,
    }

    impl harmos::Lifecycle for Host {
        type Config = ();
        type Error = std::convert::Infallible;
        type Context = harmos::Startup<Self>;

        async fn prepare(_: (), _: &mut harmos::Setup<Self>) -> std::result::Result<Self, Self::Error> {
            Ok(Self)
        }
    }

    impl harmos::Application for Host {
        type Lanes = crate::app::Lanes;
        type State = ();
        type Transaction = harmos::state::Never;
        type Record = ();
        type Streams = ();
    }

    impl harmos::Operation for Probe {
        type Error = harmos::Error;

        async fn run(self, _: harmos::Context<'_, Self>) -> harmos::Result<harmos::Response<()>> {
            Ok(().into())
        }
    }

    impl PluginLifecycle for Lifecycle {
        type Guest = ();
        type Sidecar = ();
        type Embedded = Staged;

        fn preflight(&mut self, candidate: &PluginDefinitions) -> harmos::Result<()> {
            self.operations.validate_requirements(candidate)
        }

        fn prepare(&mut self, definition: &PluginDefinition) -> harmos::Result<PluginBackend<(), (), Staged>> {
            self.events.push(format!("prepare:{}", definition.id));
            Ok(PluginBackend::Embedded(Staged::default()))
        }

        async fn fence(&mut self, _: PluginTransition, _: &[PluginDefinition]) -> harmos::Result<()> {
            Ok(())
        }

        async fn start(
            &mut self, definition: &PluginDefinition, router: &Router, _: &PluginLifecycleAuthority<'_>,
            backend: &mut PluginBackend<(), (), Staged>,
        ) -> harmos::Result<()> {
            let PluginBackend::Embedded(staged) = backend else { unreachable!() };
            staged.endpoint = Some(router.enter(&definition.id).unwrap());
            let scope = staged.endpoint.as_ref().unwrap().scope().unwrap();
            let mut registration = Registration::new(crate::app::Lanes::Open);
            for operation in &definition.contributions.operations {
                registration = registration.operation::<Probe>(operation.id.as_str())?;
            }
            staged.operations = Some(self.operations.stage_plugin(scope, |bindings| bindings.register(registration))?);
            self.events.push(format!("start:{}", definition.id));
            Ok(())
        }

        async fn stop(
            _: &PluginDefinition, _: &PluginLifecycleAuthority<'_>, backend: &mut PluginBackend<(), (), Staged>,
        ) -> harmos::Result<()> {
            let PluginBackend::Embedded(staged) = backend else { unreachable!() };
            if let Some(operations) = &mut staged.operations {
                operations.shutdown().await?;
            }
            staged.operations.take();
            staged.endpoint.take();
            Ok(())
        }
    }

    fn contract(id: &str) -> Contract {
        WireDeclaration::of::<Probe>().contract().unwrap().with_id(id).unwrap()
    }

    fn definition(id: &str, provides: &[&str], requires: &[&str]) -> PluginDefinition {
        let wire_contracts: Vec<_> = provides.iter().map(|id| contract(id)).collect();
        PluginDefinition {
            id: id.into(),
            version: "0.1.0".parse().unwrap(),
            artifact: PluginArtifact::Embedded { revision: "fixture".into() },
            requires: Vec::new(),
            configuration: b"null".to_vec(),
            contributions: PluginContributions {
                operations: wire_contracts.iter().map(|contract| contract.operation().clone()).collect(),
                wire_contracts,
                required_contracts: Some(
                    requires
                        .iter()
                        .map(|id| Required {
                            contract: contract(id),
                            grants: Vec::new(),
                        })
                        .collect(),
                ),
                ..Default::default()
            },
        }
    }

    #[test]
    fn peer_order() {
        let plugins = [definition("alpha", &[], &[]), definition("beta", &[], &[]), definition("gamma", &[], &[])];
        let forward = PluginDefinitions::new(plugins.clone(), []).unwrap();
        let reversed = PluginDefinitions::new(plugins.into_iter().rev(), []).unwrap();
        let names: Vec<_> = forward.definitions().map(|definition| definition.id.as_str()).collect();
        assert_eq!(names, ["gamma", "beta", "alpha"]);
        assert_eq!(names, reversed.definitions().map(|definition| definition.id.as_str()).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn consumer_before_provider() {
        let host = harmos::Runtime::<Host>::launch((), ()).await.unwrap();
        let packages =
            vec![vec![definition("z_consumer", &[], &["order.provider"])], vec![definition("a_provider", &["order.provider"], &[])]];
        let removed = plan_graph(
            &packages,
            &[],
            |candidate| host.operations.validate_requirements(candidate),
            |_, reason| panic!("unexpected refusal: {reason}"),
        )
        .unwrap();
        assert!(removed.is_empty());
        let definitions = PluginDefinitions::new(packages.into_iter().flatten(), []).unwrap();
        let mut manager = PluginManager::new(definitions).unwrap();
        let mut lifecycle = Lifecycle {
            events: Vec::new(),
            operations: host.operations.clone(),
        };
        manager.initialize(&mut lifecycle).await.unwrap();
        assert_eq!(lifecycle.events, ["prepare:a_provider", "start:a_provider", "prepare:z_consumer", "start:z_consumer"]);
        manager.shutdown(&mut lifecycle).await.unwrap();
        host.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn circular_contracts() {
        let host = harmos::Runtime::<Host>::launch((), ()).await.unwrap();
        let packages = vec![
            vec![definition("left", &["order.left"], &["order.right"])],
            vec![definition("right", &["order.right"], &["order.left"])],
            vec![definition("unrelated", &[], &[])],
        ];
        let mut refusals = BTreeMap::new();
        let removed = plan_graph(
            &packages,
            &[],
            |candidate| host.operations.validate_requirements(candidate),
            |index, reason| {
                assert_eq!(reason.code(), Some("plugin_contract_cycle"));
                refusals.insert(index, reason.to_string());
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(removed, BTreeSet::from([0, 1]));
        assert_eq!(refusals.len(), 2);
        for reason in refusals.values() {
            assert!(reason.contains("order.left") && reason.contains("order.right"));
        }
        let definitions = PluginDefinitions::new(
            packages.into_iter().enumerate().filter(|(index, _)| !removed.contains(index)).flat_map(|(_, package)| package),
            [],
        )
        .unwrap();
        let mut manager = PluginManager::new(definitions).unwrap();
        let mut lifecycle = Lifecycle {
            events: Vec::new(),
            operations: host.operations.clone(),
        };
        manager.initialize(&mut lifecycle).await.unwrap();
        assert_eq!(lifecycle.events, ["prepare:unrelated", "start:unrelated"]);
        manager.shutdown(&mut lifecycle).await.unwrap();
        host.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn missing_contract() {
        let host = harmos::Runtime::<Host>::launch((), ()).await.unwrap();
        let packages = vec![
            vec![definition("consumer", &[], &["order.provider"])],
            vec![definition("provider", &["order.provider"], &["order.missing"])],
            vec![definition("unrelated", &[], &[])],
        ];
        let mut refusals = BTreeMap::new();
        let removed = plan_graph(
            &packages,
            &[],
            |candidate| host.operations.validate_requirements(candidate),
            |index, reason| {
                assert_eq!(reason.code(), Some("plugin_contract_unavailable"));
                refusals.insert(index, reason.to_string());
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(removed, BTreeSet::from([0, 1]));
        assert!(refusals[&0].contains("order.provider"));
        assert!(refusals[&1].contains("order.missing"));
        let definitions = PluginDefinitions::new(
            packages.into_iter().enumerate().filter(|(index, _)| !removed.contains(index)).flat_map(|(_, package)| package),
            [],
        )
        .unwrap();
        let mut manager = PluginManager::new(definitions).unwrap();
        let mut lifecycle = Lifecycle {
            events: Vec::new(),
            operations: host.operations.clone(),
        };
        manager.initialize(&mut lifecycle).await.unwrap();
        assert_eq!(lifecycle.events, ["prepare:unrelated", "start:unrelated"]);
        manager.shutdown(&mut lifecycle).await.unwrap();
        host.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn embedded_provider_order() {
        let host = harmos::Runtime::<Host>::launch((), ()).await.unwrap();
        let registration = Registration::new(crate::app::Lanes::Open).operation::<Probe>("order.host").unwrap();
        host.operations.register_batch(registration).await.unwrap();
        let packages = vec![vec![definition("z_consumer", &[], &["order.embedded", "order.host"])]];
        let embedded = [definition("a_embedded", &["order.embedded"], &[])];
        let mut checked = Vec::new();
        let removed = plan_graph(
            &packages,
            &embedded,
            |candidate| {
                let requirements: Vec<_> =
                    candidate.host_requirements().map(|(_, required)| required.contract.operation().id.to_string()).collect();
                assert_eq!(requirements, ["order.host"]);
                checked.extend(requirements);
                host.operations.validate_requirements(candidate)
            },
            |_, reason| panic!("unexpected refusal: {reason}"),
        )
        .unwrap();
        assert!(removed.is_empty());
        assert_eq!(checked, ["order.host"]);
        let definitions = PluginDefinitions::new(packages.into_iter().flatten().chain(embedded), []).unwrap();
        let mut manager = PluginManager::new(definitions).unwrap();
        let mut lifecycle = Lifecycle {
            events: Vec::new(),
            operations: host.operations.clone(),
        };
        manager.initialize(&mut lifecycle).await.unwrap();
        assert_eq!(lifecycle.events, ["prepare:a_embedded", "start:a_embedded", "prepare:z_consumer", "start:z_consumer"]);
        manager.shutdown(&mut lifecycle).await.unwrap();
        host.shutdown().await.unwrap();
    }
}
