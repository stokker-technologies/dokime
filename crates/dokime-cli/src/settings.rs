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
    fs,
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use serde::Deserialize;
use tonic::Request;
use tonic::transport::{Channel, Endpoint};

// Local Imports
use crate::{Result, commands::pairing, error::Error};
use dokime::StorageTransferSettings;
use dokime_client_internal::CallOptions;
use dokime_client_internal::Client;
use dokime_runtime::{
    CaptureSettings, ControlledClock, CredentialNamespace, HostSettings, INTERVAL_LAG, INTERVAL_WINDOW, PlacementSettings, SchedulerClock,
    services::{ClientSettings, WallClock},
};

use dokime_runtime::{StorageSettings, storage_defaults};

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
    pub pairing: Option<pairing::File>,
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
    pub interval_window_ms: u64,
    pub interval_lag_windows: NonZeroU32,
    pub message_bytes: NonZeroUsize,
    pub retained_bytes: NonZeroUsize,
    pub stream_buffer: NonZeroUsize,
    pub max_cases: NonZeroUsize,
    pub page_items: NonZeroUsize,
    pub capture_roots: Vec<PathBuf>,
    pub capture_files: NonZeroUsize,
    pub capture_bytes: NonZeroU64,
    pub captures: NonZeroUsize,
    /// The Prefix of the Host's Native Credentials; Default `dokime`.
    pub credential_namespace: String,
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
            telemetry_rows: storage_defaults().flush_rows,
            telemetry_interval_ms: storage_defaults().flush_interval.as_millis() as u64,
            telemetry_pending_bytes: storage_defaults().pending_bytes,
            telemetry_query_bytes: storage_defaults().query_bytes,
            connect_timeout_s: 30,
            read_timeout_s: 30,
            mutation_timeout_s: 30,
            watch_timeout_s: 3600,
            reconnect_delay_ms: 25,
            interval_window_ms: INTERVAL_WINDOW.as_millis() as u64,
            interval_lag_windows: INTERVAL_LAG,
            message_bytes: NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
            retained_bytes: NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
            stream_buffer: NonZeroUsize::new(8).unwrap(),
            max_cases: NonZeroUsize::new(1000).unwrap(),
            page_items: NonZeroUsize::new(1000).unwrap(),
            capture_roots: Vec::new(),
            capture_files: NonZeroUsize::new(CaptureSettings::default().maximum_files).unwrap(),
            capture_bytes: NonZeroU64::new(CaptureSettings::default().maximum_bytes).unwrap(),
            captures: NonZeroUsize::new(CaptureSettings::default().maximum_captures).unwrap(),
            credential_namespace: CredentialNamespace::DEFAULT.into(),
        }
    }
}

impl Settings {
    pub fn load(path: Option<&Path>, connect_timeout: Option<NonZeroU64>) -> Result<Self> {
        let mut settings: Self = match path {
            Some(path) => toml::from_str(&fs::read_to_string(path)?)?,
            None => Self::default(),
        };
        if let Some(connect_timeout) = connect_timeout {
            settings.connect_timeout_s = connect_timeout.get();
        }
        if [
            settings.connect_timeout_s,
            settings.read_timeout_s,
            settings.mutation_timeout_s,
            settings.watch_timeout_s,
            settings.telemetry_interval_ms,
        ]
        .contains(&0)
        {
            return Err("Configure Positive Host and Client Timeouts".into());
        }
        if settings.interval_window_ms == 0 {
            return Err("Configure a Positive interval_window_ms".into());
        }
        settings.credentials()?;
        Ok(settings)
    }
    pub fn placement(&self) -> PlacementSettings {
        PlacementSettings {
            reconnect_delay: Duration::from_millis(self.reconnect_delay_ms),
            stream_window: self.stream_buffer,
            interval_window: Duration::from_millis(self.interval_window_ms),
            interval_lag: self.interval_lag_windows,
            message_bytes: self.message_bytes,
        }
    }
    pub fn credentials(&self) -> Result<CredentialNamespace> {
        Ok(CredentialNamespace::new(self.credential_namespace.clone())?)
    }
    pub fn host(&self, controlled: bool) -> Result<HostSettings> {
        let clock: Arc<dyn SchedulerClock> = if controlled { Arc::new(ControlledClock::new(0)) } else { Arc::new(WallClock) };
        Ok(HostSettings {
            installation: None,
            credentials: self.credentials()?,
            clock,
            placement: self.placement(),
            storage: StorageSettings {
                flush_rows: self.telemetry_rows,
                flush_interval: Duration::from_millis(self.telemetry_interval_ms),
                pending_bytes: self.telemetry_pending_bytes,
                query_bytes: self.telemetry_query_bytes,
                previews: storage_defaults().previews,
            },
            capture: CaptureSettings {
                roots: self.capture_roots.clone(),
                maximum_files: self.capture_files.get(),
                maximum_bytes: self.capture_bytes.get(),
                maximum_captures: self.captures.get(),
                ..CaptureSettings::default()
            },
        })
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
    pub fn client(&self) -> dokime_client_internal::ClientSettings {
        dokime_client_internal::ClientSettings {
            transfer: StorageTransferSettings::default(),
            retained_bytes: self.retained_bytes,
            message_bytes: self.message_bytes,
            input_items: self.stream_buffer,
            input_bytes: self.message_bytes,
        }
    }
    pub fn request<T>(&self, value: T) -> Request<T> {
        let mut request = Request::new(value);
        if let Some(pairing) = &self.pairing {
            request.metadata_mut().insert("authorization", pairing.authorization().expect("Validated Pairing Key"));
        }
        request
    }
    pub async fn sdk(&self, endpoint: Option<String>) -> Result<Client> {
        let channel = self.channel(endpoint).await?;
        Ok(Client::from_channel(channel, self.client())?)
    }
    pub fn context(&self) -> Result<CallOptions> {
        let mut context = CallOptions::default();
        if let Some(pairing) = &self.pairing {
            context.metadata.insert("authorization", pairing.authorization()?);
        }
        Ok(context)
    }

    pub fn request_id(&self, supplied: Option<String>) -> Result<String> {
        if let Some(value) = supplied {
            if value.is_empty() {
                return Err(Error::Invalid("Mutate Resource: Supply a Nonempty Request ID".into()));
            }
            return Ok(value);
        }
        let nonce = tempfile::Builder::new().prefix("dokime-request-").tempfile()?;
        Ok(nonce.path().file_name().unwrap().to_string_lossy().into_owned())
    }
    pub async fn channel(&self, endpoint: Option<String>) -> Result<Channel> {
        if let Some(pairing) = &self.pairing {
            return pairing.channel(self).await;
        }
        let endpoint = endpoint
            .ok_or_else(|| Error::Invalid("Supply --remote FILE, Set DOKIME_REMOTE, or Supply --endpoint http://HOST:PORT".into()))?;
        let channel = Endpoint::from_shared(endpoint)?.connect_timeout(Duration::from_secs(self.connect_timeout_s)).connect().await?;
        Ok(channel)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::fs;

    use clap::Parser;
    use tempfile::tempdir;

    use super::Settings;
    use crate::cli::Cli;

    #[test]
    fn connection_option_overrides_only_this_invocation() -> crate::Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("client.toml");
        let source = "connect_timeout_s = 11\n";
        fs::write(&path, source)?;

        let cli = Cli::try_parse_from(["dokime", "--config", path.to_str().unwrap(), "status", "--connect-timeout", "7"]).unwrap();
        let settings = Settings::load(cli.config.as_deref(), cli.connect_timeout)?;
        assert_eq!(settings.connect_timeout_s, 7);
        assert_eq!(fs::read_to_string(&path)?, source);

        let cli = Cli::try_parse_from(["dokime", "--config", path.to_str().unwrap(), "status"]).unwrap();
        let settings = Settings::load(cli.config.as_deref(), cli.connect_timeout)?;
        assert_eq!(settings.connect_timeout_s, 11);
        Ok(())
    }

    #[test]
    fn connection_option_requires_positive_whole_seconds() {
        for value in ["0", "-1", "1.5"] {
            assert!(Cli::try_parse_from(["dokime", "--connect-timeout", value, "status"]).is_err());
        }
    }
}
