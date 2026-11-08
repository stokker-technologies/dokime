/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Configured Row-Or-Time Flushes and the Unflushed Read Tail

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
    streams: BTreeMap<String, u64>,
    counts: BTreeMap<String, BTreeMap<String, usize>>,
    pub readable: tokio::sync::watch::Sender<u64>,
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
    pub(super) fn select(
        &self, after: u64, at: u64, mut matches: impl FnMut(&TelemetryEntry) -> Result<bool>,
    ) -> Result<Vec<(u64, TelemetryEntry)>> {
        if after >= at {
            return Ok(Vec::new());
        }
        self.entries
            .range((std::ops::Bound::Excluded(after), std::ops::Bound::Included(at)))
            .filter_map(|(position, (_, entry))| match matches(entry) {
                Ok(true) => Some(Ok((*position, entry.clone()))),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            })
            .collect()
    }

    pub(super) fn streams_between(&self, after: u64, at: u64) -> std::collections::BTreeSet<String> {
        if after >= at {
            return Default::default();
        }
        self.streams
            .iter()
            .filter(|(stream, latest)| {
                **latest > after
                    && (**latest <= at
                        || self
                            .entries
                            .range((std::ops::Bound::Excluded(after), std::ops::Bound::Included(at)))
                            .any(|(_, (_, entry))| entry.sample.stream.as_str() == stream.as_str()))
            })
            .map(|(stream, _)| stream.clone())
            .collect()
    }

    pub(in crate::storage::sqlite) fn insert(&mut self, position: u64, entry: TelemetryEntry) {
        if self.entries.contains_key(&position) {
            return;
        }
        self.streams.entry(entry.sample.stream.to_string()).and_modify(|latest| *latest = (*latest).max(position)).or_insert(position);
        let owner = entry
            .sample
            .scope
            .as_ref()
            .and_then(|scope| scope.test_execution_id.as_ref())
            .map(|id| id.as_str())
            .unwrap_or_else(|| entry.epoch.as_str());
        *self.counts.entry(entry.sample.stream.to_string()).or_default().entry(owner.to_owned()).or_default() += 1;
        self.entries.insert(position, (Instant::now(), entry));
    }

    pub(in crate::storage::sqlite) fn discard_saved(&mut self, through: u64) {
        self.streams.retain(|_, latest| *latest > through);
        while self.entries.first_key_value().is_some_and(|(position, _)| *position <= through) {
            let (_, (_, entry)) = self.entries.pop_first().unwrap();
            let owner = entry
                .sample
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref())
                .map(|id| id.as_str())
                .unwrap_or_else(|| entry.epoch.as_str());
            let counts = self.counts.get_mut(entry.sample.stream.as_str()).expect("counted tail stream");
            let count = counts.get_mut(owner).expect("counted tail owner");
            *count -= 1;
            if *count == 0 {
                counts.remove(owner);
            }
            if counts.is_empty() {
                self.counts.remove(entry.sample.stream.as_str());
            }
        }
    }

    fn cut(&mut self, mut through: StorageCursor, settings: super::super::super::Settings) -> Result<Option<Flush>> {
        self.discard_saved(through.sequence);
        let Some((started, _)) = self.entries.values().next() else {
            self.running = false;
            return Ok(None);
        };
        let deadline = started.checked_add(settings.telemetry_interval).ok_or(Error::Invalid("telemetry flush deadline overflows"))?;
        through.sequence = *self.entries.last_key_value().unwrap().0;
        Ok(Some(Flush {
            deadline,
            full: self.force || self.counts.values().flat_map(|owners| owners.values()).any(|rows| *rows >= settings.telemetry_rows.get()),
            changed: self.changed.clone(),
            through,
        }))
    }
}

impl State {
    pub(crate) fn watch_telemetry(&self) -> tokio::sync::watch::Receiver<u64> {
        self.0.telemetry_tail.lock().unwrap().readable.subscribe()
    }

    pub(crate) async fn stage_telemetry(&self, storage: Manager, through: StorageCursor, entries: Vec<TelemetryEntry>) -> Result<()> {
        let state = self.clone();
        let end = through.sequence;
        let first = end.checked_sub(entries.len() as u64).ok_or(Error::Invalid("telemetry admission cursor underflows"))? + 1;
        let settings = self.0.settings;
        let admitted = storage.clone();
        let parent = tracing::Span::current();
        let staging = std::time::Instant::now();
        // Tail Admission Needs No Disk Writer: The Saved Cursor Is Published After Its Catalog Commit.
        let start = tokio::task::spawn_blocking(move || -> Result<bool> {
            let _parent = parent.enter();
            let _span = tracing::debug_span!("ingest.host.stage").entered();
            if state.0.closing.load(std::sync::atomic::Ordering::Acquire) {
                return Err(Error::Closed);
            }
            let saved = admitted.cursor_saved();
            saved.history.check(&through.history)?;
            let mut pending = Vec::with_capacity(entries.len());
            {
                let contracts = state.0.telemetry_contracts.read().unwrap();
                for (index, mut entry) in entries.into_iter().enumerate() {
                    let position = first + index as u64;
                    if position <= saved.sequence {
                        continue;
                    }
                    let key = serde_json::to_string(&super::reader::key(&entry)?)?;
                    let (_, field) = contracts.get(&key).ok_or(Error::Invalid("telemetry tail contract is not registered"))?;
                    {
                        let _span = tracing::debug_span!("ingest.normalize.tail").entered();
                        crate::storage::ipc::normalize_values(field, &mut entry.sample.values)?;
                    }
                    pending.push((position, entry));
                }
            }
            let waiting = std::time::Instant::now();
            let mut tail = state.0.telemetry_tail.lock().unwrap();
            tracing::debug!(name: "ingest.lock.tail", elapsed_ns = waiting.elapsed().as_nanos() as u64);
            tail.discard_saved(saved.sequence);
            for (position, entry) in pending {
                let _span = tracing::debug_span!("ingest.tail.insert").entered();
                tail.insert(position, entry);
            }
            // Publish Read Progress Only After Every Row Is In The Tail Or Saved.
            tail.readable.send_if_modified(|position| {
                if end <= *position {
                    return false;
                }
                *position = end;
                true
            });
            tail.changed.notify_one();
            let start = !tail.running && !tail.entries.is_empty();
            if start {
                tail.running = true;
            }
            Ok(start)
        })
        .await??;
        tracing::debug!(name: "ingest.host.stage.elapsed", elapsed_ns = staging.elapsed().as_nanos() as u64);
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
        for ordinal in 1..=3 {
            tail.insert(
                ordinal,
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
            );
        }
        let fourth = tail.entries[&3].1.clone();
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
        tail.insert(3, fourth.clone());
        assert!(!tail.cut(saved.clone(), settings).unwrap().unwrap().full);
        let mut other = fourth.clone();
        other.epoch = SessionId::from_seed("other-session");
        tail.insert(4, other);
        assert!(!tail.cut(saved.clone(), settings).unwrap().unwrap().full);
        tail.insert(5, fourth);
        let cut = tail.cut(saved, settings).unwrap().unwrap();
        assert!(cut.full);
        assert_eq!(cut.through.sequence, 5);
        assert!(tail.cut(cut.through, settings).unwrap().is_none());
        assert!(!tail.running);
    }
}
