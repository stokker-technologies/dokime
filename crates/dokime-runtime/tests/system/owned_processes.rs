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
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, INFINITE, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, WaitForSingleObject,
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

/* === Tests === */

#[test]
fn owned_processes() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        if let Some(root) = std::env::var_os("DOKIME_OWNED_HOST_ROOT") {
            let settings = support::settings();
            let plugin = PluginProcess::start(&support::execution_fixture()?, true, &settings.placement).await?;
            let host = Host::open(Path::new(&root), settings, support::service_settings()).await?;
            let entry = host.add_remote("owned", plugin.endpoint()).await?;
            let origin = host.materialize(&entry, "execution.capture", support::parameters(19.0, 0, true)).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            host.next_prompt(&cycle.executions[0]).await?;
            println!("HOST_CHILD {}", plugin.id());
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
        let pid = loop {
            let line = lines.next_line().await?.ok_or("Await Owned Host: Missing Plugin Identifier")?;
            if let Some(pid) = line.strip_prefix("HOST_CHILD ") {
                break pid.parse::<u32>()?;
            }
        };
        // Hold the Exact Process Object Before Killing Its Parent; PID Reuse Is Irrelevant.
        let child = unsafe {
            let handle = OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return Err(std::io::Error::last_os_error().into());
            }
            OwnedHandle::from_raw_handle(handle)
        };
        host.kill().await?;
        assert!(!host.wait().await?.success());
        let mut code = 259;
        unsafe {
            assert_eq!(WaitForSingleObject(child.as_raw_handle(), INFINITE), 0, "Reap Owned Plugin After Host Termination: PID {pid}");
            assert_ne!(GetExitCodeProcess(child.as_raw_handle(), &mut code), 0);
        }
        assert_ne!(code, 259, "Reap Owned Plugin: PID {pid} Remains Active");
        Ok(())
    }))
}
