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

struct Flush {
    deadline: Instant,
    full: bool,
    changed: Arc<Notify>,
    through: StorageCursor,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Tail {
    fn discard_saved(&mut self, through: u64) {
        while self.entries.first_key_value().is_some_and(|(position, _)| *position <= through) {
            self.entries.pop_first();
        }
    }

    fn cut(&mut self, mut through: StorageCursor, settings: super::super::super::Settings) -> Result<Option<Flush>> {
        self.discard_saved(through.sequence);
        let Some((started, _)) = self.entries.values().next() else {
            self.running = false;
            return Ok(None);
        };
        let deadline = started.checked_add(settings.telemetry_interval).ok_or(Error::Invalid("telemetry flush deadline overflows"))?;
        let mut counts = BTreeMap::new();
        for (_, entry) in self.entries.values() {
            let owner = entry
                .sample
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref())
                .map(ToString::to_string)
                .unwrap_or_else(|| entry.epoch.to_string());
            *counts.entry((owner, entry.sample.stream.as_str())).or_insert(0_usize) += 1;
        }
        through.sequence = *self.entries.last_key_value().unwrap().0;
        Ok(Some(Flush {
            deadline,
            full: self.force || counts.values().any(|rows| *rows >= settings.telemetry_rows.get()),
            changed: self.changed.clone(),
            through,
        }))
    }
}

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
                    tail.discard_saved(saved.sequence);
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
        let saved = storage.cursor_saved();
        let cut = {
            let mut tail = state.0.telemetry_tail.lock().unwrap();
            tail.cut(saved.clone(), settings)?
        };
        let Some(Flush { deadline, full, changed, through }) = cut else { return Ok(()) };
        if !full && Instant::now() < deadline {
            tokio::select! {
                _ = tokio::time::sleep_until(deadline) => {},
                _ = changed.notified() => continue,
                result = storage.wait_saved_after(&saved) => {
                    result?;
                    continue;
                }
            }
        }
        storage.flush_until(&through).await?;
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_tail_cut() {
        use dokime_domain::model::identity::{SessionId, TelemetrySampleId};
        let epoch = SessionId::from_seed("flush-session");
        let mut tail = Tail { running: true, ..Default::default() };
        for ordinal in 1..=4 {
            tail.entries.insert(
                ordinal,
                (
                    Instant::now(),
                    TelemetryEntry {
                        epoch: epoch.clone(),
                        ordinal,
                        sample: crate::telemetry::TelemetrySample::new(
                            TelemetrySampleId::from_seed(ordinal.to_string()),
                            harmos::Identifier::new("flush.reading").unwrap(),
                            ordinal,
                            dokime_domain::model::time::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH),
                            "observer",
                        ),
                    },
                ),
            );
        }
        let fourth = tail.entries.remove(&4).unwrap();
        let saved = StorageCursor {
            history: harmos::storage::StoreHistory {
                store: harmos::Identifier::new("store").unwrap(),
                epoch: harmos::Identifier::new("epoch").unwrap(),
            },
            sequence: 2,
        };
        let settings = super::super::super::super::Settings {
            telemetry_rows: std::num::NonZeroUsize::new(2).unwrap(),
            ..crate::config::local_storage_defaults()
        };
        let cut = tail.cut(saved.clone(), settings).unwrap().unwrap();
        assert!(!cut.full);
        assert_eq!(cut.through.sequence, 3);
        assert_eq!(tail.entries.keys().copied().collect::<Vec<_>>(), [3]);
        tail.entries.insert(4, fourth);
        let cut = tail.cut(saved, settings).unwrap().unwrap();
        assert!(cut.full);
        assert_eq!(cut.through.sequence, 4);
        assert!(tail.cut(cut.through, settings).unwrap().is_none());
        assert!(!tail.running);
    }
}
