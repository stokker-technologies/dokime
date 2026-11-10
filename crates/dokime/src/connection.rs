/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{path::PathBuf, time::Duration};

// Third Party Imports

// Local Imports
use dokime_client::{Client, profiles::Selection};

/* === Constants === */

const MAXIMUM_TIMEOUT_MS: u64 = 86_400_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
#[group(id = "connection")]
pub struct Args {
    #[arg(skip)]
    settings: dokime_domain::model::settings::application::ClientPreferences,

    #[arg(long, help = "Pairing File; Defaults To DOKIME_REMOTE.")]
    remote: Option<PathBuf>,

    #[arg(long, value_parser = clap::value_parser!(u64).range(1..), help = "Connection And Call Deadline In Seconds; Defaults To Local Configuration.")]
    timeout_seconds: Option<u64>,
}

pub struct Resolved {
    pub selection: Selection,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Args {
    pub fn with_settings(mut self, settings: dokime_domain::model::settings::application::ClientPreferences) -> Self {
        self.settings = settings;
        self
    }

    pub fn settings(&self) -> &dokime_domain::model::settings::application::ClientPreferences {
        &self.settings
    }

    pub fn resolve(&self) -> crate::client_error::Result<Resolved> {
        self.settings.validate().map_err(crate::client_error::Error::Input)?;
        let timeout = self.timeout();
        if timeout > Duration::from_millis(MAXIMUM_TIMEOUT_MS) || std::time::Instant::now().checked_add(timeout).is_none() {
            return Err(crate::client_error::Error::Input(
                "connection deadline exceeds the configured maximum or native clock range".into(),
            ));
        }
        let mut selection = dokime_client::pairing::resolve(self.remote.as_deref())?;
        selection.target = selection.target.with_settings(self.settings);
        Ok(Resolved { selection })
    }

    pub fn timeout(&self) -> Duration {
        self.timeout_seconds.map(Duration::from_secs).unwrap_or_else(|| Duration::from_millis(self.settings.request_timeout_ms))
    }

    pub async fn connect_selected(&self, resolved: &Resolved) -> crate::client_error::Result<Client> {
        let deadline = tokio::time::Instant::now() + self.timeout();
        let connect = Client::connect(
            resolved.selection.target.clone(),
            resolved.selection.access_key.as_deref().map(String::as_str),
            resolved.selection.expected_backend_id.as_deref(),
        );
        let client = tokio::time::timeout_at(deadline, connect)
            .await
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::TimedOut, error))??;
        Ok(client.with_timeout(self.timeout()))
    }

    pub async fn connect(&self) -> crate::client_error::Result<Client> {
        let resolved = self.resolve()?;
        self.connect_selected(&resolved).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
