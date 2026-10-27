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
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

// Third Party Imports
use zeroize::Zeroizing;

// Local Imports
use dokime_client::{
    Client, Target,
    profiles::{Secret, Selection, Store},
    secrets::{KeyringStore, SecretStore},
};

/* === Constants === */

const CA_BYTES: u64 = 1_048_576;
const ENVIRONMENT_BYTES: u64 = 65_536;
const MAXIMUM_TIMEOUT_MS: u64 = 86_400_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

    #[arg(long, help = "Read The Remote Key From This UTF-8 File; Overrides DOKIME_KEY.")]
    key_file: Option<PathBuf>,

    #[arg(long, help = "Read This Explicit Dotenv File Without Overriding Process Variables Or Searching Parent Directories.")]
    env_file: Option<PathBuf>,

    #[arg(long, requires = "endpoint", conflicts_with = "backend", help = "Read An Additional PEM CA Certificate From This Local File.")]
    ca: Option<PathBuf>,

    #[arg(long, requires = "endpoint", conflicts_with = "backend", help = "Require This Backend Identity During Connection.")]
    expected_backend: Option<String>,

    #[arg(long, value_parser = clap::value_parser!(u64).range(1..), help = "Connection And Call Deadline In Seconds; Defaults To Local Configuration.")]
    timeout_seconds: Option<u64>,
}

pub struct Resolved {
    pub selection: Selection,
    pub store: Option<Store>,
    pub token: Option<Zeroizing<String>>,
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
        let environment = read_environment(self.env_file.as_deref(), ENVIRONMENT_BYTES as usize)?;
        let token = read_key(self.key_file.as_deref(), &environment)?;
        if let Some(endpoint) = &self.endpoint {
            let ca = self
                .ca
                .as_deref()
                .map(|path| {
                    let bytes = harmos::filesystem::input::read(path, CA_BYTES as usize)?;
                    String::from_utf8(bytes).map_err(|_| crate::client_error::Error::Input("CA certificate must be UTF-8 PEM".into()))
                })
                .transpose()?;
            return Ok(Resolved {
                selection: Selection {
                    name: None,
                    target: Target::new(endpoint, ca)?.with_settings(self.settings),
                    expected_backend_id: self.expected_backend.clone(),
                    secret: Secret::None,
                },
                store: None,
                token,
            });
        }
        let environment =
            if self.backend.is_some() { None } else { environment_value("DOKIME_BACKEND", &environment)?.map(|value| value.to_string()) };
        let store = Store::discover(self.profiles_file.as_deref())?;
        let document = store.load()?;
        let mut selection = dokime_client::profiles::select(&document, self.backend.as_deref(), None, environment.as_deref())?;
        selection.target = selection.target.with_settings(self.settings);
        Ok(Resolved { selection, store: Some(store), token })
    }

    pub fn has_key_file(&self) -> bool {
        self.key_file.is_some()
    }

    pub fn timeout(&self) -> Duration {
        self.timeout_seconds.map(Duration::from_secs).unwrap_or_else(|| Duration::from_millis(self.settings.request_timeout_ms))
    }

    pub async fn connect_selected(&self, resolved: &Resolved, mut token: Option<Zeroizing<String>>) -> crate::client_error::Result<Client> {
        let deadline = tokio::time::Instant::now() + self.timeout();
        if token.is_none() && resolved.selection.secret == Secret::Keyring {
            let endpoint = resolved.selection.target.endpoint().to_owned();
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
        self.connect_selected(&resolved, resolved.token.clone()).await
    }
}

/* === Public Functions === */

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

/* === Private Functions === */

fn read_environment(path: Option<&Path>, maximum: usize) -> crate::client_error::Result<BTreeMap<String, Zeroizing<String>>> {
    let Some(path) = path else { return Ok(BTreeMap::new()) };
    let bytes = Zeroizing::new(harmos::filesystem::input::read(path, maximum)?);
    if bytes.contains(&0) {
        return Err(crate::client_error::Error::Input("The selected dotenv file contains an invalid entry.".into()));
    }
    let mut values = BTreeMap::new();
    for entry in dotenv::from_read_iter(std::io::Cursor::new(bytes.as_slice())) {
        // Parser Errors Can Contain The Original Secret-Bearing Line.
        let (name, value) =
            entry.map_err(|_| crate::client_error::Error::Input("The selected dotenv file contains an invalid entry.".into()))?;
        values.insert(name, Zeroizing::new(value));
    }
    Ok(values)
}

fn environment_value(name: &str, values: &BTreeMap<String, Zeroizing<String>>) -> crate::client_error::Result<Option<Zeroizing<String>>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(Zeroizing::new(value))),
        Err(std::env::VarError::NotPresent) => Ok(values.get(name).cloned()),
        Err(_) => Err(crate::client_error::Error::Input(format!("{name} must be UTF-8"))),
    }
}

fn read_key(
    path: Option<&Path>, environment: &BTreeMap<String, Zeroizing<String>>,
) -> crate::client_error::Result<Option<Zeroizing<String>>> {
    let key = match path {
        Some(path) => {
            let bytes = Zeroizing::new(harmos::filesystem::input::read(path, 4098)?);
            let text =
                std::str::from_utf8(&bytes).map_err(|_| crate::client_error::Error::Input("Remote key file must be UTF-8 text".into()))?;
            Some(Zeroizing::new(text.trim().to_owned()))
        }
        None => environment_value("DOKIME_KEY", environment)?,
    };
    if let Some(key) = &key
        && (!(32..=4096).contains(&key.len()) || !key.bytes().all(|byte| byte.is_ascii_graphic()))
    {
        return Err(crate::client_error::Error::Input("Remote key must contain 32–4096 visible ASCII characters.".into()));
    }
    Ok(key)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct EnvironmentCase {
        name: String,
        document: String,
        valid: bool,
        key: Option<String>,
    }

    #[test]
    fn dotenv_contract() {
        let cases: Vec<EnvironmentCase> = serde_json::from_str(include_str!("../../dokime-client/tests/fixtures/secrets.json")).unwrap();
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("selected.env");
        for case in cases {
            std::fs::write(&path, case.document).unwrap();
            let result = read_environment(Some(&path), 64 * 1024);
            assert_eq!(result.is_ok(), case.valid, "{}", case.name);
            match result {
                Ok(values) => assert_eq!(values.get("DOKIME_KEY").map(|key| key.to_string()), case.key),
                Err(error) => assert!(!format!("{error:?}").contains("private")),
            }
        }
    }

    #[test]
    fn explicit_files_preflight() {
        let root = tempfile::tempdir().unwrap();
        let key_file = root.path().join("remote.key");
        let env_file = root.path().join("selected.env");
        let key = "a".repeat(32);
        std::fs::write(&key_file, format!("{key}\r\n")).unwrap();
        let fallback = BTreeMap::from([("DOKIME_KEY".into(), Zeroizing::new("b".repeat(32)))]);
        assert_eq!(read_key(Some(&key_file), &fallback).unwrap().as_deref().map(String::as_str), Some(key.as_str()));
        std::fs::write(&env_file, "invalid private entry").unwrap();
        let args = Args {
            settings: Default::default(),
            endpoint: Some("http://127.0.0.1:1".into()),
            backend: None,
            profiles_file: None,
            key_file: Some(key_file.clone()),
            env_file: Some(env_file),
            ca: None,
            expected_backend: None,
            timeout_seconds: Some(1),
        };
        let error = args.resolve().err().expect("bad explicit dotenv cannot be hidden by a valid key file");
        assert!(!format!("{error:?}").contains("private"));
        for bytes in [vec![], vec![0xff], b"short".to_vec(), vec![b'a'; 4099]] {
            std::fs::write(&key_file, bytes).unwrap();
            assert!(read_key(Some(&key_file), &fallback).is_err());
        }
        assert!(read_key(Some(&root.path().join("missing.key")), &fallback).is_err());
        assert!(read_environment(Some(root.path()), 64 * 1024).is_err());
    }

    #[test]
    fn process_values_precede_dotenv_without_mutation() {
        let process_path = std::env::var("PATH").expect("the test process has a PATH");
        let values = BTreeMap::from([("PATH".into(), Zeroizing::new("dotenv-fallback".into()))]);
        let selected = environment_value("PATH", &values).unwrap().unwrap();
        assert!(selected.as_str() == process_path);
        assert!(std::env::var("PATH").unwrap() == process_path);
    }
}
