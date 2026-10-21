/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stop The Host While An Operator Question Waits

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::{
    PROTECTION_SUITE, PROTECTION_TEMPLATE, STARTUP_SUITE, STARTUP_TEMPLATE, create_cycle, cycle_report, on_change, report_bytes, run_cycle,
    selection, suite, wait_prompt,
};
use crate::support::{self, host::Host};
use dokime_client::contracts;

/* === Constants === */

// The settle budget, the sidecar's grace and an unloaded checkpoint, with room
// for a shared CI host; the stop used to take minutes.
const STOP_BOUND: Duration = Duration::from_secs(15);
// An idle stop renders nothing and settles nothing; it used to seal every Active cycle.
const IDLE_STOP_BOUND: Duration = Duration::from_secs(3);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

// The plugin's sidecar processes among the host's children; host collectors
// spawn short-lived tools of their own beside them.
#[cfg(target_os = "linux")]
fn sidecars(pid: u32) -> Vec<u32> {
    let tasks = std::fs::read_dir(format!("/proc/{pid}/task")).unwrap();
    let children = tasks
        .filter_map(|task| std::fs::read_to_string(task.ok()?.path().join("children")).ok())
        .flat_map(|children| children.split_whitespace().filter_map(|child| child.parse::<u32>().ok()).collect::<Vec<_>>());
    children.filter(|child| std::fs::read_to_string(format!("/proc/{child}/comm")).is_ok_and(|name| name.trim() == "obc-plugin")).collect()
}

#[cfg(not(target_os = "linux"))]
fn sidecars(_pid: u32) -> Vec<u32> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn running(pid: u32) -> bool {
    // A reaped child has no entry; an unreaped one is a zombie, which has exited.
    std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .is_ok_and(|stat| stat.rsplit_once(')').is_some_and(|(_, rest)| !rest.trim_start().starts_with('Z')))
}

#[cfg(not(target_os = "linux"))]
fn running(_pid: u32) -> bool {
    false
}

/* === Tests === */

#[tokio::test]
async fn stop_while_prompt_waits() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("host");
    let bundle = support::artifacts::obc_plugin_bundle();
    let mut host = Host::launch_in(&home, Some(&bundle)).await;
    let client = host.connect().await;
    let reloaded = host.command(serde_json::json!({"command": "reload", "path": bundle})).await;
    assert_eq!(reloaded["event"], "reloaded", "{reloaded}");
    let mut changes = client.observation_watch_runtime(Default::default()).await.unwrap();

    // The Start-Up Case Runs To Its Operator Question, Which Nobody Answers
    let (mut catalog, mut planning, mut execution, mut prompts) = (client.clone(), client.clone(), client.clone(), client.clone());
    let template = suite(&mut catalog, STARTUP_SUITE)
        .await
        .templates
        .into_iter()
        .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == STARTUP_TEMPLATE))
        .expect("the start-up template is registered");
    let plan = selection(STARTUP_SUITE, &template.template_id, []);
    let cycle_id = create_cycle(&mut planning, &mut execution, plan, "native.charger.stop-while-prompt", 1).await;
    run_cycle(&mut execution, &cycle_id).await;
    let pending = wait_prompt(&mut changes, &mut prompts, &cycle_id).await;
    let execution_id = pending.scope.as_ref().and_then(|scope| scope.execution_id.clone()).expect("execution scope");
    let sidecars = sidecars(host.pid());
    assert!(cfg!(not(target_os = "linux")) || !sidecars.is_empty(), "the plugin runs as a sidecar of the host");
    drop((changes, client, catalog, planning, execution, prompts));

    // The Stop Settles The Run, Leaves No Sidecar Behind And Exits Cleanly In Bounded Time
    let (status, elapsed, diagnostics) = host.shutdown().await;
    eprintln!("the host stopped in {elapsed:?} with {status}");
    assert!(elapsed < STOP_BOUND, "the host took {elapsed:?} to stop, over {STOP_BOUND:?}:\n{diagnostics}");
    assert!(status.success(), "the host exited with {status}:\n{diagnostics}");
    for sidecar in sidecars {
        assert!(!running(sidecar), "sidecar {sidecar} outlived its host:\n{diagnostics}");
    }

    // The Next Start Seals What The Stop Deferred: The Execution Stopped, Not Interrupted, With Its Report
    let mut host = Host::launch_in(&home, Some(&bundle)).await;
    assert_eq!(host.command(serde_json::json!({"command": "reload", "path": bundle})).await["event"], "reloaded");
    let client = host.connect().await;
    let report = client.reports_download_execution_report(contracts::reports::DownloadExecutionReport { execution_id }).await.unwrap();
    let html = String::from_utf8(report_bytes(report).await).unwrap();
    assert!(html.contains("\"stopped\"") && !html.contains("\"interrupted\""), "the report records an operator stop");
    let pending = client
        .prompts_list_pending(contracts::prompts::ListPending { cycle_id: Some(cycle_id.clone()), ..Default::default() })
        .await
        .unwrap();
    assert!(pending.prompts.is_empty(), "the stop settled the unanswered question: {:?}", pending.prompts);
    drop(client);
    host.stop(false).await;
}

#[tokio::test]
async fn idle_stop_leaves_sealing_to_next_start() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("host");
    let bundle = support::artifacts::obc_plugin_bundle();
    let mut host = Host::launch_in(&home, Some(&bundle)).await;
    let client = host.connect().await;
    let reloaded = host.command(serde_json::json!({"command": "reload", "path": bundle})).await;
    assert_eq!(reloaded["event"], "reloaded", "{reloaded}");
    let mut changes = client.observation_watch_runtime(Default::default()).await.unwrap();

    // A Case Runs To Completion And Its Cycle Stays Active
    let (mut catalog, mut planning, mut execution) = (client.clone(), client.clone(), client.clone());
    let template = suite(&mut catalog, PROTECTION_SUITE)
        .await
        .templates
        .into_iter()
        .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == PROTECTION_TEMPLATE))
        .expect("the protection template is registered");
    let plan = selection(PROTECTION_SUITE, &template.template_id, []);
    let cycle_id = create_cycle(&mut planning, &mut execution, plan, "native.charger.idle-stop", 1).await;
    run_cycle(&mut execution, &cycle_id).await;
    on_change(&mut changes, async || {
        let cycle = execution.execution_get_cycle(contracts::execution::GetCycle { cycle_id: cycle_id.clone() }).await.unwrap();
        (cycle.running_count == 0 && cycle.queued_count == 0 && cycle.result.is_some()).then_some(())
    })
    .await;
    drop((changes, client, catalog, planning, execution));

    // The Stop Neither Seals Nor Renders: It Only Checkpoints
    let (status, elapsed, diagnostics) = host.shutdown().await;
    eprintln!("the idle host stopped in {elapsed:?} with {status}");
    assert!(elapsed < IDLE_STOP_BOUND, "the idle host took {elapsed:?} to stop, over {IDLE_STOP_BOUND:?}:\n{diagnostics}");
    assert!(status.success(), "the host exited with {status}:\n{diagnostics}");

    // The Next Start Seals The Cycle And Serves Its Report
    let mut host = Host::launch_in(&home, Some(&bundle)).await;
    let reloaded = host.command(serde_json::json!({"command": "reload", "path": bundle})).await;
    assert_eq!(reloaded["event"], "reloaded", "{reloaded}");
    let client = host.connect().await;
    assert!(!cycle_report(&client, &cycle_id).await.is_empty());
    drop(client);
    host.stop(false).await;
}
