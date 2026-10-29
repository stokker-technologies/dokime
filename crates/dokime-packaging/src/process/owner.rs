/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Own a native process tree through leader exit and descendant cleanup.

use std::{future::Future, io, pin::Pin, process::ExitStatus};

use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};

pub struct Child {
    child: Box<dyn ChildWrapper>,
    armed: bool,
    #[cfg(unix)]
    group: libc::pid_t,
}

impl Child {
    pub fn stdin(&mut self) -> &mut Option<ChildStdin> {
        self.child.stdin()
    }

    pub fn stdout(&mut self) -> &mut Option<ChildStdout> {
        self.child.stdout()
    }

    pub fn stderr(&mut self) -> &mut Option<ChildStderr> {
        self.child.stderr()
    }

    /// Observe the leader without releasing its process-group or job owner.
    pub fn wait_leader(&mut self) -> Pin<Box<dyn Future<Output = io::Result<ExitStatus>> + Send + '_>> {
        // spawn installs exactly one child wrapper; KillOnDrop only configures the command.
        self.child.inner_mut().wait()
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.inner_mut().try_wait()
    }

    pub async fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait().await?;
        #[cfg(unix)]
        self.group_exists()?;
        // Windows wrapper waits can return a cached leader exit after cancellation;
        // keep the stable JobObject armed until an explicit successful termination.
        Ok(status)
    }

    pub fn start_kill(&mut self) -> io::Result<()> {
        if !self.armed {
            return Ok(());
        }
        let result = match self.child.start_kill() {
            #[cfg(unix)]
            Err(error) if error.raw_os_error() == Some(libc::ESRCH) => Ok(()),
            result => result,
        };
        if result.is_ok() {
            self.armed = false;
        }
        result
    }

    pub async fn kill(&mut self) -> io::Result<()> {
        self.start_kill()?;
        self.wait().await?;
        Ok(())
    }

    #[cfg(unix)]
    pub fn signal(&mut self, signal: i32) -> io::Result<()> {
        if !self.armed {
            return Ok(());
        }
        match self.child.signal(signal) {
            Err(error) if error.raw_os_error() == Some(libc::ESRCH) => {
                self.armed = false;
                Ok(())
            }
            result => result,
        }
    }

    #[cfg(unix)]
    pub fn group_exists(&mut self) -> io::Result<bool> {
        if !self.armed {
            return Ok(false);
        }
        // A group can outlive its leader. Its numeric identity must be retired once absence is observed.
        let result = unsafe { libc::kill(-self.group, 0) };
        if result == 0 {
            return Ok(true);
        }
        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::ESRCH) => {
                self.armed = false;
                Ok(false)
            }
            Some(libc::EPERM) => Ok(true),
            _ => Err(error),
        }
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        // Cancellation must not leave descendants after the direct child's handle is dropped.
        let _ = self.start_kill();
    }
}

pub fn spawn(command: Command) -> io::Result<Child> {
    spawn_with(command, false)
}

pub fn spawn_session(command: Command) -> io::Result<Child> {
    spawn_with(command, true)
}

fn spawn_with(command: Command, session: bool) -> io::Result<Child> {
    let mut command = CommandWrap::from(command);
    command.wrap(KillOnDrop);
    #[cfg(unix)]
    if session {
        command.wrap(process_wrap::tokio::ProcessSession);
    } else {
        command.wrap(process_wrap::tokio::ProcessGroup::leader());
    }
    #[cfg(not(unix))]
    let _ = session;
    #[cfg(windows)]
    command.wrap(without_console()).wrap(process_wrap::tokio::JobObject);
    let child = command.spawn()?;
    #[cfg(unix)]
    let group = libc::pid_t::try_from(child.id().expect("a newly spawned process has an identity"))
        .map_err(|_| io::Error::other("native process identity does not fit a process group"))?;
    Ok(Child {
        child,
        armed: true,
        #[cfg(unix)]
        group,
    })
}

// JobObject Replaces Native Flags; This Wrapper Keeps The Child Without A Console Window.
#[cfg(windows)]
fn without_console() -> process_wrap::tokio::CreationFlags {
    let mut flags = process_wrap::tokio::CreationFlags(Default::default());
    flags.0.0 = dokime::domain::process::CREATE_NO_WINDOW;
    flags
}
