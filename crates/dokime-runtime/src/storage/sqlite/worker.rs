/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Serialized SQLite Operations, Durable Group Replies And Passive Maintenance

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::VecDeque;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Instant;

// Third Party Imports
use rusqlite::Connection;
use tokio::sync::oneshot;

// Local Imports
use super::{Error, Owner, State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Completion = Box<dyn FnOnce(Option<&str>) + Send>;
type Operation = Box<dyn FnOnce(&mut Connection) -> Completion + Send>;

/* === Structs === */

#[derive(Default)]
pub(super) struct Queue {
    pending: Mutex<Pending>,
    checkpoint_running: AtomicBool,
    tasks: Mutex<Vec<Task>>,
    groups: AtomicU64,
    written_pages: AtomicU64,
}

#[derive(Default)]
struct Pending {
    jobs: VecDeque<Job>,
    running: bool,
}

struct Job {
    rows: usize,
    bytes: usize,
    grouped: bool,
    operation: Operation,
}

struct Task {
    join: tokio::task::JoinHandle<()>,
    finished: std::sync::mpsc::Receiver<()>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(super) async fn grouped<T: Send + 'static>(
        &self, rows: usize, bytes: usize, operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        if rows > self.0.settings.commit.rows.get() || bytes > self.0.settings.commit.bytes.get() {
            return Err(Error::Invalid("write exceeds SQLite commit group bounds"));
        }
        self.enqueue(rows, bytes, true, operation).await
    }

    pub(super) async fn enqueue<T: Send + 'static>(
        &self, rows: usize, bytes: usize, grouped: bool,
        operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        let (reply, response) = oneshot::channel();
        let parent = tracing::Span::current();
        let operation = Box::new(move |connection: &mut Connection| -> Completion {
            if reply.is_closed() {
                return Box::new(|_| {});
            }
            let _parent = parent.enter();
            let result = operation(connection).map_err(Error::Sqlite);
            Box::new(move |failure| {
                let result = match failure {
                    Some(failure) => Err(Error::Io(std::io::Error::other(failure.to_owned()))),
                    None => result,
                };
                let _ = reply.send(result);
            })
        });
        {
            let mut pending = self.0.queue.pending.lock().unwrap();
            if self.0.closing.load(Ordering::Acquire) {
                return Err(Error::Closed);
            }
            pending.jobs.push_back(Job { rows, bytes, grouped, operation });
            if !pending.running {
                pending.running = true;
                let owner = self.0.owner.clone();
                self.0.queue.spawn(move || {
                    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(owner.clone()))).is_err() {
                        owner.failed.store(true, Ordering::Release);
                        owner.closing.store(true, Ordering::Release);
                        owner.queue.pending.lock().unwrap().jobs.clear();
                        finish_failed(&owner);
                    }
                });
            }
        }
        response.await.map_err(|_| Error::Closed)?
    }
}

impl Queue {
    fn spawn(&self, operation: impl FnOnce() + Send + 'static) {
        let (finished, receiver) = std::sync::mpsc::channel();
        let mut tasks = self.tasks.lock().unwrap();
        tasks.retain(|task| !task.join.is_finished());
        let join = tokio::task::spawn_blocking(move || {
            // Disconnect Only After The Operation And Its Owner Captures Are Dropped.
            let _finished = finished;
            operation();
        });
        tasks.push(Task { join, finished: receiver });
    }

    pub(super) fn wait(&self) {
        loop {
            let tasks = {
                let mut tasks = self.tasks.lock().unwrap();
                // A Callback May Drop Its Last Public Handle On The Worker Itself.
                // That Worker Already Owns The Remaining Drain And Resource Release.
                if tasks.iter().any(|task| tokio::task::try_id() == Some(task.join.id())) {
                    return;
                }
                std::mem::take(&mut *tasks)
            };
            if tasks.is_empty() {
                return;
            }
            for task in tasks {
                let _ = task.finished.recv();
            }
        }
    }

    pub(super) async fn settle(&self) {
        // An Enqueue Admitted Before Closing Must Register Its Worker Before We Join.
        drop(self.pending.lock().unwrap());
        std::future::poll_fn(|context| {
            let mut tasks = self.tasks.lock().unwrap();
            let mut index = 0;
            while index < tasks.len() {
                if std::future::Future::poll(std::pin::Pin::new(&mut tasks[index].join), context).is_ready() {
                    tasks.swap_remove(index);
                } else {
                    index += 1;
                }
            }
            if tasks.is_empty() { std::task::Poll::Ready(()) } else { std::task::Poll::Pending }
        })
        .await;
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn run(owner: Arc<Owner>) {
    loop {
        let mut jobs = Vec::new();
        {
            let mut pending = owner.queue.pending.lock().unwrap();
            let Some(first) = pending.jobs.pop_front() else {
                pending.running = false;
                return;
            };
            let mut rows = first.rows;
            let mut bytes = first.bytes;
            let grouped = first.grouped;
            jobs.push(first);
            while grouped && let Some(next) = pending.jobs.front() {
                if !next.grouped
                    || next.rows > owner.settings.commit.rows.get() - rows
                    || next.bytes > owner.settings.commit.bytes.get() - bytes
                {
                    break;
                }
                rows += next.rows;
                bytes += next.bytes;
                jobs.push(pending.jobs.pop_front().unwrap());
            }
        }
        let grouped = jobs[0].grouped;
        let mut live = owner.live.blocking_lock();
        let Some(live) = live.as_mut() else {
            // Dropping Unexecuted Jobs Closes Their Responses Without Acknowledgement.
            drop(jobs);
            continue;
        };
        let connection = &mut live.connection;
        if grouped && let Err(error) = connection.execute_batch("BEGIN IMMEDIATE") {
            tracing::error!(%error, "cannot begin SQLite commit group");
            drop(jobs);
            continue;
        }
        if let Err(error) = live.probe.attach(connection) {
            tracing::warn!(%error, "SQLite sync probe could not attach");
        }
        let mut completions = Vec::with_capacity(jobs.len());
        let operations = super::probe::Timing::start("operations");
        for job in jobs {
            completions.push((job.operation)(connection));
        }
        drop(operations);
        let result = if grouped {
            let _commit = super::probe::Timing::start("commit");
            let started = Instant::now();
            let result = owner.publication.lock().unwrap().sync().and_then(|()| connection.execute_batch("COMMIT").map_err(Error::Sqlite));
            super::COMMIT_NS.fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
            if result.is_ok() {
                super::COMMITTED_TRANSACTIONS.fetch_add(1, Ordering::Relaxed);
            }
            result
        } else {
            Ok(())
        };
        let failure = result.err().map(|error| error.to_string());
        if failure.is_some() {
            let _ = connection.execute_batch("ROLLBACK");
            // In-Memory Writers May Have Advanced; Reopening Reconciles Their Durable Cut.
            owner.failed.store(true, Ordering::Release);
            owner.closing.store(true, Ordering::Release);
        }
        if failure.is_none() && grouped {
            owner.publication.lock().unwrap().committed();
        }
        for complete in completions {
            complete(failure.as_deref());
        }
        if failure.is_some() {
            owner.queue.pending.lock().unwrap().jobs.clear();
            return finish_failed(&owner);
        }
        let mut pages = 0;
        let mut highwater = 0;
        // SAFETY: The Worker Exclusively Owns This Live Connection And Both Output Slots.
        let status = unsafe {
            rusqlite::ffi::sqlite3_db_status(connection.handle(), rusqlite::ffi::SQLITE_DBSTATUS_CACHE_WRITE, &mut pages, &mut highwater, 1)
        };
        if status == rusqlite::ffi::SQLITE_OK && pages > 0 {
            let pages = owner.queue.written_pages.fetch_add(pages as u64, Ordering::Relaxed) + pages as u64;
            let groups = owner.queue.groups.fetch_add(1, Ordering::Relaxed) + 1;
            if pages >= u64::from(owner.settings.checkpoint_pages.get())
                || groups.is_multiple_of(owner.settings.commit.checkpoint_groups.get() as u64)
            {
                checkpoint(&owner);
            }
        }
    }
}

fn finish_failed(owner: &Owner) {
    owner.queue.pending.lock().unwrap().running = false;
}

fn checkpoint(owner: &Arc<Owner>) {
    if owner.queue.checkpoint_running.swap(true, Ordering::AcqRel) {
        return;
    }
    owner.queue.written_pages.store(0, Ordering::Relaxed);
    let checkpoint_owner = owner.clone();
    owner.queue.spawn(move || {
        let _checkpoint = super::probe::Timing::start("checkpoint");
        let started = Instant::now();
        let result = (|| -> rusqlite::Result<()> {
            let mut probe = super::probe::Probe::default();
            let connection = Connection::open(&checkpoint_owner.readers.database)?;
            connection.busy_timeout(std::time::Duration::ZERO)?;
            connection.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;
            probe.attach(&connection)?;
            connection.pragma_update(None, "synchronous", "FULL")?;
            connection.execute_batch("PRAGMA wal_checkpoint(PASSIVE)")
        })();
        super::CHECKPOINT_NS.fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
        super::CHECKPOINTS.fetch_add(1, Ordering::Relaxed);
        if let Err(error) = result {
            tracing::warn!(%error, "SQLite maintenance checkpoint failed; WAL remains durable");
        }
        checkpoint_owner.queue.checkpoint_running.store(false, Ordering::Release);
    });
}

/* === Tests === */
