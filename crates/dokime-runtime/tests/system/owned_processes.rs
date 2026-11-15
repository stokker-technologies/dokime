/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reap Plugins After Forced Host Termination

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    error::Error,
    io::Write,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::Path,
    process::Stdio,
};

// Third Party Imports
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use windows_sys::Win32::System::{
    Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS},
    Threading::{GetExitCodeProcess, INFINITE, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, WaitForSingleObject},
};

// Local Imports
use super::support;
use dokime_runtime::{Host, PluginProcess, process};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn children(parent: u32) -> Result<Vec<u32>, Box<dyn Error>> {
    // List the Direct Children the Library Spawned for This Host Process.
    let mut children = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() || snapshot as isize == -1 {
            return Err(std::io::Error::last_os_error().into());
        }
        let snapshot = OwnedHandle::from_raw_handle(snapshot);
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut present = Process32FirstW(snapshot.as_raw_handle(), &mut entry) != 0;
        while present {
            if entry.th32ParentProcessID == parent {
                children.push(entry.th32ProcessID);
            }
            present = Process32NextW(snapshot.as_raw_handle(), &mut entry) != 0;
        }
    }
    Ok(children)
}

/* === Tests === */

#[test]
fn owned_processes() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        if let Some(root) = std::env::var_os("DOKIME_OWNED_HOST_ROOT") {
            // Start the Library-Spawned Managed Plugin Before Any Product-Spawned Process.
            let host = Host::open(Path::new(&root), support::settings(), support::service_settings()).await?;
            let before = children(std::process::id())?;
            let entry = host.add_managed("managed", &support::execution_fixture()?, &[]).await?;
            let origin = host.materialize(&entry, "execution.capture", support::parameters(19.0, 0, true)).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            host.next_prompt(&cycle.executions[0]).await?;
            let managed = children(std::process::id())?.into_iter().filter(|pid| !before.contains(pid)).collect::<Vec<_>>();
            assert_eq!(managed.len(), 1, "Start One Managed Plugin Child: {managed:?}");
            let plugin = PluginProcess::start(&support::boundary_fixture()?, true, &support::settings().placement).await?;
            host.add_remote("remote", plugin.endpoint()).await?;
            println!("HOST_CHILDREN {} {}", plugin.id(), managed[0]);
            std::io::stdout().flush()?;
            std::future::pending::<()>().await;
            unreachable!();
        }

        let root = support::tempdir()?;
        let mut host = Command::from(process::command(std::env::current_exe()?))
            .args(["--exact", "owned_processes::owned_processes", "--nocapture"])
            .env("DOKIME_OWNED_HOST_ROOT", root.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let mut lines = BufReader::new(host.stdout.take().unwrap()).lines();
        let pids = loop {
            let line = lines.next_line().await?.ok_or("Await Owned Host: Missing Plugin Identifiers")?;
            if let Some(pids) = line.strip_prefix("HOST_CHILDREN ") {
                break pids.split(' ').map(str::parse::<u32>).collect::<Result<Vec<_>, _>>()?;
            }
        };
        // Hold the Exact Process Objects Before Killing Their Parent; PID Reuse Is Irrelevant.
        let mut plugins = Vec::new();
        for pid in &pids {
            let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, 0, *pid) };
            if handle.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            plugins.push((*pid, unsafe { OwnedHandle::from_raw_handle(handle) }));
        }
        host.kill().await?;
        assert!(!host.wait().await?.success());
        for (pid, plugin) in plugins {
            let mut code = 259;
            unsafe {
                assert_eq!(WaitForSingleObject(plugin.as_raw_handle(), INFINITE), 0, "Reap Owned Plugin After Host Termination: PID {pid}");
                assert_ne!(GetExitCodeProcess(plugin.as_raw_handle(), &mut code), 0);
            }
            assert_ne!(code, 259, "Reap Owned Plugin: PID {pid} Remains Active");
        }
        Ok(())
    }))
}
