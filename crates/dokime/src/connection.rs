/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{path::PathBuf, time::Duration};

use dokime_client::{
    Client, Target,
    profiles::{Secret, Selection, Store},
    secrets::{KeyringStore, SecretStore},
};

const MAXIMUM_TIMEOUT_MS: u64 = 86_400_000;

#[derive(Debug, clap::Args)]
#[group(id = "connection")]
pub struct Args {
    #[arg(skip)]
    settings: dokime_domain::model::settings::application::ClientPreferences,

    #[arg(long, help = "Backend HTTP(S) Origin; Use Plain HTTP Only In An Isolated Lab.")]
    #[arg(conflicts_with = "backend")]
    endpoint: Option<String>,

    #[arg(long, help = "Use This Saved Backend Profile.")]
    backend: Option<String>,

    #[arg(long, help = "Use This Backend Profile Configuration File.")]
    profiles_file: Option<PathBuf>,

    #[arg(long, value_parser = clap::value_parser!(u64).range(1..), help = "Connection And Call Deadline In Seconds; Defaults To Local Configuration.")]
    timeout_seconds: Option<u64>,
}

pub struct Resolved {
    pub selection: Selection,
}

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
        if let Some(endpoint) = &self.endpoint {
            if !endpoint.starts_with("http://") {
                return Err(crate::client_error::Error::Input("secure connections require an imported pairing profile".into()));
            }
            return Ok(Resolved {
                selection: Selection {
                    identifier: None,
                    target: Target::new(endpoint, None)?.with_settings(self.settings),
                    expected_backend_id: None,
                    secret: Secret::None,
                },
            });
        }
        let environment = if self.backend.is_some() {
            None
        } else {
            match std::env::var("DOKIME_BACKEND") {
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(_) => return Err(crate::client_error::Error::Input("DOKIME_BACKEND must be UTF-8".into())),
            }
        };
        let store = Store::discover(self.profiles_file.as_deref())?;
        let document = store.load()?;
        let mut selection = dokime_client::profiles::select(&document, self.backend.as_deref(), None, environment.as_deref())?;
        selection.target = selection.target.with_settings(self.settings);
        Ok(Resolved { selection })
    }

    pub fn timeout(&self) -> Duration {
        self.timeout_seconds.map(Duration::from_secs).unwrap_or_else(|| Duration::from_millis(self.settings.request_timeout_ms))
    }

    pub async fn connect_selected(&self, resolved: &Resolved) -> crate::client_error::Result<Client> {
        let deadline = tokio::time::Instant::now() + self.timeout();
        let mut token = None;
        if resolved.selection.secret == Secret::Keyring {
            let endpoint = resolved.selection.credential_key();
            token = secrets(self.timeout(), move || KeyringStore.read(&endpoint)).await?;
            if token.is_none() {
                return Err(dokime_client::Error::AuthenticationRequired.into());
            }
        }
        let connect = Client::connect(
            resolved.selection.target.clone(),
            token.as_deref().map(String::as_str),
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

// OS Stores Can Block. Detached Workers Keep The Invocation Deadline Bounded.
// Dropping The Receiver Does Not Cancel An OS Operation Already In Progress.
pub async fn secrets<T, E>(budget: Duration, operation: impl FnOnce() -> Result<T, E> + Send + 'static) -> crate::client_error::Result<T>
where
    T: Send + 'static,
    E: Into<crate::client_error::Error> + Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    std::thread::Builder::new().name("dokime-secrets".into()).spawn(move || {
        let _ = sender.send(operation().map_err(Into::into));
    })?;
    tokio::time::timeout(budget, receiver)
        .await
        .map_err(|_| crate::client_error::Error::SecretTimeout)?
        .map_err(|_| crate::client_error::Error::Setup("Secret store worker failed".into()))?
}
