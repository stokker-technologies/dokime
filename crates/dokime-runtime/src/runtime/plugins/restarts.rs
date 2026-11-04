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
use std::sync::{Arc, Mutex, PoisonError, RwLock};

// Third Party Imports
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

// Local Imports
use super::PluginHost;
use crate::RuntimeHandle;
use crate::domains::installed_plugins::PluginRestart;
use crate::runtime::BackgroundTask;
use dokime_domain::model::execution::result::{CyclePhase, CycleStatus};
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Which Programs And Bundles Wait To Restart, Shared By Requests And The Restarter.
pub(in crate::runtime) struct Restarts {
    phases: tokio::sync::watch::Sender<BTreeMap<Target, Phase>>,
    hosted: RwLock<Hosted>,

    // The installation each bundle was last asked to apply, by package.
    requested: Mutex<BTreeMap<String, Request>>,
}

// The Latest Change Asked Of A Bundle: The Installation To Apply, And Whether A Retry Asked For It.
#[derive(Clone, Copy)]
struct Request {
    install: Uuid,
    retry: bool,
}

// What Runs Now: Each Admitted Program's Package, And Each Package's Installation.
#[derive(Default)]
struct Hosted {
    programs: BTreeMap<String, String>,
    packages: BTreeMap<String, Option<Uuid>>,
}

/* === Enums === */

// One Thing The Restarter Brings In Line With What Is Saved.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) enum Target {
    // An installed bundle, by package: it starts, replaces or stops its programs as the store says.
    Bundle(String),

    // One admitted program restarts with its saved settings.
    Program(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    // Saved values wait for the restarter; `blocked` while a cycle is active.
    Waiting { blocked: bool },

    // A generation is starting; `again` when values were saved after it read them.
    Restarting { again: bool },
}

pub(in crate::runtime) enum Outcome {
    // The restart ran; its result is recorded like a startup's.
    Settled,

    // A cycle is active; the restarter tries again once it settles.
    Deferred(Option<CancellationToken>),

    // The program is not admitted; its saved values apply on the next start.
    Unhosted,
}

/* === Traits === */

/* === Implementations === */

impl Outcome {
    // Record The Outcome; A Deferred Restart Stays Waiting And Reports Pending.
    fn settle(self, restarts: &Restarts, target: &Target) -> Self {
        match &self {
            Self::Settled => restarts.finish(target),
            Self::Unhosted => restarts.forget(target),
            Self::Deferred(_) => restarts.defer(target),
        }
        self
    }
}

impl Default for Restarts {
    fn default() -> Self {
        Self {
            phases: tokio::sync::watch::Sender::new(BTreeMap::new()),
            hosted: RwLock::default(),
            requested: Mutex::default(),
        }
    }
}

impl Restarts {
    // Publish What Runs: Each Admitted Program With Its Package, And Each Package's Installation.
    pub(in crate::runtime) fn host<'a>(
        &self, programs: impl Iterator<Item = (&'a str, &'a str)>, packages: impl Iterator<Item = (&'a str, Option<Uuid>)>,
    ) {
        let hosted = Hosted {
            programs: programs.map(|(program, package)| (program.to_owned(), package.to_owned())).collect(),
            packages: packages.map(|(package, install)| (package.to_owned(), install)).collect(),
        };
        *self.hosted.write().unwrap_or_else(PoisonError::into_inner) = hosted;
    }

    // Further Saves Before The Restart Reads Its Values Join It; Later Ones Ask For One More.
    fn request(&self, target: Target, blocked: bool) -> PluginRestart {
        if let Target::Program(plugin) = &target
            && !self.hosted.read().unwrap_or_else(PoisonError::into_inner).programs.contains_key(plugin)
        {
            return PluginRestart::Idle;
        }

        self.phases.send_if_modified(|phases| match phases.get_mut(&target) {
            Some(Phase::Restarting { again }) => !std::mem::replace(again, true),
            Some(Phase::Waiting { .. }) => false,
            None => phases.insert(target.clone(), Phase::Waiting { blocked }).is_none(),
        });
        self.state(&target)
    }

    // A Store Change For `package`: The Restarter Applies `install`, The Latest One Asked For.
    fn request_bundle(&self, package: &str, install: Uuid, retry: bool, blocked: bool) -> PluginRestart {
        self.requested.lock().unwrap_or_else(PoisonError::into_inner).insert(package.to_owned(), Request { install, retry });
        let state = self.request(Target::Bundle(package.to_owned()), blocked);
        // A Retry Joining A Waiting Change Is Weighed Again At Once: It May Start Without Waiting.
        if retry {
            self.phases.send_modify(|_| {});
        }
        state
    }

    fn requested(&self, package: &str) -> Option<Uuid> {
        self.requested.lock().unwrap_or_else(PoisonError::into_inner).get(package).map(|request| request.install)
    }

    // Whether The Latest Change Asked Of `package` Is A Retry.
    fn retrying(&self, package: &str) -> bool {
        self.requested.lock().unwrap_or_else(PoisonError::into_inner).get(package).is_some_and(|request| request.retry)
    }

    fn state(&self, target: &Target) -> PluginRestart {
        match self.phases.borrow().get(target) {
            None => PluginRestart::Idle,
            Some(Phase::Waiting { blocked: true }) => PluginRestart::Pending,
            Some(Phase::Waiting { blocked: false } | Phase::Restarting { .. }) => PluginRestart::Restarting,
        }
    }

    // A Program Also Waits While Its Bundle Does; Restarting Outranks Pending.
    fn program_state(&self, plugin: &str) -> PluginRestart {
        let own = self.state(&Target::Program(plugin.to_owned()));
        let package = self.hosted.read().unwrap_or_else(PoisonError::into_inner).programs.get(plugin).cloned();
        let bundle = package.map_or(PluginRestart::Idle, |package| self.state(&Target::Bundle(package)));
        match (own, bundle) {
            (PluginRestart::Restarting, _) | (_, PluginRestart::Restarting) => PluginRestart::Restarting,
            (PluginRestart::Pending, _) | (_, PluginRestart::Pending) => PluginRestart::Pending,
            _ => PluginRestart::Idle,
        }
    }

    // The Programs `package` Runs Now.
    fn programs(&self, package: &str) -> BTreeSet<String> {
        let hosted = self.hosted.read().unwrap_or_else(PoisonError::into_inner);
        hosted.programs.iter().filter(|(_, owner)| *owner == package).map(|(program, _)| program.clone()).collect()
    }

    // `Some` While `package` Runs, With The Installation It Runs.
    fn running(&self, package: &str) -> Option<Option<Uuid>> {
        self.hosted.read().unwrap_or_else(PoisonError::into_inner).packages.get(package).copied()
    }

    // Bundles Come First: A Bundle's New Generation Already Prepares With The Saved Values.
    fn waiting(&self) -> Vec<Target> {
        let phases = self.phases.borrow();
        phases.iter().filter(|(_, phase)| matches!(phase, Phase::Waiting { .. })).map(|(target, _)| target.clone()).collect()
    }

    // A Deferred Restart Reads Its Values Again When It Runs, So Saves Meanwhile Join It.
    fn defer(&self, target: &Target) {
        self.phases.send_if_modified(|phases| {
            let Some(phase) = phases.get_mut(target) else { return false };
            let deferred = Phase::Waiting { blocked: true };
            if *phase == deferred {
                return false;
            }
            *phase = deferred;
            true
        });
    }

    pub(in crate::runtime) fn begin(&self, target: &Target) {
        self.phases.send_if_modified(|phases| {
            let Some(phase @ Phase::Waiting { .. }) = phases.get_mut(target) else { return false };
            *phase = Phase::Restarting { again: false };
            true
        });
    }

    fn finish(&self, target: &Target) {
        self.phases.send_if_modified(|phases| {
            let Some(phase) = phases.get_mut(target) else { return false };
            if *phase == (Phase::Restarting { again: true }) {
                *phase = Phase::Waiting { blocked: false };
            } else {
                phases.remove(target);
            }
            true
        });
    }

    fn forget(&self, target: &Target) {
        self.phases.send_if_modified(|phases| phases.remove(target).is_some());
    }

    async fn settled(&self, target: Target) {
        let mut phases = self.phases.subscribe();
        let _ = phases.wait_for(|phases| !phases.contains_key(&target)).await;
    }
}

impl RuntimeHandle {
    // A Save That Changed Values: Restart The Program Now, Or Once No Cycle Is Active.
    pub(crate) fn request_plugin_restart(&self, plugin: &str) -> PluginRestart {
        let blocked = self.active_cycle().is_some();
        self.plugin_restarts.request(Target::Program(plugin.to_owned()), blocked)
    }

    // A Store Change For `package`: Apply `install` Now, Or Once No Cycle Is Active.
    // A retry of a package that does not run starts it at once.
    pub(crate) fn request_plugin_apply(&self, package: &str, install: Uuid, retry: bool) -> PluginRestart {
        let starts_only = retry && self.plugin_restarts.running(package).is_none();
        let blocked = !starts_only && self.active_cycle().is_some();
        self.plugin_restarts.request_bundle(package, install, retry, blocked)
    }

    // The Programs `package` Runs When It Runs `install` Now.
    pub(crate) fn plugin_runs(&self, package: &str, install: Uuid) -> Option<BTreeSet<String>> {
        (self.plugin_restarts.running(package) == Some(Some(install))).then(|| self.plugin_restarts.programs(package))
    }

    // The Package That Runs `install` Now.
    pub(crate) fn running_package(&self, install: Uuid) -> Option<String> {
        let hosted = self.plugin_restarts.hosted.read().unwrap_or_else(PoisonError::into_inner);
        hosted.packages.iter().find(|(_, running)| **running == Some(install)).map(|(package, _)| package.clone())
    }

    pub(crate) fn plugin_restart(&self, plugin: &str) -> PluginRestart {
        self.plugin_restarts.program_state(plugin)
    }

    pub(crate) fn plugin_bundle_restart(&self, package: &str) -> PluginRestart {
        self.plugin_restarts.state(&Target::Bundle(package.to_owned()))
    }

    // Resolves Once `plugin` Runs With Every Value Saved So Far, Or Its Failed Restart Is Recorded.
    pub async fn plugin_restart_settled(&self, plugin: &Identifier) {
        self.plugin_restarts.settled(Target::Program(plugin.as_str().to_owned())).await;
    }

    // Resolves Once `package` Runs What The Store Says, Or Its Refused Change Is Recorded.
    pub async fn plugin_bundle_settled(&self, package: &str) {
        self.plugin_restarts.settled(Target::Bundle(package.to_owned())).await;
    }

    // Under Plugin Admission: Refuse While Any Cycle Is Active, Else Close The Affected Programs'
    // Admission. A retry that only starts programs which do not run is `starts_only` and never waits.
    pub(in crate::runtime) async fn admit_plugin_restart(
        &self, affected: &[String], starts_only: bool,
    ) -> Result<(), Option<CancellationToken>> {
        let _admission = self.plugin_admission.lock().await;
        if !starts_only && let Some(settling) = self.active_cycle() {
            return Err(settling);
        }

        self.unavailable_plugins.write().expect("plugin availability lock poisoned").extend(affected.iter().cloned());
        Ok(())
    }

    // `Some` While Any Cycle Is Active: Running, Paused Or Stopping, Sealing, Or Settling A Run.
    //
    // Every execution runs in a cycle, so this covers every running or prepared
    // execution too. A provider run settles after its cycle returns to Ready and
    // starts the cycle's next case before it does, so the gap between two cases
    // counts. The inner value is such a run, which the restarter can wait on.
    pub(in crate::runtime) fn active_cycle(&self) -> Option<Option<CancellationToken>> {
        let cycling = self.state_read().cycles.values().any(|cycle| {
            matches!(cycle.lifecycle, CycleStatus::Running | CycleStatus::Paused | CycleStatus::Stopping)
                || cycle.phase == CyclePhase::Sealing
        });
        let settling = {
            let tasks = self.provider_tasks.lock().expect("provider task lock poisoned");
            tasks.iter().find(|task| !task.settled.is_cancelled()).map(|task| task.settled.clone())
        };
        (cycling || settling.is_some()).then_some(settling)
    }
}

/* === Public Functions === */

/* === Private Functions === */

// One Restarter Per Runtime: Saves Queue Programs, Store Changes Queue Bundles; Each Applies Once No Cycle Is Active.
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

// One Pass Over Waiting Bundles And Programs: Whether Any Stayed Blocked, And A Settling Provider To Wait On.
async fn restart_waiting(
    host: &tokio::sync::Mutex<PluginHost>, restarts: &Restarts, stopping: &tokio::sync::watch::Receiver<bool>,
) -> Option<(bool, Option<CancellationToken>)> {
    let mut blocked = false;
    let mut settling = None;
    for target in restarts.waiting() {
        let mut host = host.lock().await;
        if *stopping.borrow() {
            return None;
        }

        let outcome = match &target {
            Target::Bundle(package) => host.apply_bundle(package, restarts.requested(package), restarts.retrying(package)).await,
            Target::Program(plugin) => host.restart_with_saved_settings(plugin).await,
        };
        let Outcome::Deferred(provider) = outcome.settle(restarts, &target) else { continue };
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

    fn program(plugin: &str) -> Target {
        Target::Program(plugin.to_owned())
    }

    fn hosting(plugins: &[&str]) -> Restarts {
        let restarts = Restarts::default();
        restarts.host(plugins.iter().map(|plugin| (*plugin, "capture-fixture")), std::iter::once(("capture-fixture", None)));
        restarts
    }

    #[test]
    fn repeated_saves_wait_once() {
        let restarts = hosting(&["capture"]);
        assert_eq!(restarts.request(program("capture"), true), PluginRestart::Pending);
        assert_eq!(restarts.request(program("capture"), false), PluginRestart::Pending);
        assert_eq!(restarts.waiting(), [program("capture")]);
        restarts.begin(&program("capture"));
        assert_eq!(restarts.state(&program("capture")), PluginRestart::Restarting);
        restarts.finish(&program("capture"));
        assert_eq!(restarts.state(&program("capture")), PluginRestart::Idle);
        assert!(restarts.waiting().is_empty());
    }

    #[test]
    fn save_during_restart_follows() {
        let restarts = hosting(&["capture"]);
        assert_eq!(restarts.request(program("capture"), false), PluginRestart::Restarting);
        restarts.begin(&program("capture"));
        restarts.request(program("capture"), false);
        restarts.request(program("capture"), false);
        assert!(restarts.waiting().is_empty());
        restarts.finish(&program("capture"));
        assert_eq!(restarts.waiting(), [program("capture")]);
        restarts.begin(&program("capture"));
        restarts.finish(&program("capture"));
        assert_eq!(restarts.state(&program("capture")), PluginRestart::Idle);
    }

    #[test]
    fn deferred_restart_reports_pending() {
        let restarts = hosting(&["capture"]);
        restarts.request(program("capture"), false);
        restarts.defer(&program("capture"));
        assert_eq!(restarts.state(&program("capture")), PluginRestart::Pending);
        assert_eq!(restarts.waiting(), [program("capture")]);
    }

    #[test]
    fn unhosted_save_restarts_nothing() {
        let restarts = hosting(&["capture"]);
        assert_eq!(restarts.request(program("inactive"), false), PluginRestart::Idle);
        assert!(restarts.waiting().is_empty());
    }

    // A Bundle Applies The Latest Installation Asked For, Ahead Of Program Restarts, And Its Programs Report It.
    #[test]
    fn bundle_change_coalesces() {
        let restarts = hosting(&["capture"]);
        let bundle = Target::Bundle("capture-fixture".into());
        let (first, second) = (Uuid::now_v7(), Uuid::now_v7());
        restarts.request(program("capture"), true);
        assert_eq!(restarts.request_bundle("capture-fixture", first, true, true), PluginRestart::Pending);
        assert_eq!(restarts.request_bundle("capture-fixture", second, false, false), PluginRestart::Pending);
        assert_eq!(restarts.requested("capture-fixture"), Some(second));
        assert!(!restarts.retrying("capture-fixture"), "the latest request decides whether a retry asked for it");
        assert_eq!(restarts.waiting(), [bundle.clone(), program("capture")]);
        restarts.begin(&bundle);
        assert_eq!(restarts.program_state("capture"), PluginRestart::Restarting);
        restarts.finish(&bundle);
        assert_eq!(restarts.program_state("capture"), PluginRestart::Pending);
    }
}
