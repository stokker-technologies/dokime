/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reap Plugins After Forced Host Termination; Keep What the User Opened

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    error::Error,
    io::{BufRead, Write},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    process::Stdio,
};

// Third Party Imports
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    process::Command,
};
use windows_sys::Win32::{
    System::{
        Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS},
        Threading::{
            GetExitCodeProcess, INFINITE, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, WaitForSingleObject,
        },
    },
    UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_HIDE},
};

// Local Imports
use super::support;
use dokime_runtime::{Host, PluginProcess, process};

/* === Constants === */

const STILL_ACTIVE: u32 = 259;
const RELEASED: i32 = 23;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn hold(pid: u32) -> Result<OwnedHandle, Box<dyn Error>> {
    // Hold the Exact Process Object Before Killing Its Parent; PID Reuse Is Irrelevant.
    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}

fn exited(process: &OwnedHandle) -> u32 {
    let mut code = STILL_ACTIVE;
    unsafe {
        assert_eq!(WaitForSingleObject(process.as_raw_handle(), INFINITE), 0);
        assert_ne!(GetExitCodeProcess(process.as_raw_handle(), &mut code), 0);
    }
    code
}

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
    if std::env::var_os("DOKIME_OWNED_PARK").is_some() {
        // A Program the User Keeps: Report, Then Exit Only When Released.
        let mut report = std::net::TcpStream::connect(std::env::var("DOKIME_OWNED_REPORT")?)?;
        writeln!(report, "{}", std::process::id())?;
        std::io::BufReader::new(&report).read_line(&mut String::new())?;
        std::process::exit(RELEASED);
    }
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
            // Open a Document the Way the Desktop Does, Then Hand Over the Way Its Updater Does.
            let launch = std::env::var_os("DOKIME_OWNED_LAUNCH").ok_or("Launch Owned Programs: Supply the Launch Script")?;
            process::open(&launch)?;
            process::detach_later_children()?;
            let wide = |value: &std::ffi::OsStr| value.encode_wide().chain([0]).collect::<Vec<u16>>();
            let (verb, file) = (wide("open".as_ref()), wide(&launch));
            let started =
                unsafe { ShellExecuteW(std::ptr::null_mut(), verb.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_HIDE) };
            assert!(started as isize > 32, "Hand Over to the Installer: ShellExecuteW Returned {}", started as isize);
            println!("HOST_CHILDREN {} {}", plugin.id(), managed[0]);
            std::io::stdout().flush()?;
            std::future::pending::<()>().await;
            unreachable!();
        }

        let root = support::tempdir()?;
        let work = support::tempdir()?;
        let reports = TcpListener::bind("127.0.0.1:0").await?;
        // Opened Files Live in User Folders; a Space Exercises the Launcher's Quoting.
        std::fs::create_dir(work.path().join("Opened Files"))?;
        let launch = work.path().join("Opened Files").join("launch.cmd");
        let script = format!(
            "@echo off\r\nset DOKIME_OWNED_HOST_ROOT=\r\nset DOKIME_OWNED_PARK=1\r\n\"{}\" --exact owned_processes::owned_processes --nocapture\r\n",
            std::env::current_exe()?.display()
        );
        std::fs::write(&launch, script)?;
        let mut host = Command::from(process::command(std::env::current_exe()?))
            .args(["--exact", "owned_processes::owned_processes", "--nocapture"])
            .env("DOKIME_OWNED_HOST_ROOT", root.path())
            .env("DOKIME_CREDENTIAL_NAMESPACE", support::namespace().as_str())
            .env("DOKIME_OWNED_LAUNCH", &launch)
            .env("DOKIME_OWNED_REPORT", reports.local_addr()?.to_string())
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
        let plugins = pids.into_iter().map(|pid| Ok((pid, hold(pid)?))).collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        let mut launched = Vec::new();
        for _ in 0..2 {
            let (report, _) = reports.accept().await?;
            let (read, write) = report.into_split();
            let pid = BufReader::new(read).lines().next_line().await?.ok_or("Await Opened Program: Missing Identifier")?.parse::<u32>()?;
            launched.push((pid, hold(pid)?, write));
        }
        host.kill().await?;
        assert!(!host.wait().await?.success());
        for (pid, plugin) in plugins {
            assert_ne!(exited(&plugin), STILL_ACTIVE, "Reap Owned Plugin After Host Termination: PID {pid}");
        }
        // Every Plugin Has Ended, So the Job Has Closed; Only Its Survivors Can Answer.
        for (pid, program, mut release) in launched {
            release
                .write_all(
                    b"release
",
                )
                .await?;
            assert_eq!(exited(&program), RELEASED as u32, "Keep the Opened Program After Host Termination: PID {pid}");
        }
        Ok(())
    }))
}
