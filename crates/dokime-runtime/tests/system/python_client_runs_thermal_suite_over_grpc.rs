/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Python Client Runs a Thermal Suite over gRPC

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env::{self, consts::EXE_SUFFIX},
    error::Error,
    path::PathBuf,
    process::Stdio,
    sync::Arc,
    time::{Duration, SystemTime},
};

// Third Party Imports
use harmos_runtime::clock::{self, Controller, Reading, Sleep, Source};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::{self, Instant},
};
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime_client_internal::Client;
use dokime_protocol::host::{ResultStatus, execution_outcome};
use dokime_runtime::process;
use dokime_runtime::{ControlledClock, SchedulerClock, services};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct PromptClock {
    scheduler: Arc<ControlledClock>,
    execution: clock::Clock,
}

struct PromptTime {
    scheduler: Arc<ControlledClock>,
    origin: Instant,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[async_trait::async_trait]
impl SchedulerClock for PromptClock {
    fn now_ns(&self) -> i64 {
        self.scheduler.now_ns()
    }

    async fn wait_until(&self, at_ns: i64) -> dokime_runtime::Result<()> {
        self.scheduler.wait_until(at_ns).await
    }

    async fn reached(&self, at_ns: i64) {
        self.scheduler.reached(at_ns).await;
    }

    fn execution_clock(&self) -> (clock::Clock, Option<Controller>) {
        // The Story Owns Advancement; Dispatch Must Not Advance a Deadline Itself.
        (self.execution.clone(), None)
    }
}

impl Source for PromptTime {
    fn read(&self) -> Reading {
        let elapsed = Duration::from_nanos(self.scheduler.now_ns() as u64);
        Reading {
            instant: self.origin + elapsed,
            date: SystemTime::UNIX_EPOCH + elapsed,
        }
    }

    fn sleep_until(&self, deadline: Instant) -> Sleep {
        let scheduler = self.scheduler.clone();
        let at_ns = deadline.duration_since(self.origin).as_nanos() as i64;
        Box::pin(async move { scheduler.reached(at_ns).await })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn unanswered_python_starter_prompt_survives_operation_deadlines() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Start a Host With an Explicitly Advanced Execution Clock.
            let root = super::support::tempdir()?;
            let scheduler = Arc::new(ControlledClock::new(0));
            let host_clock = Arc::new(ControlledClock::new(0));
            let execution = clock::Clock::with_clock(PromptTime {
                scheduler: scheduler.clone(),
                origin: Instant::now(),
            });
            let mut settings = super::support::settings();
            settings.clock = Arc::new(PromptClock { scheduler: host_clock.clone(), execution });
            let server =
                services::serve("127.0.0.1:0".parse()?, &root.path().join("store"), settings, super::support::service_settings()).await?;
            let endpoint = format!("http://{}", server.endpoint());
            let channel = Endpoint::from_shared(endpoint.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;

            // Run the Shipped Client, Including Its Plugin Listener and Progress Observer.
            let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            let directory = if EXE_SUFFIX.is_empty() { ".venv/bin" } else { ".venv/Scripts" };
            let python = repository.join(directory).join(format!("python{EXE_SUFFIX}"));
            let mut command = process::command(&python);
            command
                .args(["-u", "-m", "starter_project"])
                .current_dir(root.path())
                .env("DOKIME_REMOTE", &endpoint)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit());
            let mut child = Command::from(command).kill_on_drop(true).spawn()?;
            let mut output = BufReader::new(child.stdout.take().ok_or("Run Starter: Capture Its Output")?).lines();
            while let Some(line) = output.next_line().await? {
                if line.trim() == "prompt" {
                    break;
                }
            }
            let pending = client.prompts.list("", true).await?;
            assert_eq!(pending.prompts.len(), 1);
            let prompt = pending.prompts[0].prompt.as_ref().ok_or("Observe Starter: Read Its Pending Prompt")?;
            let prompt_id = prompt.prompt_id.clone();
            let execution_id =
                prompt.scope.as_ref().and_then(|scope| scope.execution_id.clone()).ok_or("Observe Starter: Read Its Execution Scope")?;

            // Cross Every Default Execution Deadline Without Answering or Waiting on Wall Time.
            scheduler.wait_until(7_200_000_000_000).await?;
            host_clock.wait_until(31_000_000_000).await?;
            let pending = client.prompts.get(prompt_id.clone()).await?.prompt.ok_or("Observe Starter: Retain Its Prompt")?;
            assert!(pending.response.is_none());
            let mut progress = client.executions.progress.watch(execution_id.clone()).await?;
            let state = progress.next().await?.ok_or("Observe Starter: Retain Its Progress")?;
            assert_eq!(state.phase, "prompt");
            assert!(!state.sealed);
            assert!(child.try_wait()?.is_none());
            drop(progress);

            // Answer and Let the Original Client Seal and Save Its Evidence.
            client.prompts.answers.create(prompt_id, "0", Default::default(), true).await?;
            while output.next_line().await?.is_some() {}
            assert!(child.wait().await?.success());
            let outcome = client.executions.get(execution_id).await?.execution.ok_or("Read Starter: Retain Its Execution")?;
            assert!(!outcome.seal.is_empty());
            assert!(
                matches!(outcome.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()),
                Some(execution_outcome::Attribution::Evaluated(outcome)) if outcome.status == ResultStatus::Passed as i32),
                "{outcome:?}"
            );
            let saved = root.path().join("_output").read_dir()?.collect::<Result<Vec<_>, _>>()?;
            assert_eq!(saved.len(), 1);
            server.close().await?;
            Ok(())
        })
        .await?
    }))
}

#[test]
fn python_client_runs_thermal_suite_over_grpc() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Prepare the Installed Pure SDK and Launch the Real Plugin and Host
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            let python = env::var_os("DOKIME_PYTHON").map(PathBuf::from).unwrap_or_else(|| {
                let directory = if EXE_SUFFIX.is_empty() { ".venv/bin" } else { ".venv/Scripts" };
                repository.join(directory).join(format!("python{}", EXE_SUFFIX))
            });
            let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/system/client_thermal.py");
            let expected = root.path().join("expected.bin");
            let process = HostProcess::start_with_environment(&root.path().join("store"), &credentials.environment).await?;

            // Drive the Complete Journey with the Pure Python Package's Generated Clients
            let output = Command::from(process::command(&python))
                .arg(&script)
                .args(["run", &process.url])
                .arg(&expected)
                .kill_on_drop(true)
                .output()
                .await?;
            assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            process.close().await?;

            // Reconnect After a Real Process Restart and Compare All Stored Evidence
            let process = HostProcess::start_with_environment(&root.path().join("store"), &credentials.environment).await?;
            let output = Command::from(process::command(&python))
                .arg(&script)
                .args(["read", &process.url])
                .arg(&expected)
                .kill_on_drop(true)
                .output()
                .await?;
            assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            process.close().await?;
            Ok(())
        })
        .await?
    }))
}
