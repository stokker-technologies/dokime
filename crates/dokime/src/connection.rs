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
    credentials::{KeyringStore, SecretStore},
    profiles::{Credential, Selection, Store},
};

/* === Constants === */

const ENVIRONMENT_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
#[group(id = "connection")]
pub(crate) struct Args {
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

    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=86400), help = "Connection And Call Deadline In Seconds.")]
    timeout_seconds: u64,
}

pub(crate) struct Resolved {
    pub(crate) selection: Selection,
    pub(crate) store: Option<Store>,
    pub(crate) token: Option<Zeroizing<String>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Args {
    pub(crate) fn resolve(&self) -> crate::Result<Resolved> {
        let environment = read_environment(self.env_file.as_deref())?;
        let token = read_key(self.key_file.as_deref(), &environment)?;
        if let Some(endpoint) = &self.endpoint {
            let ca = self
                .ca
                .as_deref()
                .map(|path| {
                    let bytes = harmos::filesystem::input::read(path, 1024 * 1024)?;
                    String::from_utf8(bytes).map_err(|_| crate::Error::Input("CA certificate must be UTF-8 PEM".into()))
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
                token,
            });
        }
        let environment =
            if self.backend.is_some() { None } else { environment_value("DOKIME_BACKEND", &environment)?.map(|value| value.to_string()) };
        let store = Store::discover(self.profiles_file.as_deref())?;
        let selection = store.load()?.select(self.backend.as_deref(), None, environment.as_deref())?;
        Ok(Resolved { selection, store: Some(store), token })
    }

    pub(crate) fn has_key_file(&self) -> bool {
        self.key_file.is_some()
    }

    pub(crate) fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    pub(crate) async fn connect_selected(&self, resolved: &Resolved, mut token: Option<Zeroizing<String>>) -> crate::Result<Client> {
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
        self.connect_selected(&resolved, resolved.token.clone()).await
    }
}

/* === Public Functions === */

// OS Stores Can Block. Detached Workers Keep The Invocation Deadline Bounded.
// Dropping The Receiver Does Not Cancel An OS Operation Already In Progress.
pub(crate) async fn credentials<T, E>(budget: Duration, operation: impl FnOnce() -> Result<T, E> + Send + 'static) -> crate::Result<T>
where
    T: Send + 'static,
    E: Into<crate::Error> + Send + 'static,
{
    let (sender, receiver) = tokio::sync::oneshot::channel();
    std::thread::Builder::new().name("dokime-credentials".into()).spawn(move || {
        let _ = sender.send(operation().map_err(Into::into));
    })?;
    tokio::time::timeout(budget, receiver)
        .await
        .map_err(|_| crate::Error::CredentialTimeout)?
        .map_err(|_| crate::Error::Setup("Credential store worker failed".into()))?
}

/* === Private Functions === */

fn read_environment(path: Option<&Path>) -> crate::Result<BTreeMap<String, Zeroizing<String>>> {
    let Some(path) = path else { return Ok(BTreeMap::new()) };
    let bytes = Zeroizing::new(harmos::filesystem::input::read(path, ENVIRONMENT_BYTES)?);
    if bytes.contains(&0) {
        return Err(crate::Error::Input("The selected dotenv file contains an invalid entry.".into()));
    }
    let mut values = BTreeMap::new();
    for entry in dotenv::from_read_iter(std::io::Cursor::new(bytes.as_slice())) {
        // Parser Errors Can Contain The Original Secret-Bearing Line.
        let (name, value) = entry.map_err(|_| crate::Error::Input("The selected dotenv file contains an invalid entry.".into()))?;
        values.insert(name, Zeroizing::new(value));
    }
    Ok(values)
}

fn environment_value(name: &str, values: &BTreeMap<String, Zeroizing<String>>) -> crate::Result<Option<Zeroizing<String>>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(Zeroizing::new(value))),
        Err(std::env::VarError::NotPresent) => Ok(values.get(name).cloned()),
        Err(_) => Err(crate::Error::Input(format!("{name} must be UTF-8"))),
    }
}

fn read_key(path: Option<&Path>, environment: &BTreeMap<String, Zeroizing<String>>) -> crate::Result<Option<Zeroizing<String>>> {
    let key = match path {
        Some(path) => {
            let bytes = Zeroizing::new(harmos::filesystem::input::read(path, 4098)?);
            let text = std::str::from_utf8(&bytes).map_err(|_| crate::Error::Input("Remote key file must be UTF-8 text".into()))?;
            Some(Zeroizing::new(text.trim().to_owned()))
        }
        None => environment_value("DOKIME_KEY", environment)?,
    };
    if let Some(key) = &key
        && (!(32..=4096).contains(&key.len()) || !key.bytes().all(|byte| byte.is_ascii_graphic()))
    {
        return Err(crate::Error::Input("Remote key must contain 32–4096 visible ASCII characters.".into()));
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
        let cases: Vec<EnvironmentCase> =
            serde_json::from_str(include_str!("../../dokime-client/tests/fixtures/credentials.json")).unwrap();
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("selected.env");
        for case in cases {
            std::fs::write(&path, case.document).unwrap();
            let result = read_environment(Some(&path));
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
            endpoint: Some("http://127.0.0.1:1".into()),
            backend: None,
            profiles_file: None,
            key_file: Some(key_file.clone()),
            env_file: Some(env_file),
            ca: None,
            expected_backend: None,
            timeout_seconds: 1,
        };
        let error = args.resolve().err().expect("bad explicit dotenv cannot be hidden by a valid key file");
        assert!(!format!("{error:?}").contains("private"));
        for bytes in [vec![], vec![0xff], b"short".to_vec(), vec![b'a'; 4099]] {
            std::fs::write(&key_file, bytes).unwrap();
            assert!(read_key(Some(&key_file), &fallback).is_err());
        }
        assert!(read_key(Some(&root.path().join("missing.key")), &fallback).is_err());
        assert!(read_environment(Some(root.path())).is_err());
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
