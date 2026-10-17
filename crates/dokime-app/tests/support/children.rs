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
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

// Third Party Imports

// Local Imports

/* === Constants === */
/// The Line A Child Story Prints Once It Reached The State The Parent Kills It In.
const READY: &str = "dokime-child-story-ready";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// A Child Story That Is Killed And Reaped When Dropped, Even If The Parent Panics.
pub struct ChildStory(Child);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ChildStory {
    /// Re-Runs One Story Of This Test Binary As A Child Process The Parent Story Can Kill.
    pub fn spawn(story: &str, environment: &[(&str, &OsStr)]) -> Self {
        let child = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", story, "--nocapture"])
            .envs(environment.iter().copied())
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn child story");
        Self(child)
    }

    /// Waits For The Child's Readiness Line; Panics If The Child Exits First.
    ///
    /// The reader keeps draining the child's stdout until it exits, so the child
    /// never writes into a closed pipe while it waits to be killed.
    pub async fn ready(&mut self) {
        let stdout = self.0.stdout.take().expect("the child's stdout is piped");
        let (announced, signal) = tokio::sync::oneshot::channel();
        tokio::task::spawn_blocking(move || {
            let mut announced = Some(announced);
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else {
                    break;
                };
                if line == READY
                    && let Some(announced) = announced.take()
                {
                    let _ = announced.send(());
                }
            }
        });
        if signal.await.is_err() {
            let status = self.0.wait().expect("reap the child story");
            panic!("the child story exited before it was ready: {status}");
        }
    }

    /// Kills The Child At Its Kill Point And Reaps It.
    pub fn kill(self) {
        drop(self);
    }
}

impl Drop for ChildStory {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/* === Public Functions === */
/// Tells The Parent Story That This Child Reached Its Kill Point.
pub fn announce_ready() {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{READY}").expect("the parent reads the child's stdout");
    stdout.flush().expect("the parent reads the child's stdout");
}
/* === Private Functions === */

/* === Tests === */
