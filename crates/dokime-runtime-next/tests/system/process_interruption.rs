/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Process Interruption Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::Write, process::Stdio};

// Third Party Imports
use tokio::io::{AsyncBufReadExt, BufReader};

// Local Imports
use dokime_protocol::v1;
use dokime_runtime_next::{EntryState, Host, PluginProcess};
use dokime_storage::{ExecutionState, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn process_interruption() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Run the Production Host in a Separately Killable Process
        if let Some(root) = std::env::var_os("DOKIME_INTERRUPTION_CHILD") {
            let host = Host::open(std::path::Path::new(&root), super::support::settings())?;
            let entry = host.add_remote(&std::env::var("DOKIME_INTERRUPTION_ENDPOINT")?).await?;
            let origin = host.materialize(&entry, "execution.capture", super::support::parameters(19.0, 0, true)).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            host.next_prompt(&cycle.executions[0]).await?;
            println!("HOST_READY {}", cycle.executions[0]);
            std::io::stdout().flush()?;
            std::future::pending::<()>().await;
            unreachable!();
        }

        // Await Durable Evidence While the Step Is in Flight
        let root = tempfile::tempdir()?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::execution_fixture()?, true, &settings.placement).await?;
        let mut child = tokio::process::Command::from(dokime_runtime_next::process::command(std::env::current_exe()?))
            .args(["--exact", "process_interruption::process_interruption", "--nocapture"])
            .env("DOKIME_INTERRUPTION_CHILD", root.path())
            .env("DOKIME_INTERRUPTION_ENDPOINT", plugin.endpoint())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let execution = async {
            while let Some(line) = lines.next_line().await? {
                if let Some(identifier) = line.strip_prefix("HOST_READY ") {
                    return Ok::<_, std::io::Error>(identifier.to_owned());
                }
            }
            Err(std::io::Error::other("Host Exited Before the Step Prompt"))
        }
        .await?;
        let store = Store::open(root.path())?;
        let before = store.read(&execution)?;
        assert_eq!(before.state, ExecutionState::Open);
        assert!(!before.samples.is_empty());
        assert_eq!(before.logs()?.len(), 1);
        drop(store);

        // Kill the Real Host Process and Recover on the Same Root
        child.kill().await?;
        assert!(!child.wait().await?.success());
        let host = Host::open(root.path(), settings.clone())?;
        let recovered = host.execution(&execution)?;
        assert_eq!(recovered.state, ExecutionState::Sealed);
        assert_eq!(recovered.execution.termination, v1::Termination::Interrupted as i32);
        assert_eq!(recovered.evidence, before.evidence);
        assert_eq!(recovered.samples, before.samples);
        assert!(host.streams_closed(&execution)?);
        assert_eq!(host.entry_state("execution")?, EntryState::Unavailable);
        assert!(host.description("execution")?.catalog.is_some());
        assert!(!host.capabilities(&execution)?.cancel);

        // Reattach Without Replaying Operations or Samples
        host.add_remote(plugin.endpoint()).await?;
        assert_eq!(host.entry_state("execution")?, EntryState::Ready);
        assert_eq!(host.execution(&execution)?, recovered);
        host.shutdown().await?;
        drop(host);
        let host = Host::open(root.path(), settings)?;
        assert_eq!(host.execution(&execution)?, recovered);
        host.shutdown().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
