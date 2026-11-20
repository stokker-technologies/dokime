/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Collector State Shared by Its Observers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use crate::domain::telemetry::Bluetooth;
use dokime::encoding::Message;
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Bluetooth Collection State Retained Between Observer Ticks.
#[derive(Default)]
pub struct Collector {
    state: Arc<Mutex<State>>,
}

/// What the Bluetooth Collection Reads From the Plugin Settings.
pub struct Options {
    pub adapter: Option<String>,
    pub probe_timeout: Duration,
    pub deep_refresh_interval: Duration,
}

/// The Sample Count and the Last Native Inventory, Reused Between Deep Refreshes.
#[derive(Default)]
pub struct State {
    sample_index: u64,
    last_deep_refresh: Option<Instant>,
    cached: Option<Vec<u8>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Collector {
    /// One Bluetooth Row; the Native Inventory Runs Only When a Deep Refresh Is Due.
    pub async fn collect(&self, options: Options) -> Result<Bluetooth> {
        let state = Arc::clone(&self.state);
        tokio::task::spawn_blocking(move || {
            let mut state = state
                .lock()
                .map_err(|_| Error::Invalid("Collect Bluetooth Telemetry: a previous collection panicked; Restart the Plugin".into()))?;
            state.collect_at(Instant::now(), options.deep_refresh_interval, || inventory(&options))
        })
        .await
        .map_err(|error| {
            Error::Invalid(format!("Collect Bluetooth Telemetry: the collection worker failed: {error}; Restart the Plugin"))
        })?
    }
}

impl State {
    /// One Row at `now`: a Fresh Inventory When the Deep Refresh Is Due, Else the Cached One; a Failed Refresh Reuses the Cache as Stale.
    pub fn collect_at(
        &mut self, now: Instant, deep_refresh_interval: Duration, probe: impl FnOnce() -> std::result::Result<Bluetooth, String>,
    ) -> Result<Bluetooth> {
        self.sample_index += 1;
        let refresh = self.last_deep_refresh.is_none_or(|last| now.duration_since(last) >= deep_refresh_interval);
        let (mut row, status) = if refresh {
            self.last_deep_refresh = Some(now);
            match probe() {
                Ok(snapshot) => {
                    let encoded = snapshot.encode_to_vec();
                    self.cached = Some(encoded);
                    (snapshot, "ok")
                }
                Err(error) => {
                    let (mut row, status) = match self.cached()? {
                        Some(cached) => (cached, "stale"),
                        None => (Bluetooth::default(), "error"),
                    };
                    row.probe_error = Some(error);
                    (row, status)
                }
            }
        } else {
            match self.cached()? {
                Some(cached) => (cached, "ok"),
                None => (Bluetooth::default(), "unavailable"),
            }
        };
        row.sample_index = Some(i64::try_from(self.sample_index).unwrap_or(i64::MAX));
        row.probe_status = Some(status.into());
        Ok(row)
    }

    fn cached(&self) -> Result<Option<Bluetooth>> {
        let Some(cached) = &self.cached else {
            return Ok(None);
        };
        Bluetooth::decode(cached.as_slice())
            .map(Some)
            .map_err(|error| Error::Invalid(format!("Reuse the Bluetooth Inventory: {error}; Restart the Plugin")))
    }
}

/* === Public Functions === */

/* === Private Functions === */

#[cfg(windows)]
fn inventory(options: &Options) -> std::result::Result<Bluetooth, String> {
    crate::platform::windows::bluetooth::snapshot(options.adapter.as_deref())
}

#[cfg(unix)]
fn inventory(options: &Options) -> std::result::Result<Bluetooth, String> {
    crate::platform::unix::bluetooth::snapshot(options.adapter.as_deref(), options.probe_timeout)
}

/* === Tests === */
