/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::{Arc, PoisonError};
use std::time::Duration;

use tokio::sync::{oneshot, watch};

use super::log::Level;
use crate::artifacts::Artifacts;
use crate::transport::buffer::{BufferedFrame, Executions, Tasks, lock};

pub(super) struct Scanning {
    stop: watch::Sender<bool>,
    completion: oneshot::Receiver<()>,
}

pub(super) fn start(artifacts: Artifacts, executions: &Executions, token: &str, tasks: &Tasks) -> Scanning {
    let interval = Duration::from_millis(100);
    let (stop, mut listener) = watch::channel(false);
    let (completed, completion) = oneshot::channel();
    let owner = Arc::clone(executions);
    let owned = token.to_owned();
    let mut tasks = tasks.lock().unwrap_or_else(PoisonError::into_inner);
    tasks.retain(|task| !task.is_finished());
    tasks.push(tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = listener.wait_for(|stopped| *stopped) => break,
                _ = tokio::time::sleep(interval) => {}
            }
            let notes = scan_blocking(&artifacts).await;
            report(&owner, &owned, notes);
        }
        let _ = completed.send(());
    }));

    Scanning { stop, completion }
}

pub(super) async fn settle(scanning: Scanning, artifacts: &Artifacts, executions: &Executions, token: &str) {
    scanning.stop.send_replace(true);
    let _ = scanning.completion.await;
    let artifacts = artifacts.clone();
    let notes = tokio::task::spawn_blocking(move || artifacts.sweep()).await.expect("the final sweep does not panic");
    report(executions, token, notes);
}

async fn scan_blocking(artifacts: &Artifacts) -> Vec<String> {
    let artifacts = artifacts.clone();
    tokio::task::spawn_blocking(move || artifacts.scan()).await.expect("a scan tick does not panic")
}

fn report(executions: &Executions, token: &str, notes: Vec<String>) {
    if notes.is_empty() {
        return;
    }
    let mut executions = lock(executions);
    let Some(state) = executions.get_mut(token) else {
        return;
    };
    for note in notes {
        state.push(BufferedFrame::log(Level::Warn, note));
    }
}
