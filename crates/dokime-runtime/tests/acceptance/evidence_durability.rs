/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Durability Through Actual SQLite Fault And Retry Boundaries

use crate::evidence_support::{CRASH_READY, crash_writer, recovered_after_crash};
use std::io::BufRead;

struct Writer(std::process::Child);

impl Drop for Writer {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[tokio::test]
async fn evidence_durability() {
    if let Ok(phase) = std::env::var("DOKIME_EVIDENCE_CRASH_PHASE") {
        let root = std::env::var_os("DOKIME_EVIDENCE_CRASH_ROOT").unwrap();
        if phase == "checkpoint" {
            crate::managed_support::checkpoint_crash_writer(root.into()).await;
        } else {
            crash_writer(root.into(), phase).await;
        }
        return;
    }
    let root = tempfile::tempdir().unwrap();
    for phase in ["row", "capture", "frontier", "saved", "checkpoint"] {
        let directory = root.path().join(phase);
        std::fs::create_dir(&directory).unwrap();
        let executable = std::env::current_exe().unwrap();
        let mut command = dokime_domain::process::command(executable);
        command
            .args(["--exact", "evidence_durability::evidence_durability", "--nocapture"])
            .env("DOKIME_EVIDENCE_CRASH_PHASE", phase)
            .env("DOKIME_EVIDENCE_CRASH_ROOT", &directory)
            .stdout(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null());
        let mut writer = Writer(command.spawn().unwrap());
        let output = writer.0.stdout.take().unwrap();
        let mut ready = false;
        for line in std::io::BufReader::new(output).lines() {
            if line.unwrap() == CRASH_READY {
                ready = true;
                break;
            }
        }
        assert!(ready);
        writer.0.kill().unwrap();
        writer.0.wait().unwrap();
        if phase == "checkpoint" {
            crate::managed_support::checkpoint_recovered(directory).await;
        } else {
            recovered_after_crash(directory, phase == "saved").await;
        }
    }
}
