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

// Third Party Imports
use uuid::Uuid;

// Local Imports
use super::restarts::{Outcome, Target};
use super::{
    ActivePackage, Package, PluginHost, admission, extraction_root, graph, kept, lifecycle, noted, operation, prepare_input, record,
    refusal, rejection, start_error, start_failure,
};
use crate::plugins::{PluginBundle, PluginState, PluginStore};
use crate::runtime::{Error, Result};
use dokime_domain::model::plugins::wire::PluginStartFailure;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The Packages A Change Took Out Of The Running Set, Kept To Start Them Again If It Fails.
struct Replaced(Vec<(String, ActivePackage)>);

/* === Enums === */

enum Planned {
    // The reload to apply, and the packages it stops because they need what changed.
    Apply { reload: harmos::plugin::PluginReloadPlan, dropped: BTreeMap<String, PluginStartFailure> },

    // The new bundle is refused before anything stops.
    Refused(PluginStartFailure),
}

/* === Traits === */

/* === Implementations === */

impl PluginHost {
    // Bring `package` In Line With The Store: Start, Replace Or Stop Its Programs, Once Nothing Uses Them.
    //
    // The store names what should run: the requested installation while it is
    // installed, else the running one while it still is, else nothing. The
    // change is admitted like a startup, so it ends where a restart would.
    pub(super) async fn apply_bundle(&mut self, package: &str, requested: Option<Uuid>) -> Outcome {
        let store = self.handle.plugin_store();
        let running = self.plugins.get(package).map(|active| active.package.input.clone());
        // Configured Bundles Change Through Their Configuration Or A Development Reload.
        if running.as_ref().is_some_and(|input| input.install_id().is_none()) {
            return Outcome::Settled;
        }
        let current = running.as_ref().and_then(PluginBundle::install_id);
        let desired = match desired(&store, requested, current).await {
            Ok(desired) => desired,
            Err(error) => {
                tracing::error!(package, %error, "the plugin store could not be read; the running programs are unchanged");
                return Outcome::Settled;
            }
        };
        if desired == current && (desired.is_none() || self.package_ready(package)) {
            return Outcome::Settled;
        }
        // Deferring Before Preparing Spares Re-Reading The Bundle On Every State Change.
        if let Some(settling) = self.handle.plugin_use(&self.package_scope(package)) {
            return Outcome::Deferred(settling);
        }

        self.handle.plugin_restarts.begin(&Target::Bundle(package.to_owned()));
        match (desired, current) {
            (Some(install), _) => self.start_bundle(&store, package, install, running).await,
            (None, Some(_)) => self.stop_bundle(&store, package).await,
            (None, None) => Outcome::Settled,
        }
    }

    // Start `install` As `package`, Replacing What Runs Under That Name.
    async fn start_bundle(&mut self, store: &PluginStore, package: &str, install: Uuid, running: Option<PluginBundle>) -> Outcome {
        let input = match store.bundle(install).await {
            Ok(Some(input)) => input,
            Ok(None) => return Outcome::Settled,
            Err(error) => {
                tracing::error!(package, %error, "the installed bundle could not be read; the running programs are unchanged");
                return Outcome::Settled;
            }
        };
        let extraction = extraction_root(&self.handle.config());
        let prepared = match std::fs::create_dir_all(&extraction) {
            Ok(()) => prepare_input(input.clone(), &extraction, &self.handle).await,
            Err(error) => Err(operation(error)),
        };
        let candidate = match prepared {
            Ok(candidate) if candidate.catalog.package == package => candidate,
            Ok(candidate) => {
                let reason = format!("its bundle names package `{}`", candidate.catalog.package);
                return self.refuse(store, package, &input, running.as_ref(), rejection(package, reason, None)).await;
            }
            Err(error) => return self.refuse(store, package, &input, running.as_ref(), refusal(package, &error)).await,
        };
        let (reload, dropped) = match self.plan_bundle(package, Some(&candidate)) {
            Ok(Planned::Apply { reload, dropped }) => (reload, dropped),
            Ok(Planned::Refused(failure)) => return self.refuse(store, package, &input, running.as_ref(), failure).await,
            Err(error) => return self.refuse(store, package, &input, running.as_ref(), refusal(package, &error)).await,
        };
        if let Err(blocker) = self.handle.admit_plugin_restart(&reload.affected()).await {
            return Outcome::Deferred(blocker);
        }

        let progress = self.handle.operation_progress().begin(format!("Starting {package}"));
        let replaced = self.replace(package, Some(candidate), dropped.keys());
        let applied = self.apply(reload).await;
        let Err(error) = applied else {
            self.settle_change(store, package, Some(&input), &replaced, &dropped).await;
            progress.finish();
            return Outcome::Settled;
        };

        // The New Generation Did Not Start, And Harmos Stopped What It Replaced: Start That Again.
        let failure = match &error {
            Error::Harmos(error) => start_failure(error).map(|(owner, source, _)| start_error(owner, package, source)),
            _ => None,
        }
        .unwrap_or_else(|| rejection(package, &error, None));
        tracing::warn!(package, reason = %failure.message, "installed plugin did not start");
        let previous = running.as_ref().and_then(PluginBundle::install_id);
        let restored = self.restore(package, replaced).await;
        self.reconcile_ready(package).await;
        if let Err(error) = self.expose_plugins() {
            tracing::error!(package, %error, "restored plugins were not exposed");
        }
        self.publish_hosted();
        match (previous, restored) {
            (Some(previous), Ok(())) if previous != install => {
                self.keep(store, package, previous, install, noted(failure, "The previous version runs again.")).await;
            }
            // A retried installation that ran before runs again; its startup state stands.
            (Some(_), Ok(())) => record(store, &input, Some(noted(failure, "The previous generation runs again."))).await,
            (previous, restored) => {
                let unrestored = previous.zip(restored.err());
                let failed = std::iter::once(install).chain(unrestored.as_ref().map(|(previous, _)| *previous));
                self.handle.plugin_boot.write().expect("plugin boot lock poisoned").extend(failed.map(|install| (install, false)));
                record(store, &input, Some(failure)).await;
                // The Replaced Installation Did Not Start Again Either: Its Own Failure Says So.
                if let (Some(previous), Some((_, error))) = (running.as_ref().filter(|_| previous != Some(install)), &unrestored) {
                    tracing::error!(package, %error, "the replaced plugin did not start again");
                    record(store, previous, Some(rejection(package, format!("it did not start again: {error}"), None))).await;
                }
            }
        }
        progress.fail(format!("{package} did not start"));
        Outcome::Settled
    }

    // Stop What Runs As `package`; Packages That Need It Stop With It, As At Startup.
    async fn stop_bundle(&mut self, store: &PluginStore, package: &str) -> Outcome {
        let (reload, dropped) = match self.plan_bundle(package, None) {
            Ok(Planned::Apply { reload, dropped }) => (reload, dropped),
            Ok(Planned::Refused(failure)) => {
                tracing::error!(package, reason = %failure.message, "the plugin was not stopped");
                return Outcome::Settled;
            }
            Err(error) => {
                tracing::error!(package, %error, "the plugin was not stopped");
                return Outcome::Settled;
            }
        };
        if let Err(blocker) = self.handle.admit_plugin_restart(&reload.affected()).await {
            return Outcome::Deferred(blocker);
        }

        let progress = self.handle.operation_progress().begin(format!("Stopping {package}"));
        let replaced = self.replace(package, None, dropped.keys());
        let applied = self.apply(reload).await;
        self.settle_change(store, package, None, &replaced, &dropped).await;
        if let Err(error) = progress.track(applied) {
            tracing::error!(package, %error, "the plugin did not stop cleanly");
        }
        Outcome::Settled
    }

    // Admit The Running Set With `package` Replaced By `candidate`, Or Without It.
    fn plan_bundle(&self, package: &str, candidate: Option<&Package>) -> Result<Planned> {
        let mut packages: Vec<&Package> =
            self.plugins.iter().filter(|(name, _)| *name != package).map(|(_, active)| &active.package).collect();
        let others = packages.len();
        packages.extend(candidate);
        let mut refusals = admission(&packages, &self.embedded, &self.handle)?;
        if let Some(failure) = refusals.remove(&others) {
            return Ok(Planned::Refused(failure));
        }
        if let Some(candidate) = candidate {
            for program in &candidate.programs {
                self.handle
                    .streams
                    .validate_publications(program.executable.id(), program.executable.0.declaration().streams.clone())
                    .map_err(operation)?;
            }
        }

        let dropped: BTreeMap<_, _> =
            refusals.into_iter().map(|(index, failure)| (packages[index].catalog.package.clone(), failure)).collect();
        let programs =
            packages.iter().filter(|package| !dropped.contains_key(&package.catalog.package)).flat_map(|package| &package.programs);
        let rebuilt =
            candidate.into_iter().flat_map(|candidate| candidate.programs.iter().map(|program| program.executable.id().to_owned()));
        let definitions = graph(programs, self.embedded.values(), rebuilt)?;
        let reload = self.plugin_manager.as_ref().expect("manager initialized at startup").plan_reload(definitions)?;
        Ok(Planned::Apply { reload, dropped })
    }

    // Swap The Running Set; The Manager's Backends Keep The Old Generations' Files Until They Stop.
    fn replace<'a>(&mut self, package: &str, candidate: Option<Package>, dropped: impl Iterator<Item = &'a String>) -> Replaced {
        let mut replaced = Vec::new();
        replaced.extend(self.plugins.remove(package).map(|previous| (package.to_owned(), previous)));
        replaced.extend(dropped.filter_map(|name| self.plugins.remove(name).map(|active| (name.clone(), active))));
        if let Some(candidate) = candidate {
            self.plugins.insert(package.to_owned(), ActivePackage { package: candidate, running: Vec::new() });
        }
        Replaced(replaced)
    }

    async fn apply(&mut self, reload: harmos::plugin::PluginReloadPlan) -> Result<()> {
        let mut lifecycle = lifecycle::Adapter {
            handle: &self.handle,
            publications: &self.publications,
            packages: &mut self.plugins,
            embedded: &mut self.embedded,
        };
        let manager = self.plugin_manager.as_mut().expect("manager initialized at startup");
        manager.apply_reload(reload, &mut lifecycle).await.map_err(Error::from)
    }

    // Put The Replaced Packages Back And Start Them Again.
    async fn restore(&mut self, package: &str, replaced: Replaced) -> Result<()> {
        self.plugins.remove(package);
        let mut rebuilt = BTreeSet::new();
        for (name, mut active) in replaced.0 {
            // Their exposure ended when they stopped; starting again records it anew.
            active.running.clear();
            rebuilt.extend(active.package.programs.iter().map(|program| program.executable.id().to_owned()));
            self.plugins.insert(name, active);
        }
        let definitions = graph(self.plugins.values().flat_map(|active| &active.package.programs), self.embedded.values(), rebuilt)?;
        let reload = self.plugin_manager.as_ref().expect("manager initialized at startup").plan_reload(definitions)?;
        self.apply(reload).await
    }

    // A Change Is Reported Like A Startup: Catalogs, Exposure, And Each Installation's Outcome.
    async fn settle_change(
        &mut self, store: &PluginStore, package: &str, started: Option<&PluginBundle>, replaced: &Replaced,
        dropped: &BTreeMap<String, PluginStartFailure>,
    ) {
        self.reconcile_ready(package).await;
        if let Err(error) = self.expose_plugins() {
            tracing::error!(package, %error, "changed plugins were not exposed");
        }
        self.publish_hosted();
        {
            let mut boot = self.handle.plugin_boot.write().expect("plugin boot lock poisoned");
            for (name, active) in &replaced.0 {
                let Some(install) = active.package.input.install_id() else { continue };
                if dropped.contains_key(name) {
                    boot.insert(install, false);
                } else {
                    boot.remove(&install);
                }
            }
            if let Some(install) = started.and_then(PluginBundle::install_id) {
                boot.insert(install, true);
            }
        }
        if let Some(input) = started {
            record(store, input, None).await;
        }
        for (name, active) in &replaced.0 {
            if let Some(failure) = dropped.get(name) {
                tracing::warn!(package = %name, reason = %failure.message, "plugin stopped with a package it needs");
                record(store, &active.package.input, Some(failure.clone())).await;
            }
        }
    }

    // A Refused Change Stopped Nothing: What Runs Keeps Running, And The Store Says So.
    async fn refuse(
        &mut self, store: &PluginStore, package: &str, input: &PluginBundle, running: Option<&PluginBundle>, failure: PluginStartFailure,
    ) -> Outcome {
        tracing::warn!(package, reason = %failure.message, "installed plugin was not started");
        let install = input.install_id().expect("the store installs every applied bundle");
        match running.and_then(PluginBundle::install_id) {
            Some(previous) if previous != install => self.keep(store, package, previous, install, kept(failure)).await,
            // A retried installation that still runs keeps its generation and its startup state.
            Some(_) => record(store, input, Some(kept(failure))).await,
            None => {
                self.handle.plugin_boot.write().expect("plugin boot lock poisoned").insert(install, false);
                record(store, input, Some(failure)).await;
            }
        }
        Outcome::Settled
    }

    // A Refused Replacement Keeps The Previous Installation: It Is Installed Again With Its
    // Saved Values, And The Refused One Waits For Removal, Reported Like A Failed Startup.
    async fn keep(&mut self, store: &PluginStore, package: &str, previous: Uuid, refused: Uuid, failure: PluginStartFailure) {
        if let Err(error) = store.restore(previous, refused, failure).await {
            tracing::error!(package, %error, "the refused installation could not be recorded");
        }
        self.handle.plugin_boot.write().expect("plugin boot lock poisoned").insert(refused, false);
        let programs = self.plugins.get(package).into_iter().flat_map(|active| &active.package.programs);
        for program in programs {
            if let Ok(owner) = crate::Identifier::new(program.executable.id()) {
                self.handle.plugin_preferences().bind_install(&owner, previous);
            }
        }
    }

    fn package_ready(&self, package: &str) -> bool {
        let manager = self.plugin_manager.as_ref().expect("manager initialized at startup");
        let ready: BTreeSet<_> = manager.ready().collect();
        self.plugins
            .get(package)
            .is_some_and(|active| active.package.programs.iter().all(|program| ready.contains(program.executable.id())))
    }

    // The Package's Programs And Every Program That Restarts With Them.
    fn package_scope(&self, package: &str) -> BTreeSet<String> {
        let programs = self.plugins.get(package).into_iter().flat_map(|active| &active.package.programs);
        programs.flat_map(|program| self.restart_scope(program.executable.id())).collect()
    }
}

/* === Public Functions === */

/* === Private Functions === */

// What Should Run: The Requested Installation While Installed, Else The Running One While Still Installed.
async fn desired(
    store: &PluginStore, requested: Option<Uuid>, current: Option<Uuid>,
) -> std::result::Result<Option<Uuid>, crate::plugins::PluginError> {
    for install in requested.into_iter().chain(current) {
        if store.state(install).await? == Some(PluginState::Installed) {
            return Ok(Some(install));
        }
    }
    Ok(None)
}

/* === Tests === */
