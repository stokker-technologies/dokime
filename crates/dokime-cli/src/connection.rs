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
use zeroize::Zeroizing;

// Local Imports
use dokime_client::{
    Client, Target,
    credentials::{KeyringStore, SecretStore},
    profiles::{Credential, Selection, Store},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
#[group(id = "connection")]
pub(crate) struct Args {
    #[arg(
        long,
        help = "Backend HTTP(S) Origin; Plain HTTP Is Restricted To Loopback."
    )]
    #[arg(conflicts_with = "backend")]
    endpoint: Option<String>,
    #[arg(long, help = "Use This Saved Backend Profile.")]
    backend: Option<String>,
    #[arg(long, help = "Use This Backend Profile Configuration File.")]
    profiles_file: Option<PathBuf>,
    #[arg(long, help = "Read A Bearer Token From This Local File.")]
    token_file: Option<PathBuf>,
    #[arg(
        long,
        requires = "endpoint",
        conflicts_with = "backend",
        help = "Read An Additional PEM CA Certificate From This Local File."
    )]
    ca: Option<PathBuf>,
    #[arg(
        long,
        requires = "endpoint",
        conflicts_with = "backend",
        help = "Require This Backend Identity During Connection."
    )]
    expected_backend: Option<String>,
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=86400), help = "Connection And Call Deadline In Seconds.")]
    timeout_seconds: u64,
}

pub(crate) struct Resolved {
    pub(crate) selection: Selection,
    pub(crate) store: Option<Store>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Args {
    pub(crate) fn resolve(&self) -> crate::Result<Resolved> {
        if let Some(endpoint) = &self.endpoint {
            let ca = self
                .ca
                .as_deref()
                .map(|path| {
                    let bytes = dokime_filesystem::input::read(path, 1024 * 1024)?;
                    String::from_utf8(bytes)
                        .map_err(|_| crate::Error::Input("CA certificate must be UTF-8 PEM".into()))
                })
                .transpose()?;
            return Ok(Resolved {
                selection: Selection {
                    name: None,
                    target: Target::new(endpoint, ca)?,
                    expected_backend_id: self.expected_backend.clone(),
                    credential: Credential::None,
                },
                store: None,
            });
        }
        let environment = if self.backend.is_some() {
            None
        } else {
            match std::env::var("DOKIME_BACKEND") {
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(_) => return Err(crate::Error::Input("DOKIME_BACKEND must be UTF-8".into())),
            }
        };
        let store = Store::discover(self.profiles_file.as_deref())?;
        let selection =
            store
                .load()?
                .select(self.backend.as_deref(), None, environment.as_deref())?;
        Ok(Resolved {
            selection,
            store: Some(store),
        })
    }

    pub(crate) fn provided_token(&self) -> crate::Result<Option<Zeroizing<String>>> {
        self.token_file
            .as_deref()
            .map(|path| {
                let bytes = Zeroizing::new(dokime_filesystem::input::read(path, 4098)?);
                let text = std::str::from_utf8(&bytes)
                    .map_err(|_| crate::Error::Input("bearer token must be UTF-8 text".into()))?;
                Ok(Zeroizing::new(text.trim().to_owned()))
            })
            .transpose()
    }

    pub(crate) fn has_token_file(&self) -> bool {
        self.token_file.is_some()
    }

    pub(crate) fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    pub(crate) async fn connect_selected(
        &self,
        resolved: &Resolved,
        mut token: Option<Zeroizing<String>>,
    ) -> crate::Result<Client> {
        let deadline = tokio::time::Instant::now() + self.timeout();
        if token.is_none() && resolved.selection.credential == Credential::Keyring {
            let endpoint = resolved.selection.target.endpoint().to_owned();
            token = credentials(self.timeout(), move || KeyringStore.read(&endpoint)).await?;
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

    pub(crate) async fn connect(&self) -> crate::Result<Client> {
        let resolved = self.resolve()?;
        self.connect_selected(&resolved, self.provided_token()?)
            .await
    }
}

/* === Public Functions === */
// OS Stores Can Block. Detached Workers Keep The Invocation Deadline Bounded.
// Dropping The Receiver Does Not Cancel An OS Operation Already In Progress.
pub(crate) async fn credentials<T, E>(
    budget: Duration,
    operation: impl FnOnce() -> Result<T, E> + Send + 'static,
) -> crate::Result<T>
where
    T: Send + 'static,
    E: Into<crate::Error> + Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    std::thread::Builder::new()
        .name("dokime-credentials".into())
        .spawn(move || {
            let _ = sender.send(operation().map_err(Into::into));
        })?;
    tokio::time::timeout(budget, receiver)
        .await
        .map_err(|_| crate::Error::CredentialTimeout)?
        .map_err(|_| crate::Error::Setup("Credential store worker failed".into()))?
}

/* === Private Functions === */

/* === Tests === */
