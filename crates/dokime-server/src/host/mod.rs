/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod args;
mod error;
mod hub_log;
mod logging;
mod network;
pub mod version;

#[cfg(debug_assertions)]
mod profiles;

/* === Re-Exports === */

pub use args::{Args, NetworkArgs};
pub use error::{Error, Result};

// Applications install tracing once and retain its guards for their process.
pub use logging::{LoggingGuards, init as install_logging};

/* === Imports === */

// Standard Library Imports
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

// Third Party Imports
use futures_util::future::BoxFuture;

// Local Imports

/* === Constants === */

// The run's settle budget plus the final checkpoint; the desktop waits ten seconds longer.
const STOP_BUDGET: std::time::Duration = std::time::Duration::from_secs(dokime::domain::STOP_SETTLEMENT_SECONDS + 20);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Session {
    executor: tokio::runtime::Handle,
    resources: Option<Resources>,
    draining: Option<tokio::task::JoinHandle<(Resources, Result<()>)>>,
    stage: Arc<Mutex<&'static str>>,
    task_failure: Option<String>,
    plugins: Vec<PathBuf>,
    expected: Vec<ExpectedPlugin>,
}

struct Resources {
    runtime: Option<dokime::Runtime>,
    server: Option<crate::grpc::Server>,
}

struct ExpectedPlugin {
    package: String,
    digest: String,
    owners: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Session {
    pub async fn start(
        args: &Args, mut config: dokime::config::Config, server_config: crate::grpc::Config, mut on_phase: impl FnMut(&str) + Send,
    ) -> Result<Self> {
        let config_root = config.output_root.clone();
        if let Some(title) = &args.title {
            config.title = title.clone();
        }
        config.label = args.label.clone();
        config.seed_plugins = !args.no_seed_validation_artifact;
        config.explicit_plugins = if args.dev_control { Vec::new() } else { args.plugins.clone() };
        let expected = expected_plugins(&args.plugins)?;
        on_phase("storage");
        let runtime = dokime::app::launch(config).await.map_err(|error| Error::launch(error, config_root.clone()))?;
        let mut session = Self {
            executor: tokio::runtime::Handle::current(),
            resources: Some(Resources { runtime: Some(runtime), server: None }),
            draining: None,
            stage: Arc::new(Mutex::new("starting")),
            task_failure: None,
            plugins: args.plugins.clone(),
            expected,
        };
        on_phase("plugins");
        let runtime = session.resources.as_ref().unwrap().runtime.as_ref().expect("session owns runtime");
        if let Err(error) = verify_plugins(&runtime.handle(), if args.dev_control { &[] } else { &session.expected }).await {
            return match session.shutdown().await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(Error::StartupCleanup { source: Box::new(error), cleanup: Box::new(cleanup) }),
            };
        }
        on_phase("requests");
        let runtime = session.resources.as_ref().unwrap().runtime.as_ref().expect("session owns runtime");
        match crate::grpc::Server::start(runtime.handle(), server_config).await {
            Ok(server) => {
                session.resources.as_mut().unwrap().server = Some(server);
                Ok(session)
            }
            Err(error) => {
                let cleanup = session.shutdown().await;
                match cleanup {
                    Ok(()) => Err(error.into()),
                    Err(cleanup) => Err(Error::StartupCleanup { source: Box::new(error.into()), cleanup: Box::new(cleanup) }),
                }
            }
        }
    }

    pub fn endpoint(&self) -> String {
        self.resources.as_ref().and_then(|resources| resources.server.as_ref()).expect("ready session owns listener").local_endpoint()
    }

    pub fn address(&self) -> SocketAddr {
        self.resources.as_ref().and_then(|resources| resources.server.as_ref()).expect("ready session owns listener").address()
    }

    pub async fn reload(&mut self, path: &Path, rerun: bool) -> Result<Option<String>> {
        let path = path.canonicalize()?;
        let index = self
            .plugins
            .iter()
            .position(|allowed| allowed.canonicalize().ok().as_ref() == Some(&path))
            .ok_or_else(|| Error::Input("bundle path was not selected for this development session".into()))?;
        let candidate = expected_plugins(std::slice::from_ref(&path))?;
        if candidate[0].package != self.expected[index].package {
            return Err(Error::Input("package name changed; restart the development command".into()));
        }
        let runtime = self.resources.as_mut().and_then(|resources| resources.runtime.as_mut()).expect("session owns runtime");
        runtime.reload_plugin(dokime::plugins::PluginBundle::new(path)).await?;
        verify_plugins(&runtime.handle(), &candidate).await?;
        if rerun { Ok(Some(rerun_latest_case(&runtime.handle(), &candidate[0].owners).await)) } else { Ok(None) }
    }

    pub fn shutdown(&mut self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            if let Some(message) = &self.task_failure {
                return Err(Error::Setup(message.clone()));
            }
            if self.draining.is_none() {
                let Some(mut resources) = self.resources.take() else { return Ok(()) };
                let stage = self.stage.clone();
                self.draining = Some(self.executor.spawn(async move {
                    let outcome = resources.drain(&stage).await;
                    (resources, outcome)
                }));
            }
            let waiting = self.draining.as_mut().expect("one retained drain");
            let result = match tokio::time::timeout(STOP_BUDGET, waiting).await {
                Err(_) => {
                    let stage = *self.stage.lock().expect("shutdown stage poisoned");
                    return Err(Error::StopIncomplete { stage, budget_s: STOP_BUDGET.as_secs() });
                }
                Ok(result) => result,
            };
            self.draining = None;
            match result {
                Ok((resources, outcome)) => {
                    if outcome.is_err() {
                        self.resources = Some(resources);
                    }
                    outcome
                }
                Err(error) => {
                    let message = format!("server shutdown task failed: {error}");
                    self.task_failure = Some(message.clone());
                    Err(Error::Setup(message))
                }
            }
        })
    }
}

impl Resources {
    async fn drain(&mut self, stage: &Mutex<&'static str>) -> Result<()> {
        *stage.lock().expect("shutdown stage poisoned") = "stopping listeners";
        let server_result = match self.server.as_mut() {
            Some(server) => server.drain().await,
            None => Ok(()),
        };
        if server_result.is_ok() {
            self.server = None;
        }
        *stage.lock().expect("shutdown stage poisoned") = "stopping runtime";
        let runtime_result = match self.runtime.as_mut() {
            Some(runtime) => runtime.shutdown().await.map(|_| ()),
            None => Ok(()),
        };
        if runtime_result.is_ok() {
            // Runtime Completion Includes Every Registered gRPC Generation And Storage.
            // A Prior Detach Failure Is Resolved Only By That Authoritative Completion.
            self.server = None;
            self.runtime = None;
        }
        Error::shutdown(server_result, runtime_result)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let resources = self.resources.take();
        let draining = self.draining.take();
        if resources.is_none() && draining.is_none() {
            return;
        }
        let stage = self.stage.clone();
        self.executor.spawn(async move {
            if let Some(draining) = draining {
                if let Ok((_, Err(error))) = draining.await {
                    tracing::error!(%error, "server session cleanup failed");
                }
            } else if let Some(mut resources) = resources
                && let Err(error) = resources.drain(&stage).await
            {
                tracing::error!(%error, "cancelled server session cleanup failed");
            }
        });
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn rerun_latest_case(handle: &dokime::RuntimeHandle, owners: &[String]) -> String {
    let cycles = handle.cycles().await;
    let latest = cycles
        .iter()
        .flat_map(|cycle| {
            cycle
                .test_executions
                .iter()
                .filter(|execution| {
                    execution.snapshot.plugin().is_some_and(|plugin| owners.contains(&plugin.source))
                        && cycle.cases().any(|case| case.test_case_id == execution.test_case_id)
                })
                .map(move |execution| (cycle, execution))
        })
        .max_by_key(|(_, execution)| execution.started_at.as_ref());
    let Some((cycle, execution)) = latest else {
        return "Run a case in the desktop GUI to select it for automatic re-run.".into();
    };
    let execution_id: dokime::TestExecutionId = dokime::model::identity::new_id();
    let binding = (execution_id.clone(), execution.test_case_id.clone());
    if let Err(error) = handle.append_executions(cycle.test_cycle_id.clone(), vec![binding]).await {
        return format!("Automatic re-run unavailable: {error}");
    }
    let receipt = handle.start_execution(execution_id, execution.mode, Default::default()).await;
    match receipt.rejection {
        Some(error) => format!("Automatic re-run refused: {}", error.message),
        None => "Re-running the latest case with its retained inputs.".into(),
    }
}

fn expected_plugins(paths: &[PathBuf]) -> crate::host::Result<Vec<ExpectedPlugin>> {
    use sha2::{Digest, Sha256};
    paths
        .iter()
        .map(|path| {
            // Match the runtime's bundle input bound before allocation.
            use std::io::Read;
            const MAX_BYTES: u64 = 272 * 1024 * 1024;
            if !std::fs::symlink_metadata(path)?.file_type().is_file() {
                return Err(crate::host::Error::Input("plugin bundle must be a regular file".into()));
            }
            let mut bytes = Vec::new();
            std::fs::File::open(path)?.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_BYTES {
                return Err(crate::host::Error::Input(format!("plugin bundle too large: {}", path.display())));
            }
            let (_, metadata) = dokime::plugins::inspect(&bytes).map_err(|error| crate::host::Error::Input(error.to_string()))?;
            Ok(ExpectedPlugin {
                package: metadata.identity.name,
                digest: Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                owners: metadata.owners,
            })
        })
        .collect()
}

async fn verify_plugins(handle: &dokime::RuntimeHandle, expected: &[ExpectedPlugin]) -> crate::host::Result<()> {
    use dokime::runtime::query::{RuntimeResolution, RuntimeViewPayload, RuntimeViewQuery, RuntimeViewTopic, RuntimeWindow};
    if expected.is_empty() {
        return Ok(());
    }
    let frame = handle
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::InstalledPlugins,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await?;
    let RuntimeViewPayload::InstalledPlugins { frame } = frame.payload else {
        return Err(crate::host::Error::Input("runtime returned an unexpected admission view".into()));
    };
    for plugin in expected {
        for owner in &plugin.owners {
            let admitted = frame.installed_plugins.iter().any(|artifact| {
                artifact.enabled
                    && artifact.plugin_id.as_str() == owner
                    && matches!(&artifact.owner,
                        Some(dokime::state::PluginOwner::Bundle { package, content_digest, installation: None, .. })
                        if package == &plugin.package && content_digest == &plugin.digest)
            });
            if !admitted {
                return Err(crate::host::Error::Input(format!(
                    "plugin {} owner {owner} was not admitted; see runtime diagnostics",
                    plugin.package
                )));
            }
        }
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;

    fn arguments(root: &Path) -> Args {
        Args {
            shutdown_stdin: false,
            dev_control: false,
            network: NetworkArgs {
                host: std::net::Ipv4Addr::LOCALHOST.into(),
                port: 0,
                message_limit_mib: 2,
                grpc_web: false,
                allowed_origins: Vec::new(),
            },
            title: None,
            label: None,
            output_root: Some(root.join("store")),
            application_root: Some(root.join("application")),
            env_file: None,
            plugins: Vec::new(),
            plugins_root: None,
            no_seed_validation_artifact: true,
        }
    }

    async fn reacquire(args: &Args) -> Session {
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                match Session::start(args, args.configuration().unwrap(), args.network.config().unwrap(), |_| {}).await {
                    Ok(session) => break session,
                    Err(_) => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
                }
            }
        })
        .await
        .expect("cancelled startup releases its execution store")
    }

    #[tokio::test]
    async fn startup_unwind_drains() {
        let root = tempfile::tempdir().unwrap();
        let args = arguments(root.path());
        let startup = Session::start(&args, args.configuration().unwrap(), args.network.config().unwrap(), |phase| {
            if phase == "plugins" {
                panic!("deliberate startup cancellation");
            }
        });
        assert!(std::panic::AssertUnwindSafe(startup).catch_unwind().await.is_err());
        reacquire(&args).await.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn session_drop_drains() {
        let root = tempfile::tempdir().unwrap();
        let args = arguments(root.path());
        let session = Session::start(&args, args.configuration().unwrap(), args.network.config().unwrap(), |_| {}).await.unwrap();
        let address = session.address();
        drop(session);
        let mut replacement = reacquire(&args).await;
        assert!(tokio::net::TcpStream::connect(address).await.is_err(), "the dropped session releases its listener");
        replacement.shutdown().await.unwrap();
    }
}
