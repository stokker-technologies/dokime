/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Run the frontend's prepared commands under the configured native development owner.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    future::poll_fn,
    path::PathBuf,
    process::{ExitStatus, Stdio},
    task::Poll,
    time::Duration,
};

// Third Party Imports
use serde::Deserialize;
use tokio::process::Command;

// Local Imports
use crate::error::Error;
use dokime_packaging::process::Child;

/* === Constants === */

const PROCESS_STOP_MS: u64 = 5_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub struct Args {
    #[arg(long)]
    plan: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    cwd: PathBuf,
    commands: Vec<Process>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Process {
    arguments: Vec<String>,
    environment: BTreeMap<String, String>,
    remove_environment: Vec<String>,
}

struct Interrupts {
    #[cfg(unix)]
    interrupt: tokio::signal::unix::Signal,
    #[cfg(unix)]
    terminate: tokio::signal::unix::Signal,
    #[cfg(unix)]
    hangup: tokio::signal::unix::Signal,
}

/* === Enums === */

enum Completion {
    Child(usize, ExitStatus),
    Signal(i32),
}

/* === Traits === */

/* === Implementations === */

impl Interrupts {
    fn open() -> std::io::Result<Self> {
        Ok(Self {
            #[cfg(unix)]
            interrupt: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?,
            #[cfg(unix)]
            terminate: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?,
            #[cfg(unix)]
            hangup: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?,
        })
    }

    async fn received(&mut self) -> std::io::Result<i32> {
        #[cfg(unix)]
        {
            use tokio::signal::unix::SignalKind;
            Ok(tokio::select! {
                _ = self.interrupt.recv() => SignalKind::interrupt().as_raw_value(),
                _ = self.terminate.recv() => SignalKind::terminate().as_raw_value(),
                _ = self.hangup.recv() => SignalKind::hangup().as_raw_value(),
            })
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await?;
            Ok(2)
        }
    }
}

/* === Public Functions === */

pub async fn run(args: Args) -> Result<(), Error> {
    let plan: Plan = serde_json::from_str(&args.plan)?;
    if plan.commands.is_empty() || plan.commands.iter().any(|command| command.arguments.is_empty()) {
        return Err(Error::Invalid("desktop development requires a nonempty prepared command".into()));
    }
    let mut interrupts = Interrupts::open()?;
    let mut children = Vec::with_capacity(plan.commands.len());

    // Keep every already-started tree owned if another command cannot start.
    let started = start(&plan, &mut children);
    if let Err(error) = started {
        settle(&mut children, termination_signal()).await?;
        return Err(error);
    }
    let completion = tokio::select! {
        result = first_exit(&mut children) => result.map(|(index, status)| Completion::Child(index, status)),
        result = interrupts.received() => result.map(Completion::Signal),
    };
    let signal = match &completion {
        Ok(Completion::Signal(signal)) => *signal,
        _ => termination_signal(),
    };
    settle(&mut children, signal).await?;

    let code = match completion? {
        Completion::Signal(signal) => 128 + signal,
        Completion::Child(index, status) => {
            if index + 1 != children.len() && status.success() {
                1
            } else {
                exit_code(status)
            }
        }
    };
    if code == 0 { Ok(()) } else { Err(Error::DevelopmentExit(code)) }
}

/* === Private Functions === */

fn start(plan: &Plan, children: &mut Vec<Child>) -> Result<(), Error> {
    for process in &plan.commands {
        let mut command = Command::new(&process.arguments[0]);
        // Match the existing Python CREATE_NO_WINDOW launch on Windows.
        #[cfg(windows)]
        command.creation_flags(0x0800_0000);
        command.args(&process.arguments[1..]).current_dir(&plan.cwd);
        command.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
        for name in &process.remove_environment {
            command.env_remove(name);
        }
        command.envs(&process.environment);
        children.push(dokime_packaging::process::spawn_session(command)?);
    }
    Ok(())
}

async fn first_exit(children: &mut [Child]) -> std::io::Result<(usize, ExitStatus)> {
    let mut waiting = children.iter_mut().map(Child::wait_leader).collect::<Vec<_>>();
    poll_fn(|context| {
        for (index, wait) in waiting.iter_mut().enumerate() {
            if let Poll::Ready(status) = wait.as_mut().poll(context) {
                return Poll::Ready(status.map(|status| (index, status)));
            }
        }
        Poll::Pending
    })
    .await
}

async fn settle(children: &mut [Child], signal: i32) -> Result<(), Error> {
    let grace = Duration::from_millis(PROCESS_STOP_MS);
    #[cfg(unix)]
    {
        let deadline = tokio::time::Instant::now() + grace;
        for child in children.iter_mut() {
            child.signal(signal)?;
        }
        let _ = tokio::time::timeout_at(deadline, leaders_exited(children)).await;
        let groups_exist = children.iter_mut().try_fold(false, |alive, child| child.group_exists().map(|exists| alive || exists))?;
        if groups_exist {
            tokio::time::sleep_until(deadline).await;
        }
    }
    #[cfg(not(unix))]
    let _ = signal;

    // Force cleanup also when the leader was already reaped: its descendants retain the native owner.
    for child in children.iter_mut() {
        child.start_kill()?;
    }
    tokio::time::timeout(grace, trees_exited(children))
        .await
        .map_err(|_| Error::Invalid("desktop development process trees did not exit within the configured deadline".into()))??;
    Ok(())
}

#[cfg(unix)]
async fn leaders_exited(children: &mut [Child]) -> std::io::Result<()> {
    for child in children {
        child.wait_leader().await?;
    }
    Ok(())
}

async fn trees_exited(children: &mut [Child]) -> std::io::Result<()> {
    for child in children {
        child.wait().await?;
    }
    Ok(())
}

fn termination_signal() -> i32 {
    #[cfg(unix)]
    {
        tokio::signal::unix::SignalKind::terminate().as_raw_value()
    }
    #[cfg(not(unix))]
    {
        15
    }
}

fn exit_code(status: ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.code().unwrap_or_else(|| -status.signal().expect("a Unix exit has a status or signal"))
    }
    #[cfg(not(unix))]
    {
        status.code().unwrap_or(1)
    }
}

/* === Tests === */
