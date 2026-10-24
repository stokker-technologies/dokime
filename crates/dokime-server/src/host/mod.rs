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
};

// Third Party Imports

// Local Imports

/* === Constants === */

// The run's settle budget plus the final checkpoint; the desktop waits ten seconds longer.
const STOP_BUDGET: std::time::Duration = std::time::Duration::from_secs(dokime::domain::STOP_SETTLEMENT_SECONDS + 20);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Session {
    executor: tokio::runtime::Handle,
    runtime: Option<dokime::Runtime>,
    server: Option<crate::grpc::Server>,
    stage: &'static str,
    plugins: Vec<PathBuf>,
    expected: Vec<ExpectedPlugin>,
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
            runtime: Some(runtime),
            server: None,
            stage: "starting",
            plugins: args.plugins.clone(),
            expected,
        };
        on_phase("plugins");
        let runtime = session.runtime.as_ref().expect("session owns runtime");
        if let Err(error) = verify_plugins(&runtime.handle(), if args.dev_control { &[] } else { &session.expected }).await {
            return match session.shutdown().await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(Error::StartupCleanup { source: Box::new(error), cleanup: Box::new(cleanup) }),
            };
        }
        on_phase("requests");
        let runtime = session.runtime.as_ref().expect("session owns runtime");
        match crate::grpc::Server::start(runtime.handle(), server_config).await {
            Ok(server) => {
                session.server = Some(server);
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
        self.server.as_ref().expect("ready session owns listener").local_endpoint()
    }

    pub fn address(&self) -> SocketAddr {
        self.server.as_ref().expect("ready session owns listener").address()
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
        let runtime = self.runtime.as_mut().expect("session owns runtime");
        runtime.reload_plugin(dokime::plugins::PluginBundle::new(path)).await?;
        verify_plugins(&runtime.handle(), &candidate).await?;
        if rerun { Ok(Some(rerun_latest_case(&runtime.handle(), &candidate[0].owners).await)) } else { Ok(None) }
    }

    pub async fn shutdown(mut self) -> Result<()> {
        match tokio::time::timeout(STOP_BUDGET, self.drain()).await {
            Ok(result) => result,
            Err(_) => {
                let stage = if self.stage == "stopping runtime" {
                    self.runtime.as_ref().map_or(self.stage, dokime::Runtime::shutdown_stage)
                } else {
                    self.stage
                };
                Err(Error::StopIncomplete { stage, budget_s: STOP_BUDGET.as_secs() })
            }
        }
    }

    async fn drain(&mut self) -> Result<()> {
        self.stage = "stopping listeners";
        let server_result = match self.server.take() {
            Some(server) => server.shutdown().await,
            None => Ok(()),
        };
        self.stage = "stopping runtime";
        let runtime_result = match self.runtime.as_mut() {
            Some(runtime) => runtime.shutdown().await.map(|_| ()),
            None => Ok(()),
        };
        let outcome = Error::shutdown(server_result, runtime_result);
        if outcome.is_ok() {
            self.runtime = None;
        }
        outcome
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let runtime = self.runtime.take();
        let server = self.server.take();
        if runtime.is_none() && server.is_none() {
            return;
        }
        let executor = self.executor.clone();
        self.executor.spawn(async move {
            let mut session = Session { executor, runtime, server, stage: "cancelled startup", plugins: Vec::new(), expected: Vec::new() };
            if let Err(error) = session.drain().await {
                tracing::error!(%error, "cancelled server session cleanup failed");
                // Do not recursively spawn cleanup on an unrecoverable drain failure.
                session.runtime = None;
                session.server = None;
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
        let replacement = reacquire(&args).await;
        assert!(tokio::net::TcpStream::connect(address).await.is_err(), "the dropped session releases its listener");
        replacement.shutdown().await.unwrap();
    }
}
