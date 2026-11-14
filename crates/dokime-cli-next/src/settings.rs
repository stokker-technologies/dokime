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

// Standard Library Imports
use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use serde::Deserialize;

// Local Imports
use dokime_runtime_next::{
    ControlledClock, HostSettings, PlacementSettings, SchedulerClock,
    services::{ClientSettings, WallClock},
};

use dokime_storage::StorageSettings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Settings {
    pub endpoint: Option<String>,

    #[serde(skip)]
    pub remote: Option<PathBuf>,

    #[serde(skip)]
    pub pairing: Option<crate::commands::pairing::File>,
    pub storage: Option<PathBuf>,
    pub telemetry_rows: NonZeroUsize,
    pub telemetry_interval_ms: u64,
    pub telemetry_pending_bytes: NonZeroUsize,
    pub telemetry_query_bytes: NonZeroUsize,
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
            remote: None,
            pairing: None,
            storage: None,
            telemetry_rows: dokime_storage::storage_defaults().flush_rows,
            telemetry_interval_ms: dokime_storage::storage_defaults().flush_interval.as_millis() as u64,
            telemetry_pending_bytes: dokime_storage::storage_defaults().pending_bytes,
            telemetry_query_bytes: dokime_storage::storage_defaults().query_bytes,
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
            settings.telemetry_interval_ms,
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
        HostSettings {
            clock,
            placement: self.placement(),
            storage: StorageSettings {
                flush_rows: self.telemetry_rows,
                flush_interval: Duration::from_millis(self.telemetry_interval_ms),
                pending_bytes: self.telemetry_pending_bytes,
                query_bytes: self.telemetry_query_bytes,
                previews: dokime_storage::storage_defaults().previews,
            },
        }
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
    pub fn client(&self) -> dokime::client::ClientSettings {
        dokime::client::ClientSettings {
            transfer: dokime::client::StorageTransferSettings { chunk_bytes: Some(1024) },
            retained_bytes: self.retained_bytes,
            message_bytes: self.message_bytes,
            input_items: self.stream_buffer,
            input_bytes: self.message_bytes,
        }
    }
    pub fn request<T>(&self, value: T) -> tonic::Request<T> {
        let mut request = tonic::Request::new(value);
        if let Some(pairing) = &self.pairing {
            request.metadata_mut().insert("authorization", pairing.authorization().expect("Validated Pairing Key"));
        }
        request
    }
    pub async fn sdk(&self, endpoint: Option<String>, streaming: bool) -> crate::Result<dokime::Client> {
        let channel = self.channel(endpoint, streaming).await?;
        let mut client = dokime::Client::from_channel(channel, self.client())?;
        if let Some(pairing) = &self.pairing {
            client = client.authorization(pairing.authorization()?);
        }
        Ok(client)
    }
    pub fn request_id(&self, supplied: Option<String>) -> crate::Result<String> {
        if let Some(value) = supplied {
            if value.is_empty() {
                return Err(crate::error::Error::Invalid("Mutate Resource: Supply a Nonempty Request ID".into()));
            }
            return Ok(value);
        }
        let nonce = tempfile::Builder::new().prefix("dokime-request-").tempfile()?;
        Ok(nonce.path().file_name().unwrap().to_string_lossy().into_owned())
    }
    pub fn interceptor(&self) -> impl tonic::service::Interceptor + Clone + use<> {
        let authorization = self.pairing.as_ref().map(|pairing| pairing.authorization().expect("Validated Pairing Key"));
        move |mut request: tonic::Request<()>| {
            if let Some(authorization) = &authorization {
                request.metadata_mut().insert("authorization", authorization.clone());
            }
            Ok(request)
        }
    }
    pub async fn channel(&self, endpoint: Option<String>, streaming: bool) -> crate::Result<tonic::transport::Channel> {
        if let Some(pairing) = &self.pairing {
            return pairing.channel(self, streaming).await;
        }
        let endpoint = endpoint.ok_or_else(|| {
            crate::error::Error::Invalid("Supply --remote FILE, Set DOKIME_REMOTE, or Supply --endpoint http://HOST:PORT".into())
        })?;
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
