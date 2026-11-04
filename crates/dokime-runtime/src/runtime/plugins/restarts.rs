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
use std::sync::{Arc, PoisonError, RwLock};

// Third Party Imports
use tokio_util::sync::CancellationToken;

// Local Imports
use super::PluginHost;
use crate::RuntimeHandle;
use crate::domains::installed_plugins::PluginRestart;
use crate::runtime::BackgroundTask;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Which Programs Wait To Restart With Their Saved Settings, Shared By Saves And The Restarter.
pub(in crate::runtime) struct Restarts {
    phases: tokio::sync::watch::Sender<BTreeMap<String, Phase>>,
    hosted: RwLock<BTreeSet<String>>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    // Saved values wait for the restarter; `blocked` while something uses an affected program.
    Waiting { blocked: bool },

    // A generation is starting; `again` when values were saved after it read them.
    Restarting { again: bool },
}

pub(in crate::runtime) enum Outcome {
    // The restart ran; its result is recorded like a startup's.
    Settled,

    // Something uses an affected program; the restarter tries again once it settles.
    Deferred(Option<CancellationToken>),

    // The program is not admitted; its saved values apply on the next start.
    Unhosted,
}

/* === Traits === */

/* === Implementations === */

impl Outcome {
    // Record The Outcome; A Deferred Restart Stays Waiting And Reports Pending.
    fn settle(self, restarts: &Restarts, plugin: &str) -> Self {
        match &self {
            Self::Settled => restarts.finish(plugin),
            Self::Unhosted => restarts.forget(plugin),
            Self::Deferred(_) => restarts.defer(plugin),
        }
        self
    }
}

impl Default for Restarts {
    fn default() -> Self {
        Self {
            phases: tokio::sync::watch::Sender::new(BTreeMap::new()),
            hosted: RwLock::default(),
        }
    }
}

impl Restarts {
    pub(in crate::runtime) fn host<'a>(&self, programs: impl Iterator<Item = &'a str>) {
        let programs = programs.map(str::to_owned).collect();
        *self.hosted.write().unwrap_or_else(PoisonError::into_inner) = programs;
    }

    // Further Saves Before The Restart Reads Its Values Join It; Later Ones Ask For One More.
    fn request(&self, plugin: &str, blocked: bool) -> PluginRestart {
        let hosted = self.hosted.read().unwrap_or_else(PoisonError::into_inner).contains(plugin);
        if !hosted {
            return PluginRestart::Idle;
        }

        self.phases.send_if_modified(|phases| match phases.get_mut(plugin) {
            Some(Phase::Restarting { again }) => !std::mem::replace(again, true),
            Some(Phase::Waiting { .. }) => false,
            None => phases.insert(plugin.to_owned(), Phase::Waiting { blocked }).is_none(),
        });
        self.state(plugin)
    }

    fn state(&self, plugin: &str) -> PluginRestart {
        match self.phases.borrow().get(plugin) {
            None => PluginRestart::Idle,
            Some(Phase::Waiting { blocked: true }) => PluginRestart::Pending,
            Some(Phase::Waiting { blocked: false } | Phase::Restarting { .. }) => PluginRestart::Restarting,
        }
    }

    fn waiting(&self) -> Vec<String> {
        let phases = self.phases.borrow();
        phases.iter().filter(|(_, phase)| matches!(phase, Phase::Waiting { .. })).map(|(plugin, _)| plugin.clone()).collect()
    }

    // A Deferred Restart Reads Its Values Again When It Runs, So Saves Meanwhile Join It.
    fn defer(&self, plugin: &str) {
        self.phases.send_if_modified(|phases| {
            let Some(phase) = phases.get_mut(plugin) else { return false };
            let deferred = Phase::Waiting { blocked: true };
            if *phase == deferred {
                return false;
            }
            *phase = deferred;
            true
        });
    }

    pub(in crate::runtime) fn begin(&self, plugin: &str) {
        self.phases.send_if_modified(|phases| {
            let Some(phase @ Phase::Waiting { .. }) = phases.get_mut(plugin) else { return false };
            *phase = Phase::Restarting { again: false };
            true
        });
    }

    fn finish(&self, plugin: &str) {
        self.phases.send_if_modified(|phases| {
            let Some(phase) = phases.get_mut(plugin) else { return false };
            if *phase == (Phase::Restarting { again: true }) {
                *phase = Phase::Waiting { blocked: false };
            } else {
                phases.remove(plugin);
            }
            true
        });
    }

    fn forget(&self, plugin: &str) {
        self.phases.send_if_modified(|phases| phases.remove(plugin).is_some());
    }
}

impl PluginHost {
    // The Program And Every Program That Restarts With It.
    pub(super) fn restart_scope(&self, plugin: &str) -> BTreeSet<String> {
        let manager = self.plugin_manager.as_ref().expect("manager initialized at startup");
        let mut scope = BTreeSet::from([plugin.to_owned()]);
        let mut frontier = vec![plugin.to_owned()];
        while let Some(next) = frontier.pop() {
            let dependents = manager.dependents(&next).unwrap_or_default();
            frontier.extend(dependents.into_iter().filter(|dependent| scope.insert(dependent.clone())));
        }
        scope
    }
}

impl RuntimeHandle {
    // A Save That Changed Values: Restart The Program Now, Or Once Nothing Uses It.
    pub(crate) fn request_plugin_restart(&self, plugin: &str) -> PluginRestart {
        let blocked = self.plugin_use(&BTreeSet::from([plugin.to_owned()])).is_some();
        self.plugin_restarts.request(plugin, blocked)
    }

    pub(crate) fn plugin_restart(&self, plugin: &str) -> PluginRestart {
        self.plugin_restarts.state(plugin)
    }

    // Resolves Once `plugin` Runs With Every Value Saved So Far, Or Its Failed Restart Is Recorded.
    pub async fn plugin_restart_settled(&self, plugin: &Identifier) {
        let mut phases = self.plugin_restarts.phases.subscribe();
        let _ = phases.wait_for(|phases| !phases.contains_key(plugin.as_str())).await;
    }

    // Under Plugin Admission: Refuse While Anything Uses An Affected Program, Else Close Their Admission.
    pub(in crate::runtime) async fn admit_plugin_restart(&self, affected: &[String]) -> Result<(), Option<CancellationToken>> {
        let affected: BTreeSet<String> = affected.iter().cloned().collect();
        let _admission = self.plugin_admission.lock().await;
        if let Some(settling) = self.plugin_use(&affected) {
            return Err(settling);
        }

        self.unavailable_plugins.write().expect("plugin availability lock poisoned").extend(affected);
        Ok(())
    }

    // `Some` While An Execution That Uses Any Of `sources` Is Running Or Prepared.
    //
    // An execution uses the program that runs its case, and every program
    // publishing telemetry or records it may capture. The inner value is a
    // provider run still settling, which the restarter can wait on.
    pub(in crate::runtime) fn plugin_use(&self, sources: &BTreeSet<String>) -> Option<Option<CancellationToken>> {
        let (publishing, executing) = {
            let state = self.state_read();
            let publishes = |plugin: &crate::InstalledPlugin| !plugin.telemetry_contracts.is_empty() || !plugin.record_contracts.is_empty();
            let publishing = sources.iter().filter_map(|source| state.installed_plugins.get(source)).any(publishes);
            let active = |status: &ExecutionStatus| matches!(status, ExecutionStatus::Pending | ExecutionStatus::Running);
            let runs = |execution: &dokime_domain::model::execution::result::TestExecution| {
                execution.snapshot.plugin().is_some_and(|plugin| sources.contains(&plugin.source))
            };
            let mut executions = state.executions.values().filter(|execution| execution.finished_at.is_none() && active(&execution.status));
            (publishing, executions.any(|execution| publishing || runs(execution)))
        };

        // A Provider Run Settles Its Captured Telemetry After Its Execution Finishes.
        let settling = {
            let tasks = self.provider_tasks.lock().expect("provider task lock poisoned");
            let uses = |source: &str| publishing || sources.contains(source);
            let settling = tasks.iter().find(|task| uses(&task.source) && !task.settled.is_cancelled());
            settling.map(|task| task.settled.clone())
        };
        (settling.is_some() || executing).then_some(settling)
    }
}

/* === Public Functions === */

/* === Private Functions === */

// One Restarter Per Runtime: Saves Queue Programs, And It Restarts Each Once Nothing Uses It.
pub(in crate::runtime) fn spawn(host: Arc<tokio::sync::Mutex<PluginHost>>, handle: RuntimeHandle) -> BackgroundTask {
    BackgroundTask(tokio::spawn(serve(host, handle)))
}

async fn serve(host: Arc<tokio::sync::Mutex<PluginHost>>, handle: RuntimeHandle) {
    let restarts = Arc::clone(&handle.plugin_restarts);
    let mut phases = restarts.phases.subscribe();
    let mut state = handle.state_watch.subscribe();
    let mut stopping = handle.shutdown_requested.subscribe();

    loop {
        let Some((blocked, settling)) = restart_waiting(&host, &restarts, &stopping).await else { return };
        tokio::select! {
            changed = phases.changed() => if changed.is_err() { return },
            changed = state.changed(), if blocked => if changed.is_err() { return },
            () = settled(settling.as_ref()), if settling.is_some() => {}
            () = stopped(&mut stopping) => return,
        }
    }
}

// One Pass Over Waiting Programs: Whether Any Stayed Blocked, And A Settling Provider To Wait On.
async fn restart_waiting(
    host: &tokio::sync::Mutex<PluginHost>, restarts: &Restarts, stopping: &tokio::sync::watch::Receiver<bool>,
) -> Option<(bool, Option<CancellationToken>)> {
    let mut blocked = false;
    let mut settling = None;
    for plugin in restarts.waiting() {
        let mut host = host.lock().await;
        if *stopping.borrow() {
            return None;
        }

        let outcome = host.restart_with_saved_settings(&plugin).await;
        let Outcome::Deferred(provider) = outcome.settle(restarts, &plugin) else { continue };
        blocked = true;
        settling = settling.or(provider);
    }
    Some((blocked, settling))
}

async fn settled(provider: Option<&CancellationToken>) {
    if let Some(provider) = provider {
        provider.cancelled().await;
    }
}

async fn stopped(stopping: &mut tokio::sync::watch::Receiver<bool>) {
    let _ = stopping.wait_for(|stopping| *stopping).await;
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn hosting(plugins: &[&str]) -> Restarts {
        let restarts = Restarts::default();
        restarts.host(plugins.iter().copied());
        restarts
    }

    #[test]
    fn repeated_saves_wait_once() {
        let restarts = hosting(&["capture"]);
        assert_eq!(restarts.request("capture", true), PluginRestart::Pending);
        assert_eq!(restarts.request("capture", false), PluginRestart::Pending);
        assert_eq!(restarts.waiting(), ["capture"]);
        restarts.begin("capture");
        assert_eq!(restarts.state("capture"), PluginRestart::Restarting);
        restarts.finish("capture");
        assert_eq!(restarts.state("capture"), PluginRestart::Idle);
        assert!(restarts.waiting().is_empty());
    }

    #[test]
    fn save_during_restart_follows() {
        let restarts = hosting(&["capture"]);
        assert_eq!(restarts.request("capture", false), PluginRestart::Restarting);
        restarts.begin("capture");
        restarts.request("capture", false);
        restarts.request("capture", false);
        assert!(restarts.waiting().is_empty());
        restarts.finish("capture");
        assert_eq!(restarts.waiting(), ["capture"]);
        restarts.begin("capture");
        restarts.finish("capture");
        assert_eq!(restarts.state("capture"), PluginRestart::Idle);
    }

    #[test]
    fn deferred_restart_reports_pending() {
        let restarts = hosting(&["capture"]);
        restarts.request("capture", false);
        restarts.defer("capture");
        assert_eq!(restarts.state("capture"), PluginRestart::Pending);
        assert_eq!(restarts.waiting(), ["capture"]);
    }

    #[test]
    fn unhosted_save_restarts_nothing() {
        let restarts = hosting(&["capture"]);
        assert_eq!(restarts.request("inactive", false), PluginRestart::Idle);
        assert!(restarts.waiting().is_empty());
    }
}
