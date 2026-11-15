/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    ffi::OsStr,
    io,
    mem::size_of,
    os::windows::{
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
        process::CommandExt,
    },
    process::Command,
    sync::OnceLock,
};

// Third Party Imports
use windows_sys::Win32::System::{
    JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_BREAKAWAY_OK, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject,
    },
    Threading::{CREATE_BREAKAWAY_FROM_JOB, GetCurrentProcess},
};

// Local Imports

/* === Constants === */

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

// Plugins and Hosts Inherit the Job; Launches Meant for the User Break Away Explicitly.
const OWNED: u32 = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_BREAKAWAY_OK;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn own_descendants() -> io::Result<()> {
    // The Non-Inherited Handle Closes Even When This Process Is Forcibly Killed.
    // Descendants Inherit Membership, Including Plugins the Library Spawns.
    children().map(drop)
}

pub fn open(target: impl AsRef<OsStr>) -> io::Result<()> {
    // The Shell Launcher Leaves the Job, So What It Opens Outlives This Process.
    let target = target.as_ref();
    let launcher = std::env::var_os("SystemRoot")
        .map_or_else(|| "rundll32.exe".into(), |root| std::path::PathBuf::from(root).join("System32").join("rundll32.exe"));
    #[allow(clippy::disallowed_methods, reason = "a user launch that leaves the owned job")]
    let mut command = Command::new(launcher);
    command.arg("shell32.dll,ShellExec_RunDLL").arg(target).creation_flags(CREATE_BREAKAWAY_FROM_JOB);
    let launched = match command.spawn() {
        // An Enclosing Job That Forbids Breakaway Keeps Ownership of the Launch.
        Err(source) if source.kind() == io::ErrorKind::PermissionDenied => command.creation_flags(0).spawn(),
        launched => launched,
    };
    launched
        .map(drop)
        .map_err(|source| io::Error::new(source.kind(), format!("Open {}: Start the Windows Shell Launcher: {source}", target.display())))
}

pub fn detach_later_children() -> io::Result<()> {
    // For a Handover Such as an Installer: Every Later Child Outlives This Process.
    // Children Already Running Stay Owned and End With It.
    let job = children()?;
    set_limits(job, OWNED | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK)
        .map_err(|source| io::Error::other(format!("Detach Later Child Processes: Allow Silent Breakaway: {source}")))
}

pub fn own_later_children() -> io::Result<()> {
    // A Handover That Did Not Start Returns Later Children to the Job.
    let job = children()?;
    set_limits(job, OWNED).map_err(|source| io::Error::other(format!("Own Later Child Processes: Restore the Job Limits: {source}")))
}

pub fn command(program: impl AsRef<OsStr>) -> Command {
    own_descendants().expect("Own Child Processes: Assign the Host to a Kill-on-Close Job");
    #[allow(clippy::disallowed_methods, reason = "the replacement host process constructor")]
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/* === Private Functions === */

fn children() -> io::Result<&'static OwnedHandle> {
    static CHILDREN: OnceLock<Result<OwnedHandle, String>> = OnceLock::new();
    match CHILDREN.get_or_init(|| own_children().map_err(|source| source.to_string())) {
        Ok(job) => Ok(job),
        Err(source) => Err(io::Error::other(format!("Own Child Processes: Assign the Host to a Kill-on-Close Job: {source}"))),
    }
}

fn set_limits(job: &OwnedHandle, flags: u32) -> io::Result<()> {
    unsafe {
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        limits.BasicLimitInformation.LimitFlags = flags;
        if SetInformationJobObject(
            job.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

fn own_children() -> io::Result<OwnedHandle> {
    // Each Handle Is Owned Exactly Once and Never Inherited by a Child.
    unsafe {
        let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let handle = OwnedHandle::from_raw_handle(handle);
        set_limits(&handle, OWNED)?;
        if AssignProcessToJobObject(handle.as_raw_handle(), GetCurrentProcess()) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(handle)
    }
}

/* === Tests === */
