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
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectExtendedLimitInformation, SetInformationJobObject,
    },
    Threading::GetCurrentProcess,
};

// Local Imports

/* === Constants === */

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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
    static CHILDREN: OnceLock<Result<OwnedHandle, String>> = OnceLock::new();
    match CHILDREN.get_or_init(|| own_children().map_err(|source| source.to_string())) {
        Ok(_) => Ok(()),
        Err(source) => Err(io::Error::other(format!("Own Child Processes: Assign the Host to a Kill-on-Close Job: {source}"))),
    }
}

pub fn command(program: impl AsRef<OsStr>) -> Command {
    own_descendants().expect("Own Child Processes: Assign the Host to a Kill-on-Close Job");
    #[allow(clippy::disallowed_methods, reason = "the replacement host process constructor")]
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/* === Private Functions === */

fn own_children() -> io::Result<OwnedHandle> {
    // Each Handle Is Owned Exactly Once and Never Inherited by a Child.
    unsafe {
        let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let handle = OwnedHandle::from_raw_handle(handle);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(
            handle.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        if AssignProcessToJobObject(handle.as_raw_handle(), GetCurrentProcess()) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(handle)
    }
}

/* === Tests === */
