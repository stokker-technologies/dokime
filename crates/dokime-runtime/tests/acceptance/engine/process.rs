/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The process step built-in through `ctx.host()` with no host behind it, so
// each story launches a real `sh`: one run with its output in a produced
// artifact, and one spawned process cancelled once it reports readiness.

#![cfg(unix)]

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use dokime_runtime::harmos::operations::Host;

// Local Imports
use dokime_runtime::execution::log::Level;
use dokime_runtime::host::{ProcessExited, StepHandleCancelled};
use dokime_runtime::operations::process::{Output, RunProcess, RunProcessRequest};
use dokime_runtime::plugin::testing::Harness;
use dokime_runtime::prelude::*;

/* === Constants === */
const SUITE: &str = "process.checks";
const RUN: &str = "process.checks.run";
const CANCEL: &str = "process.checks.cancel";

// Readiness poll for the spawned shell's marker file: a step sees no other
// signal from a process it submitted.
const READY_POLL: Duration = Duration::from_millis(5);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Checks;

struct RunOnce;

struct SpawnCancel;

#[dokime_runtime::parameters]
struct CancelParameters {
    #[parameter(label = "Readiness Marker", default = "")]
    ready: String,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime_runtime::Error),
    #[error("host: {0}")]
    Host(#[from] dokime_runtime::harmos::sidecar::CallError),
    #[error("{0}")]
    Unexpected(String),
}

#[dokime_runtime::catalog]
enum Suites {
    Checks(Checks),
}

#[dokime_runtime::catalog]
enum Templates {
    RunOnce(RunOnce),
    SpawnCancel(SpawnCancel),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::suite(id = SUITE, label = "Process Built-In", templates = Templates)]
impl Checks {}

#[dokime_runtime::template(id = RUN, label = "Run Once")]
impl RunOnce {
    async fn initialize(_: &(), _ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Run the Tool")]
    async fn run(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let exit = ctx
            .host()
            .call_as::<RunProcess>(
                shell("printf 'alpha\\nbeta\\n'; echo \"template=$DOKIME_TEMPLATE_ID\" >&2")
                    .stdout(Output::Artifact("logs/tool.log".into())),
            )
            .await?
            .into_owned();
        if !exit.success() || exit.cancelled {
            return Err(Error::Unexpected(format!(
                "the tool exited {:?}",
                exit.exit_code
            )));
        }
        Ok(())
    }
}

#[dokime_runtime::template(id = CANCEL, label = "Spawn And Cancel", parameters = CancelParameters)]
impl SpawnCancel {
    async fn initialize(
        _: &CancelParameters,
        _ctx: &mut InitializationContext<'_>,
    ) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Spawn the Tool")]
    async fn spawn(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let ready = std::path::PathBuf::from(&ctx.parameters().ready);
        let script =
            shell("echo started; : > \"$1\"; exec sleep 30").args(["sh", ready.to_str().unwrap()]);
        let running = ctx.host().submit_as::<RunProcess>(script).await?;

        // The shell writes its marker after its first line, then waits on its
        // own lifetime until the cancellation ends it.
        while !ready.exists() {
            tokio::time::sleep(READY_POLL).await;
        }
        let exit = running.cancel_and_join().await?.into_owned();
        if !exit.cancelled {
            return Err(Error::Unexpected("the tool was not cancelled".into()));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn shell(script: &str) -> RunProcessRequest {
    RunProcessRequest::new("sh").args(["-c", script])
}

fn logged(report: &dokime_runtime::plugin::testing::Report, level: Level) -> Vec<String> {
    report
        .log()
        .iter()
        .filter(|entry| entry.level == level as i32)
        .map(|entry| entry.message.clone())
        .collect()
}

/* === Tests === */
// One shot: stdout streams into the produced artifact, stderr reaches the log
// at warning with the execution's identity, and the exit is recorded.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_output_artifact() {
    let report = Harness::template(RUN)
        .run::<Suites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(report.artifact("logs/tool.log").bytes, b"alpha\nbeta\n");
    assert_eq!(
        logged(&report, Level::Warn),
        vec![format!("template={RUN}")],
        "{}",
        report.diagnostics()
    );
    assert!(
        logged(&report, Level::Info)
            .iter()
            .all(|line| !line.contains("alpha")),
        "redirected output stays out of the log"
    );
    let exited = report.records::<ProcessExited>();
    assert_eq!(exited.len(), 1);
    assert_eq!(exited[0].program, "sh");
    assert_eq!(exited[0].exit_code, Some(0));
    assert!(!exited[0].cancelled);
}

// Submitted and cancelled: SIGTERM ends it, and the exit says so.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_cancel_exit() {
    let directory = tempfile::tempdir().unwrap();
    let ready = directory.path().join("ready");
    let report = Harness::template(CANCEL)
        .parameter("ready", ready.to_str().unwrap())
        .run::<Suites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert!(
        logged(&report, Level::Info).contains(&"started".to_owned()),
        "{}",
        report.diagnostics()
    );
    let exited = report.records::<ProcessExited>();
    assert_eq!(exited.len(), 1);
    assert!(exited[0].cancelled);
    assert_eq!(exited[0].exit_code, None, "a signal ended it");
    assert!(report.records::<StepHandleCancelled>().is_empty());
}
