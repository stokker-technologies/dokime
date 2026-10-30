/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Configured Row-Or-Time Flushes And The Unflushed Read Tail

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use harmos::storage::StorageCursor;
use tokio::{sync::Notify, time::Instant};

// Local Imports
use super::super::super::State;
use crate::storage::{Error, Manager, Result, telemetry::TelemetryEntry};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(crate) struct Tail {
    pub entries: BTreeMap<u64, (Instant, TelemetryEntry)>,
    pub running: bool,
    pub force: bool,
    pub changed: Arc<Notify>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn stage_telemetry(&self, storage: Manager, through: StorageCursor, entries: Vec<TelemetryEntry>) -> Result<()> {
        let state = self.clone();
        let end = through.sequence;
        let first = end.checked_sub(entries.len() as u64).ok_or(Error::Invalid("telemetry admission cursor underflows"))? + 1;
        let settings = self.0.telemetry.lock().unwrap().settings;
        let start = self
            .call(move |connection| {
                Ok((|| -> Result<bool> {
                    let saved = super::super::saved(connection)?;
                    saved.history.check(&through.history)?;
                    let store = state.0.telemetry.lock().unwrap();
                    let mut tail = state.0.telemetry_tail.lock().unwrap();
                    for (index, mut entry) in entries.into_iter().enumerate() {
                        let position = first + index as u64;
                        if position > saved.sequence {
                            store.prepare_tail(&mut entry)?;
                            tail.entries.entry(position).or_insert((Instant::now(), entry));
                        }
                    }
                    tail.changed.notify_one();
                    let start = !tail.running && !tail.entries.is_empty();
                    if start {
                        tail.running = true;
                    }
                    Ok(start)
                })())
            })
            .await??;
        if !start {
            return Ok(());
        }
        let state = self.clone();
        let task = tokio::spawn(async move {
            let result = run(&state, &storage, settings).await;
            if let Err(error) = &result {
                state.0.telemetry_tail.lock().unwrap().running = false;
                tracing::error!(%error,"telemetry flush failed; saved cursor did not advance");
            }
            result
        });
        let mut tasks = self.0.telemetry_tasks.lock().await;
        tasks.retain(|task| !task.is_finished());
        tasks.push(task);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn run(state: &State, storage: &Manager, settings: super::super::super::Settings) -> Result<()> {
    loop {
        let (deadline, full, changed) = {
            let mut tail = state.0.telemetry_tail.lock().unwrap();
            let Some((started, _)) = tail.entries.values().next() else {
                tail.running = false;
                return Ok(());
            };
            let deadline = started.checked_add(settings.telemetry_interval).ok_or(Error::Invalid("telemetry flush deadline overflows"))?;
            let mut counts = BTreeMap::new();
            for (_, entry) in tail.entries.values() {
                let owner = entry
                    .sample
                    .scope
                    .as_ref()
                    .and_then(|scope| scope.test_execution_id.as_ref())
                    .map(ToString::to_string)
                    .unwrap_or_else(|| entry.epoch.to_string());
                *counts.entry((owner, entry.sample.stream.as_str())).or_insert(0_usize) += 1;
            }
            (deadline, tail.force || counts.values().any(|rows| *rows >= settings.telemetry_rows.get()), tail.changed.clone())
        };
        if !full && Instant::now() < deadline {
            tokio::select! { _ = tokio::time::sleep_until(deadline) => {}, _ = changed.notified() => { continue; } }
        }
        let through = storage.cursor_accepted();
        storage.flush_until(&through).await?;
        let saved = storage.cursor_saved();
        state.0.telemetry_tail.lock().unwrap().entries.retain(|position, _| *position > saved.sequence);
    }
}

/* === Tests === */
