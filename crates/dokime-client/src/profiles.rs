/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Named Backend Profiles And Atomic Configuration Updates

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeSet,
    env,
    fs::{OpenOptions, TryLockError},
    io,
    path::{Path, PathBuf},
};

// Third Party Imports
pub use dokime_domain::model::backends::{Document, Profile, Secret};

// Local Imports
use super::{Target, secrets::SecretStore};

/* === Constants === */

const CA_BYTES: u64 = 1_048_576;
const PROFILE_BYTES: u64 = 1_048_576;
const PROFILE_ID_BYTES: u64 = 256;
const PROFILES: u64 = 128;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A configuration file. Reading and selection never create files or connect.
pub struct Store {
    path: PathBuf,
}

pub struct Selection {
    pub identifier: Option<String>,
    pub target: Target,
    pub expected_backend_id: Option<String>,
    pub secret: Secret,
}

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid backend profiles: {0}")]
    Invalid(String),

    #[error("Backend profile configuration is busy; retry the edit.")]
    Busy,

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Filesystem(#[from] harmos::filesystem::Error),

    #[error(transparent)]
    Secrets(#[from] super::secrets::Error),
}

/* === Traits === */

/* === Implementations === */

pub fn decode(bytes: &[u8]) -> Result<Document, Error> {
    if bytes.len() as u64 > PROFILE_BYTES {
        return Err(invalid("file exceeds the configured profile byte limit"));
    }
    let document: Document = serde_json::from_slice(bytes).map_err(|_| invalid("file does not match the version 1 profile schema"))?;
    validate(&document)?;
    Ok(document)
}

pub fn encode(document: &Document) -> Result<Vec<u8>, Error> {
    validate(document)?;
    let bytes = serde_json::to_vec_pretty(document).map_err(|_| invalid("cannot encode profile document"))?;
    if bytes.len() as u64 > PROFILE_BYTES {
        return Err(invalid("file exceeds the configured profile byte limit"));
    }
    Ok(bytes)
}

pub fn validate(document: &Document) -> Result<(), Error> {
    if document.version != 1 || document.profiles.len() as u64 > PROFILES {
        return Err(invalid("unsupported version or too many profiles"));
    }
    let mut identifiers = BTreeSet::new();
    for profile in &document.profiles {
        if !identifiers.insert(&profile.identifier) {
            return Err(invalid("duplicate profile identifier"));
        }
        selection(profile)?;
    }
    if let Some(identifier) = &document.default {
        get(document, identifier)?;
    }
    Ok(())
}

pub fn get<'a>(document: &'a Document, identifier: &str) -> Result<&'a Profile, Error> {
    validate_identifier(identifier)?;
    document.profiles.iter().find(|profile| profile.identifier == identifier).ok_or_else(|| invalid("requested profile does not exist"))
}

// Selection happens once; subsequent default edits cannot alter a connection.
pub fn select(document: &Document, backend: Option<&str>, endpoint: Option<&str>, environment: Option<&str>) -> Result<Selection, Error> {
    if backend.is_some() && endpoint.is_some() {
        return Err(invalid("backend and endpoint are mutually exclusive"));
    }
    if let Some(endpoint) = endpoint {
        let target = Target::new(endpoint, None).map_err(|_| invalid("invalid endpoint"))?;
        return Ok(Selection {
            identifier: None,
            target,
            expected_backend_id: None,
            secret: Secret::None,
        });
    }
    let identifier =
        backend.or(environment).or(document.default.as_deref()).ok_or_else(|| invalid("select a backend profile or explicit endpoint"))?;
    selection(get(document, identifier)?)
}

pub fn selection(profile: &Profile) -> Result<Selection, Error> {
    profile.validate().map_err(invalid)?;
    if let Some(identity) = &profile.expected_backend_id
        && (identity.is_empty() || identity.len() as u64 > PROFILE_ID_BYTES || identity.chars().any(char::is_control))
    {
        return Err(invalid("invalid expected backend identity"));
    }
    if let Some(ca) = &profile.ca_certificate_pem
        && (ca.is_empty() || ca.len() as u64 > CA_BYTES || ca.contains('\0'))
    {
        return Err(invalid("invalid CA certificate"));
    }
    let target = Target::new(&profile.endpoint, profile.ca_certificate_pem.clone())
        .map_err(|_| invalid("invalid endpoint or CA certificate association"))?;
    Ok(Selection {
        identifier: Some(profile.identifier.clone()),
        target,
        expected_backend_id: profile.expected_backend_id.clone(),
        secret: profile.secret,
    })
}

impl Selection {
    pub fn credential_key(&self) -> String {
        match &self.identifier {
            Some(identifier) => format!("dokime-profile:{identifier}:{}", self.target.endpoint()),
            None => self.target.endpoint().to_owned(),
        }
    }
}

impl Store {
    pub fn import_pairing(
        &self, pairing: &dokime_domain::model::remote::PairingFile, identifier: &str, secrets: &dyn SecretStore,
    ) -> Result<Profile, Error> {
        pairing.validate().map_err(Error::Invalid)?;
        let profile = Profile {
            identifier: identifier.to_owned(),
            label: pairing.label.clone(),
            endpoint: pairing.endpoint.clone(),
            expected_backend_id: Some(pairing.backend_identifier.clone()),
            ca_certificate_pem: pairing.ca_certificate_pem.clone(),
            secret: if pairing.access_key.is_some() { Secret::Keyring } else { Secret::None },
        };
        let selected = selection(&profile)?;
        let credential = selected.credential_key();
        let mut saved_secret = false;
        let result = self.update(|document| {
            if document.profiles.iter().any(|existing| existing.identifier == identifier) {
                return Err(invalid("profile already exists"));
            }
            document.profiles.push(profile.clone());
            encode(document)?;
            if let Some(key) = &pairing.access_key {
                if secrets.read(&credential)?.is_some() {
                    return Err(invalid("profile credential already exists"));
                }
                secrets.write(&credential, key)?;
                saved_secret = true;
            }
            Ok(())
        });
        if let Err(error) = result {
            if saved_secret {
                secrets.remove(&credential)?;
            }
            return Err(error);
        }
        Ok(profile)
    }

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn discover(override_path: Option<&Path>) -> Result<Self, Error> {
        if let Some(path) = override_path {
            return Ok(Self::new(path));
        }
        if let Some(path) = env::var_os("DOKIME_PROFILES_FILE") {
            if path.is_empty() {
                return Err(invalid("DOKIME_PROFILES_FILE is empty"));
            }
            return Ok(Self::new(path));
        }
        #[cfg(target_os = "windows")]
        let base = environment_path("APPDATA")?;
        #[cfg(target_os = "macos")]
        let base = environment_path("HOME")?.join("Library/Application Support");
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let base = match env::var_os("XDG_CONFIG_HOME") {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            _ => environment_path("HOME")?.join(".config"),
        };
        if !base.is_absolute() {
            return Err(invalid("configuration root must be absolute"));
        }
        Ok(Self::new(base.join("dokime/backends.json")))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn parent(&self) -> Result<&Path, Error> {
        if self.path.file_name().is_none() {
            return Err(invalid("configuration path must name a file"));
        }
        Ok(self.path.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new(".")))
    }

    pub fn load(&self) -> Result<Document, Error> {
        self.parent()?;

        match harmos::filesystem::input::read(&self.path, PROFILE_BYTES as usize) {
            Ok(bytes) => decode(&bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Document::default()),
            Err(error) => Err(error.into()),
        }
    }

    // Called only after bearer authentication and backend identity negotiation.
    // Protect the original profile association across the preceding network IO.
    // A file publication failure can leave the secret saved separately.
    pub fn remember(&self, selection: &Selection, token: &str, secrets: &dyn SecretStore) -> Result<(), Error> {
        let identifier = selection.identifier.as_deref().ok_or_else(|| invalid("a named profile is required"))?;
        self.update(|document| {
            let current = self::selection(get(document, identifier)?)?;
            if current.target.with_settings(*selection.target.settings()) != selection.target
                || current.expected_backend_id != selection.expected_backend_id
            {
                return Err(invalid("profile target changed during login; retry explicitly"));
            }
            let profile = document.profiles.iter_mut().find(|profile| profile.identifier == identifier).expect("validated profile exists");
            profile.secret = Secret::Keyring;
            // Refuse Schema And Capacity Failures Before Saving A Secret.
            encode(document)?;
            secrets.write(&selection.credential_key(), token)?;
            Ok(())
        })
    }

    // Edit the latest snapshot while holding a stable sibling lock. Fail fast
    // on contention; never unlink the lock file or hold a lock during network IO.
    pub fn update<T>(&self, edit: impl FnOnce(&mut Document) -> Result<T, Error>) -> Result<T, Error> {
        let parent = self.parent()?;
        harmos::filesystem::publication::create_dir_all(parent)?;
        let mut lock_path = self.path.as_os_str().to_os_string();
        lock_path.push(".lock");
        let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(PathBuf::from(lock_path))?;
        match lock.try_lock() {
            Ok(()) => (),
            Err(TryLockError::WouldBlock) => return Err(Error::Busy),
            Err(TryLockError::Error(error)) => return Err(error.into()),
        }
        let mut document = self.load()?;
        let result = edit(&mut document)?;
        let bytes = encode(&document)?;
        harmos::filesystem::publication::write(&self.path, &bytes)?;
        Ok(result)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid(message: &str) -> Error {
    Error::Invalid(message.to_owned())
}

fn validate_identifier(identifier: &str) -> Result<(), Error> {
    dokime_domain::model::secrets::catalog::validate_identifier(identifier).map_err(invalid)
}

fn environment_path(identifier: &str) -> Result<PathBuf, Error> {
    env::var_os(identifier)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| invalid("standard configuration root is unavailable; provide an explicit profile file"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;
    use zeroize::Zeroizing;

    struct Credentials {
        values: Mutex<BTreeMap<String, String>>,
        fail_publication: Option<PathBuf>,
    }

    impl SecretStore for Credentials {
        fn read(&self, key: &str) -> Result<Option<Zeroizing<String>>, super::super::secrets::Error> {
            Ok(self.values.lock().unwrap().get(key).cloned().map(Zeroizing::new))
        }

        fn write(&self, key: &str, token: &str) -> Result<(), super::super::secrets::Error> {
            self.values.lock().unwrap().insert(key.to_owned(), token.to_owned());
            if let Some(path) = &self.fail_publication {
                std::fs::create_dir(path).unwrap();
            }
            Ok(())
        }

        fn remove(&self, key: &str) -> Result<(), super::super::secrets::Error> {
            self.values.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn pairing(key: char) -> dokime_domain::model::remote::PairingFile {
        dokime_domain::model::remote::PairingFile {
            version: 1,
            endpoint: "http://127.0.0.1:8765".into(),
            backend_identifier: "backend".into(),
            label: "Lab Backend".into(),
            ca_certificate_pem: None,
            access_key: Some(key.to_string().repeat(32)),
        }
    }

    #[test]
    fn pairing_profiles_keep_independent_credentials() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::new(root.path().join("backends.json"));
        let secrets = Credentials {
            values: Mutex::new(BTreeMap::new()),
            fail_publication: None,
        };
        let first = store.import_pairing(&pairing('a'), "first", &secrets).unwrap();
        let second = store.import_pairing(&pairing('b'), "second", &secrets).unwrap();
        let first_key = selection(&first).unwrap().credential_key();
        let second_key = selection(&second).unwrap().credential_key();
        assert_ne!(first_key, second_key);
        assert_eq!(secrets.read(&first_key).unwrap().unwrap().as_str(), "a".repeat(32));
        assert_eq!(secrets.read(&second_key).unwrap().unwrap().as_str(), "b".repeat(32));
        assert!(store.load().unwrap().default.is_none());
        assert!(store.import_pairing(&pairing('c'), "first", &secrets).is_err());
        assert_eq!(secrets.read(&first_key).unwrap().unwrap().as_str(), "a".repeat(32));
    }

    #[test]
    fn pairing_save_failure_removes_new_credential() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("backends.json");
        let store = Store::new(&path);
        let secrets = Credentials {
            values: Mutex::new(BTreeMap::new()),
            fail_publication: Some(path),
        };
        assert!(store.import_pairing(&pairing('a'), "first", &secrets).is_err());
        assert!(secrets.values.lock().unwrap().is_empty());
    }

    #[test]
    fn invalid_pairing_never_opens_registry_or_keyring() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("absent/backends.json");
        let store = Store::new(&path);
        let secrets = Credentials {
            values: Mutex::new(BTreeMap::new()),
            fail_publication: None,
        };
        let mut invalid = pairing('a');
        invalid.endpoint = "http://0.0.0.0:8765".into();
        assert!(store.import_pairing(&invalid, "first", &secrets).is_err());
        assert!(!path.parent().unwrap().exists());
        assert!(secrets.values.lock().unwrap().is_empty());
    }
}
