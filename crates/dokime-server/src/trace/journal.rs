/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Per-Scope Rings Of Captured Tracing Events
//
// Every captured event enters the instance ring, and also the ring of the
// cycle and the execution it belongs to. Each ring evicts its oldest events
// first when it exceeds its event or byte bound, so a chatty instance never
// pushes a quiet cycle's lines out. A ring numbers its events with its own
// contiguous sequence, which is what makes an eviction gap exact to count for
// any reader holding a cursor into that ring.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use time::OffsetDateTime;
use tokio::sync::watch;

const SERVER_TRACE_CYCLES: u64 = 32;
const SERVER_TRACE_EXECUTIONS: u64 = 128;
const SERVER_TRACE_INSTANCE_BYTES: u64 = 8_388_608;
const SERVER_TRACE_INSTANCE_EVENTS: u64 = 8_192;
const SERVER_TRACE_LINEAGE: u64 = 1_024;
const SERVER_TRACE_SCOPE_BYTES: u64 = 2_097_152;
const SERVER_TRACE_SCOPE_EVENTS: u64 = 4_096;

#[derive(Debug)]
pub struct Event {
    pub time: OffsetDateTime,
    pub level: tracing::Level,
    pub target: String,
    pub message: String,
    pub plugin: Option<String>,
    pub cycle: Option<String>,
    pub execution: Option<String>,
    pub fields: Vec<(String, String)>,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub instance_events: usize,
    pub instance_bytes: usize,
    pub scope_events: usize,
    pub scope_bytes: usize,
    pub cycles: usize,
    pub executions: usize,
    pub lineage: usize,
}

#[derive(Debug, Default)]
pub struct Page {
    pub events: Vec<(u64, Arc<Event>)>,
    pub missed: u64,
    pub next: u64,
}

pub struct Journal {
    limits: Limits,
    state: Mutex<State>,
    head: watch::Sender<u64>,
    installed: AtomicBool,
}

struct State {
    sequence: u64,
    instance: Ring,
    cycles: Rings,
    executions: Rings,
    lineage: Lineage,
}

struct Ring {
    // Sequences Of An Earlier Incarnation Of This Scope All Lie At Or Below It.
    base: u64,
    front: u64,
    entries: VecDeque<Arc<Event>>,
    bytes: usize,
    written: u64,
}

struct Rings {
    capacity: usize,
    rings: HashMap<String, Ring>,
}

struct Lineage {
    capacity: usize,
    cycles: HashMap<String, String>,
    order: VecDeque<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("trace sequence {after} was never issued by this backend process (head {head})")]
pub struct Stale {
    pub after: u64,
    pub head: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    Instance,
    Cycle(String),
    Execution(String),
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            instance_events: usize::try_from(SERVER_TRACE_INSTANCE_EVENTS).expect("validated trace instance count"),
            instance_bytes: usize::try_from(SERVER_TRACE_INSTANCE_BYTES).expect("validated trace instance bytes"),
            scope_events: usize::try_from(SERVER_TRACE_SCOPE_EVENTS).expect("validated trace scope count"),
            scope_bytes: usize::try_from(SERVER_TRACE_SCOPE_BYTES).expect("validated trace scope bytes"),
            cycles: usize::try_from(SERVER_TRACE_CYCLES).expect("validated trace cycle count"),
            executions: usize::try_from(SERVER_TRACE_EXECUTIONS).expect("validated trace execution count"),
            lineage: usize::try_from(SERVER_TRACE_LINEAGE).expect("validated trace lineage count"),
        }
    }
}

impl std::fmt::Debug for Journal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Journal").field("limits", &self.limits).field("installed", &self.installed()).finish_non_exhaustive()
    }
}

impl Event {
    pub fn bytes(&self) -> usize {
        let bytes = std::mem::size_of::<Self>()
            .saturating_add(self.target.capacity())
            .saturating_add(self.message.capacity())
            .saturating_add(self.fields.capacity().saturating_mul(std::mem::size_of::<(String, String)>()));
        let bytes = [&self.plugin, &self.cycle, &self.execution]
            .into_iter()
            .flatten()
            .fold(bytes, |bytes, value| bytes.saturating_add(value.capacity()));
        self.fields.iter().fold(bytes, |bytes, (name, value)| bytes.saturating_add(name.capacity()).saturating_add(value.capacity()))
    }
}

impl Journal {
    pub fn new(limits: Limits) -> Arc<Self> {
        Arc::new(Self {
            limits,
            state: Mutex::new(State {
                sequence: 0,
                instance: Ring::new(0),
                cycles: Rings::new(limits.cycles),
                executions: Rings::new(limits.executions),
                lineage: Lineage::new(limits.lineage),
            }),
            head: watch::Sender::new(0),
            installed: AtomicBool::new(false),
        })
    }

    pub fn installed(&self) -> bool {
        self.installed.load(Ordering::Acquire)
    }

    pub(super) fn mark_installed(&self) {
        self.installed.store(true, Ordering::Release);
    }

    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.head.subscribe()
    }

    pub fn record(&self, mut event: Event) {
        let sequence = {
            let mut state = self.lock();
            state.sequence += 1;
            let sequence = state.sequence;

            // Resolve Scope Membership
            match (&event.cycle, &event.execution) {
                (Some(cycle), Some(execution)) => state.lineage.learn(execution, cycle),
                (None, Some(execution)) => {
                    event.cycle = state.lineage.cycle(execution).cloned();
                }
                _ => {}
            }

            // Append To Every Ring The Event Belongs To
            let event = Arc::new(event);
            let limits = self.limits;
            state.instance.push(&event, sequence, limits.instance_events, limits.instance_bytes);
            if let Some(cycle) = &event.cycle {
                state.cycles.ring(cycle, sequence).push(&event, sequence, limits.scope_events, limits.scope_bytes);
            }
            if let Some(execution) = &event.execution {
                state.executions.ring(execution, sequence).push(&event, sequence, limits.scope_events, limits.scope_bytes);
            }
            sequence
        };
        self.head.send_replace(sequence);
    }

    pub fn read(
        &self, scope: &Scope, after: u64, accept: impl Fn(&Event) -> bool, max_events: usize, max_bytes: usize,
    ) -> Result<Page, Stale> {
        self.read_until(scope, after, accept, max_events, max_bytes, u64::MAX)
    }

    pub fn read_until(
        &self, scope: &Scope, after: u64, accept: impl Fn(&Event) -> bool, max_events: usize, max_bytes: usize, through: u64,
    ) -> Result<Page, Stale> {
        let state = self.lock();
        if after > state.sequence {
            return Err(Stale { after, head: state.sequence });
        }
        let Some(ring) = state.ring(scope) else {
            if after != 0 {
                return Err(Stale { after, head: 0 });
            }
            return Ok(Page {
                events: Vec::new(),
                missed: 0,
                next: after,
            });
        };

        let tail = ring.end().saturating_sub(1);
        if after > tail || (after != 0 && after < ring.base) || (through != 0 && through < ring.base) {
            return Err(Stale { after, head: tail });
        }

        // Count What Eviction Took Ahead Of The Cursor
        let wanted = after.max(ring.base) + 1;
        let end = ring.end().min(through.saturating_add(1));
        let missed = ring.front.min(end).saturating_sub(wanted);
        let mut sequence = wanted.max(ring.front).min(end).max(after + 1);

        // Scan Forward Within The Page Bounds
        let mut events = Vec::new();
        let mut bytes = 0_usize;
        while sequence < end && events.len() < max_events {
            let event = &ring.entries[(sequence - ring.front) as usize];
            if accept(event) {
                let size = event.bytes();
                if !events.is_empty() && bytes.saturating_add(size) > max_bytes {
                    break;
                }
                bytes = bytes.saturating_add(size);
                events.push((sequence, Arc::clone(event)));
            }
            sequence += 1;
        }
        Ok(Page { events, missed, next: sequence - 1 })
    }

    pub fn tail(&self, scope: &Scope) -> u64 {
        self.lock().ring(scope).map_or(0, |ring| ring.end().saturating_sub(1).max(ring.base))
    }

    pub fn evicted(&self, scope: &Scope) -> u64 {
        self.lock().ring(scope).map_or(0, |ring| ring.front - ring.base - 1)
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        // A Panic Elsewhere Never Leaves A Ring Half-Written: Pushes Are Infallible.
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl State {
    fn ring(&self, scope: &Scope) -> Option<&Ring> {
        match scope {
            Scope::Instance => Some(&self.instance),
            Scope::Cycle(id) => self.cycles.rings.get(id),
            Scope::Execution(id) => self.executions.rings.get(id),
        }
    }
}

impl Ring {
    fn new(base: u64) -> Self {
        Self {
            base,
            front: base + 1,
            entries: VecDeque::new(),
            bytes: 0,
            written: base,
        }
    }

    fn end(&self) -> u64 {
        self.front + self.entries.len() as u64
    }

    fn push(&mut self, event: &Arc<Event>, written: u64, events: usize, bytes: usize) {
        let size = event.bytes();
        if events == 0 || size > bytes {
            self.front = self.end() + 1;
            self.entries.clear();
            self.bytes = 0;
            self.written = written;
            return;
        }
        while !self.entries.is_empty() && (self.entries.len() >= events || self.bytes.saturating_add(size) > bytes) {
            if let Some(evicted) = self.entries.pop_front() {
                self.bytes = self.bytes.saturating_sub(evicted.bytes());
                self.front += 1;
            }
        }
        self.bytes = self.bytes.saturating_add(size);
        self.entries.push_back(Arc::clone(event));
        self.written = written;
    }
}

impl Rings {
    fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            rings: HashMap::new(),
        }
    }

    fn ring(&mut self, id: &str, sequence: u64) -> &mut Ring {
        if !self.rings.contains_key(id) {
            if self.rings.len() >= self.capacity
                && let Some(stale) = self.rings.iter().min_by_key(|(_, ring)| ring.written).map(|(id, _)| id.clone())
            {
                self.rings.remove(&stale);
            }
            // A Recreated Scope Continues Above Every Sequence Its Earlier Ring Issued.
            self.rings.insert(id.to_owned(), Ring::new(sequence - 1));
        }
        self.rings.get_mut(id).expect("the ring was just found or inserted")
    }
}

impl Lineage {
    fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            cycles: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn learn(&mut self, execution: &str, cycle: &str) {
        if self.cycles.contains_key(execution) {
            return;
        }
        if self.order.len() >= self.capacity
            && let Some(oldest) = self.order.pop_front()
        {
            self.cycles.remove(&oldest);
        }
        self.cycles.insert(execution.to_owned(), cycle.to_owned());
        self.order.push_back(execution.to_owned());
    }

    fn cycle(&self, execution: &str) -> Option<&String> {
        self.cycles.get(execution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(cycle: Option<&str>, execution: Option<&str>, message: &str) -> Event {
        Event {
            time: OffsetDateTime::UNIX_EPOCH,
            level: tracing::Level::INFO,
            target: "fixture".into(),
            message: message.into(),
            plugin: None,
            cycle: cycle.map(str::to_owned),
            execution: execution.map(str::to_owned),
            fields: Vec::new(),
        }
    }

    #[test]
    fn journal_counts_retained_capacity() {
        let mut retained = event(None, None, "x");
        retained.message.reserve_exact(1024);
        let journal = Journal::new(Limits { instance_bytes: 512, ..Limits::default() });
        journal.record(retained);
        let page = journal.read(&Scope::Instance, 0, |_| true, 1, 512).unwrap();
        assert!(page.events.is_empty());
        assert_eq!(page.missed, 1);
    }
    #[test]
    fn journal_bounds_retention() {
        let journal = Journal::new(Limits {
            instance_events: 2,
            scope_events: 2,
            cycles: 1,
            executions: 1,
            ..Limits::default()
        });
        let scope = Scope::Cycle("cycle-a".into());
        journal.record(event(Some("cycle-a"), Some("execution-a"), "one"));
        journal.record(event(None, Some("execution-a"), "two"));
        journal.record(event(None, Some("execution-a"), "three"));
        let page = journal.read(&scope, 0, |_| true, 2, 4096).unwrap();
        assert_eq!(page.missed, 1);
        assert_eq!(page.next, 3);
        assert_eq!(page.events.iter().map(|(_, event)| event.message.as_str()).collect::<Vec<_>>(), ["two", "three"]);
        assert_eq!(page.events[0].1.cycle.as_deref(), Some("cycle-a"));
        let selected = journal.read(&scope, 1, |_| false, 2, 4096).unwrap();
        assert!(selected.events.is_empty());
        assert_eq!(selected.next, 3);
        let snapshot = journal.read_until(&scope, 0, |_| true, 2, 4096, 2).unwrap();
        assert_eq!(snapshot.events.len(), 1);
        assert_eq!(snapshot.next, 2);
        assert!(journal.read(&scope, 4, |_| true, 2, 4096).is_err());
        journal.record(event(Some("cycle-b"), None, "other"));
        assert!(journal.read(&scope, 3, |_| true, 2, 4096).is_err());
        journal.record(event(Some("cycle-a"), None, "recreated"));
        assert!(journal.read(&scope, 3, |_| true, 2, 4096).is_err());
        assert_eq!(journal.read(&scope, 0, |_| true, 2, 4096).unwrap().events[0].0, 5);

        let bounded = Journal::new(Limits {
            instance_bytes: 128,
            scope_bytes: 128,
            ..Limits::default()
        });
        bounded.record(event(Some("cycle-a"), None, &"large".repeat(100)));
        let page = bounded.read(&Scope::Instance, 0, |_| true, 1, 128).unwrap();
        assert!(page.events.is_empty());
        assert_eq!(page.missed, 1);
    }
}
