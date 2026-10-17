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

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
// Third Party Imports
use time::OffsetDateTime;
use tokio::sync::watch;
// Local Imports

/* === Constants === */
const EVENT_OVERHEAD_BYTES: usize = 96;

const FIELD_OVERHEAD_BYTES: usize = 16;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// One captured event, already redacted and truncated at capture.
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

/// The bounds of one journal: per-ring event and byte bounds, how many cycle
/// and execution rings live at once, and how many execution-to-cycle
/// associations are remembered.
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

/// One read: the accepted events with their scope sequences, how many events
/// of the scope were evicted before this read reached them, and the last
/// sequence scanned.
#[derive(Debug, Default)]
pub struct Page {
    pub events: Vec<(u64, Arc<Event>)>,
    pub missed: u64,
    pub next: u64,
}

/// The process's captured tracing output.
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

/* === Enums === */
/// Which ring a reader follows.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    Instance,
    Cycle(String),
    Execution(String),
}

/// A cursor this journal never issued, typically one from before a restart.
#[derive(Debug, thiserror::Error)]
#[error("trace sequence {after} was never issued by this backend process (head {head})")]
pub struct Stale {
    pub after: u64,
    pub head: u64,
}

/* === Traits === */

/* === Implementations === */
impl Default for Limits {
    fn default() -> Self {
        Self {
            instance_events: 8_192,
            instance_bytes: 8 * 1024 * 1024,
            scope_events: 4_096,
            scope_bytes: 2 * 1024 * 1024,
            cycles: 32,
            executions: 128,
            lineage: 1_024,
        }
    }
}

impl Event {
    /// A conservative estimate of the event's encoded size, used for bounds.
    pub fn bytes(&self) -> usize {
        let ids = [&self.plugin, &self.cycle, &self.execution].into_iter().flatten().map(String::len).sum::<usize>();
        let fields = self.fields.iter().map(|(name, value)| name.len() + value.len() + FIELD_OVERHEAD_BYTES).sum::<usize>();
        EVENT_OVERHEAD_BYTES + self.target.len() + self.message.len() + ids + fields
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

    /// The journal the process-wide subscriber captures into.
    pub fn global() -> &'static Arc<Self> {
        static GLOBAL: OnceLock<Arc<Journal>> = OnceLock::new();
        GLOBAL.get_or_init(|| Self::new(Limits::default()))
    }

    /// Whether a capture layer feeds this journal; an unfed journal would
    /// answer every reader with silence that looks like a quiet backend.
    pub fn installed(&self) -> bool {
        self.installed.load(Ordering::Acquire)
    }

    pub(super) fn mark_installed(&self) {
        self.installed.store(true, Ordering::Release);
    }

    /// Wakes whenever any event is captured.
    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.head.subscribe()
    }

    /// Appends one event to the instance ring and to its scope rings.
    ///
    /// An execution seen with its cycle teaches the journal that association,
    /// so the execution's later events, which carry only their own id, still
    /// reach the cycle ring.
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

    /// Accepted events of `scope` after `after`, at most `max_events` and,
    /// beyond the first, within `max_bytes`.
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
            return Ok(Page { events: Vec::new(), missed: 0, next: after });
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

    /// The last sequence `scope` has issued, or zero before its first event.
    pub fn tail(&self, scope: &Scope) -> u64 {
        self.lock().ring(scope).map_or(0, |ring| ring.end().saturating_sub(1).max(ring.base))
    }

    /// Events evicted from `scope` since its ring was created.
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
        Self { base, front: base + 1, entries: VecDeque::new(), bytes: 0, written: base }
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
        Self { capacity: capacity.max(1), rings: HashMap::new() }
    }

    /// The ring of `id`, created on its first event. A new ring replaces the
    /// least recently written one once the bound is reached.
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
        Self { capacity: capacity.max(1), cycles: HashMap::new(), order: VecDeque::new() }
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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
