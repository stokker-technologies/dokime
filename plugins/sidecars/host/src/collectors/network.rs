/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Network Collector: Adapter Throughput With the Next Link, Address or Default-Route Change per Row

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

// Third Party Imports
use sysinfo::{NetworkData, Networks};

// Local Imports
use crate::adapters::{self, AdapterStatus};
use crate::domain::readings::Network;
#[cfg(unix)]
use crate::platform::unix::connectivity;
#[cfg(windows)]
use crate::platform::windows::connectivity;
use crate::settings;
use dokime::operations::Context;
use dokime::{Error, Result};

/* === Constants === */

/// Changes Queued Between Collections; a Longer Burst Is Reported Instead of Silently Dropped.
pub const PENDING_CHANGES: usize = 4_095;

const BYTES_PER_MEGABYTE: f64 = 1_000_000.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Network Collection State, Created by the First Collection of an Observed Stream.
#[derive(Default)]
pub struct Collector {
    state: Arc<Mutex<Option<State>>>,
}

/// One Native Network Change.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub kind: String,
    pub adapter_id: String,
    pub address: Option<String>,
}

/// Changes Awaiting Collection, Deduplicated Against Each Link's Last Known State.
pub struct Pending {
    events: VecDeque<Change>,
    links: BTreeMap<String, bool>,
    failed: bool,
}

struct State {
    sample_index: i64,
    networks: Networks,
    last_refresh: Instant,
    primary_interface: Option<String>,
    refresh_primary: bool,
    notifications: connectivity::Notifications,
}

/// The Settings One Collection Reads.
struct Timing {
    probe_timeout: Duration,
    minimum_rate_window: Duration,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Collector {
    /// Collect One Throughput Reading With Every Change Queued Since the Previous One.
    ///
    /// Native change watching starts with the first collection, so it runs only once the stream
    /// is observed.
    pub async fn collect(&self, ctx: &Context) -> Result<Network> {
        let timing = Timing {
            probe_timeout: settings::duration_value(ctx, settings::NETWORK_PROBE_TIMEOUT).await?,
            minimum_rate_window: settings::duration_value(ctx, settings::MINIMUM_RATE_WINDOW).await?,
        };
        let state = self.state.clone();
        let collected = tokio::task::spawn_blocking(move || {
            let mut state = state.lock().unwrap_or_else(PoisonError::into_inner);
            if state.is_none() {
                *state = Some(State::new(&timing)?);
            }
            let state = state.as_mut().expect("State Created Above");
            state.collect(&timing)
        })
        .await
        .map_err(|error| Error::Invalid(format!("Collect Network Telemetry: {error}; Restart the Plugin")))?;
        Ok(collected.unwrap_or_else(|error| Network {
            probe_status: Some("error".into()),
            probe_error: Some(error.to_string()),
            ..Network::default()
        }))
    }
}

impl Collector {
    /// Stop Watching Native Changes and Drop the Counters When the Last Listener Leaves.
    pub fn stop(&self) {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).take();
    }
}

impl Change {
    pub fn new(kind: &str, adapter_id: String, address: Option<String>) -> Self {
        Self { kind: kind.into(), adapter_id, address }
    }
}

impl Pending {
    pub fn new(links: BTreeMap<String, bool>) -> Self {
        Self {
            events: VecDeque::new(),
            links,
            failed: false,
        }
    }

    /// Queue a Change; a Link Change Repeating the Link's Known State Is No Change.
    pub fn push(&mut self, change: Change, up: Option<bool>) {
        if let Some(up) = up
            && self.links.insert(change.adapter_id.clone(), up) == Some(up)
        {
            return;
        }
        if self.events.len() >= PENDING_CHANGES {
            self.failed = true;
            return;
        }
        self.events.push_back(change);
    }

    pub fn fail(&mut self) {
        self.failed = true;
    }

    /// Take the Oldest Queued Change; After an Overflow or Native Lookup Failure, Report It Once and Start Over.
    pub fn pop(&mut self) -> Result<Option<Change>> {
        if std::mem::take(&mut self.failed) {
            self.events.clear();
            return Err(Error::Invalid(
                "Watch Network Changes: Notifications Overflowed or a Native Lookup Failed; Shorten the Network Interval".into(),
            ));
        }
        Ok(self.events.pop_front())
    }
}

impl State {
    fn new(timing: &Timing) -> Result<Self> {
        let links = adapters::native(timing.probe_timeout)?.adapters.into_iter().map(|adapter| {
            let up = adapter.status == AdapterStatus::Up.as_str();
            (adapter.id, up)
        });
        Ok(Self {
            sample_index: 0,
            networks: Networks::new_with_refreshed_list()
                .map_err(|error| Error::Invalid(format!("Read Network Adapters: {error}; Restart the Plugin")))?,
            last_refresh: Instant::now(),
            primary_interface: None,
            refresh_primary: true,
            notifications: connectivity::Notifications::new(links.collect())?,
        })
    }

    fn collect(&mut self, timing: &Timing) -> Result<Network> {
        let mut changes = Vec::new();
        while let Some(change) = self.notifications.take()? {
            changes.push(change);
        }
        if changes.iter().any(|change| change.kind == "default_route_changed") {
            self.refresh_primary = true;
        }
        self.sample_index += 1;
        let now = Instant::now();
        let elapsed_s = now.duration_since(self.last_refresh).as_secs_f64().max(timing.minimum_rate_window.as_secs_f64());
        self.networks.refresh(true);
        self.last_refresh = now;
        if self.refresh_primary {
            self.primary_interface =
                adapters::native(timing.probe_timeout)?.adapters.into_iter().find(|adapter| adapter.primary).map(|adapter| adapter.id);
            self.refresh_primary = false;
        }
        let sum = |value: fn(&NetworkData) -> u64| self.networks.values().map(value).sum::<u64>();
        let rate = |value: fn(&NetworkData) -> u64| sum(value) as f64 / elapsed_s / BYTES_PER_MEGABYTE;
        #[cfg(windows)]
        let internet = connectivity::internet();
        #[cfg(unix)]
        let internet = connectivity::internet(timing.probe_timeout);
        Ok(Network {
            sample_index: Some(self.sample_index),
            probe_status: Some("ok".into()),
            primary_interface: self.primary_interface.clone(),
            changes,
            network_interface_count: Some(self.networks.len() as i64),
            network_recv_mb_per_second: Some(rate(NetworkData::received)),
            network_sent_mb_per_second: Some(rate(NetworkData::transmitted)),
            network_total_recv_bytes: Some(integer(sum(NetworkData::total_received))),
            network_total_sent_bytes: Some(integer(sum(NetworkData::total_transmitted))),
            network_rx_errors: Some(integer(sum(NetworkData::errors_on_received))),
            network_tx_errors: Some(integer(sum(NetworkData::errors_on_transmitted))),
            internet,
            ..Network::default()
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn integer(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Change, PENDING_CHANGES, Pending};

    fn change(kind: &str) -> Change {
        Change::new(kind, "nic".into(), None)
    }

    fn kinds(pending: &mut Pending) -> Vec<String> {
        std::iter::from_fn(|| pending.pop().unwrap()).map(|change| change.kind).collect()
    }

    #[test]
    fn link_changes_repeating_the_known_state_are_dropped() {
        let mut pending = Pending::new(BTreeMap::from([("nic".into(), true)]));
        pending.push(change("link_up"), Some(true));
        pending.push(change("address_added"), None);
        pending.push(change("link_down"), Some(false));
        pending.push(change("link_down"), Some(false));
        assert_eq!(kinds(&mut pending), ["address_added", "link_down"]);
    }

    #[test]
    fn an_overflow_is_reported_once_and_collection_starts_over() {
        let mut pending = Pending::new(BTreeMap::new());
        for _ in 0..=PENDING_CHANGES {
            pending.push(change("default_route_changed"), None);
        }
        assert!(pending.pop().is_err());
        pending.push(change("address_removed"), None);
        assert_eq!(kinds(&mut pending), ["address_removed"]);
    }
}
