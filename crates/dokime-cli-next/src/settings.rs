/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Command-Line and Host Configuration

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use dokime_runtime_next::{
    ControlledClock, HostSettings, PlacementSettings, SchedulerClock,
    services::{ClientSettings, WallClock},
};
use serde::Deserialize;
use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Settings {
    pub endpoint: Option<String>,
    pub storage: Option<PathBuf>,
    pub connect_timeout_s: u64,
    pub read_timeout_s: u64,
    pub mutation_timeout_s: u64,
    pub watch_timeout_s: u64,
    pub reconnect_delay_ms: u64,
    pub stop_grace_s: u64,
    pub message_bytes: NonZeroUsize,
    pub retained_bytes: NonZeroUsize,
    pub stream_buffer: NonZeroUsize,
    pub max_cases: NonZeroUsize,
    pub page_items: NonZeroUsize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for Settings {
    fn default() -> Self {
        Self {
            endpoint: None,
            storage: None,
            connect_timeout_s: 30,
            read_timeout_s: 30,
            mutation_timeout_s: 30,
            watch_timeout_s: 3600,
            reconnect_delay_ms: 25,
            stop_grace_s: 30,
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            retained_bytes: NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
            stream_buffer: NonZeroUsize::new(8).unwrap(),
            max_cases: NonZeroUsize::new(1000).unwrap(),
            page_items: NonZeroUsize::new(1000).unwrap(),
        }
    }
}
impl Settings {
    pub fn load(path: Option<&Path>) -> crate::Result<Self> {
        let settings: Self = match path {
            Some(path) => toml::from_str(&std::fs::read_to_string(path)?)?,
            None => Self::default(),
        };
        if [
            settings.connect_timeout_s,
            settings.read_timeout_s,
            settings.mutation_timeout_s,
            settings.watch_timeout_s,
            settings.stop_grace_s,
        ]
        .contains(&0)
        {
            return Err("Configure Positive Host and Client Timeouts".into());
        }
        Ok(settings)
    }
    pub fn placement(&self) -> PlacementSettings {
        PlacementSettings {
            connect_timeout: Duration::from_secs(self.connect_timeout_s),
            reconnect_delay: Duration::from_millis(self.reconnect_delay_ms),
            stop_grace: Duration::from_secs(self.stop_grace_s),
            stream_window: self.stream_buffer,
            message_bytes: self.message_bytes,
        }
    }
    pub fn host(&self, controlled: bool) -> HostSettings {
        let clock: Arc<dyn SchedulerClock> = if controlled { Arc::new(ControlledClock::new(0)) } else { Arc::new(WallClock) };
        HostSettings { clock, placement: self.placement() }
    }
    pub fn services(&self) -> ClientSettings {
        ClientSettings {
            read_timeout: Duration::from_secs(self.read_timeout_s),
            mutation_timeout: Duration::from_secs(self.mutation_timeout_s),
            watch_timeout: Duration::from_secs(self.watch_timeout_s),
            max_cases: self.max_cases.get(),
            page_items: self.page_items.get(),
            message_bytes: self.message_bytes.get(),
            stream_buffer: self.stream_buffer.get(),
        }
    }
    pub async fn channel(&self, endpoint: Option<String>, streaming: bool) -> crate::Result<tonic::transport::Channel> {
        let endpoint = endpoint.ok_or("Supply --endpoint http://HOST:PORT or Configure endpoint")?;
        let timeout = if streaming { self.watch_timeout_s } else { self.mutation_timeout_s.max(self.read_timeout_s) };
        let channel = tonic::transport::Endpoint::from_shared(endpoint)?
            .connect_timeout(Duration::from_secs(self.connect_timeout_s))
            .timeout(Duration::from_secs(timeout))
            .connect()
            .await?;
        Ok(channel)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
