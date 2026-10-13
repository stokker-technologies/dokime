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
use serde::{Deserialize, Serialize};

// Local Imports
use super::{Target, credentials::SecretStore};

/* === Constants === */
pub const MAX_BYTES: usize = 1024 * 1024;
const MAX_PROFILES: usize = 128;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub endpoint: String,
    pub expected_backend_id: Option<String>,
    pub ca_certificate_pem: Option<String>,
    pub credential: Credential,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u8,
    pub default: Option<String>,
    pub profiles: Vec<Profile>,
}

// A configuration file. Reading and selection never create files or connect.
pub struct Store {
    path: PathBuf,
}

pub struct Selection {
    pub name: Option<String>,
    pub target: Target,
    pub expected_backend_id: Option<String>,
    pub credential: Credential,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Credential {
    None,
    Keyring,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid backend profiles: {0}")]
    Invalid(String),
    #[error("Backend profile configuration is busy; retry the edit.")]
    Busy,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Filesystem(#[from] dokime_filesystem::Error),
    #[error(transparent)]
    Credentials(#[from] super::credentials::Error),
}

/* === Traits === */

/* === Implementations === */
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            default: None,
            profiles: Vec::new(),
        }
    }
}

impl Document {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_BYTES {
            return Err(invalid("file exceeds 1 MiB"));
        }
        let document: Self = serde_json::from_slice(bytes)
            .map_err(|_| invalid("file does not match the version 1 profile schema"))?;
        document.validate()?;
        Ok(document)
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|_| invalid("cannot encode profile document"))?;
        if bytes.len() > MAX_BYTES {
            return Err(invalid("file exceeds 1 MiB"));
        }
        Ok(bytes)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.version != 1 || self.profiles.len() > MAX_PROFILES {
            return Err(invalid("unsupported version or too many profiles"));
        }
        let mut names = BTreeSet::new();
        for profile in &self.profiles {
            if !names.insert(&profile.name) {
                return Err(invalid("duplicate profile name"));
            }
            profile.selection()?;
        }
        if let Some(name) = &self.default {
            self.get(name)?;
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Profile, Error> {
        validate_name(name)?;
        self.profiles
            .iter()
            .find(|profile| profile.name == name)
            .ok_or_else(|| invalid("requested profile does not exist"))
    }

    // Selection happens once; subsequent default edits cannot alter a connection.
    pub fn select(
        &self,
        backend: Option<&str>,
        endpoint: Option<&str>,
        environment: Option<&str>,
    ) -> Result<Selection, Error> {
        if backend.is_some() && endpoint.is_some() {
            return Err(invalid("backend and endpoint are mutually exclusive"));
        }
        if let Some(endpoint) = endpoint {
            let target = Target::new(endpoint, None).map_err(|_| invalid("invalid endpoint"))?;
            return Ok(Selection {
                name: None,
                target,
                expected_backend_id: None,
                credential: Credential::None,
            });
        }
        let name = backend
            .or(environment)
            .or(self.default.as_deref())
            .ok_or_else(|| invalid("select a backend profile or explicit endpoint"))?;
        self.get(name)?.selection()
    }
}

impl Profile {
    pub fn selection(&self) -> Result<Selection, Error> {
        validate_name(&self.name)?;
        if let Some(identity) = &self.expected_backend_id {
            if identity.is_empty() || identity.len() > 256 || identity.chars().any(char::is_control)
            {
                return Err(invalid("invalid expected backend identity"));
            }
        }
        if let Some(ca) = &self.ca_certificate_pem {
            if ca.is_empty() || ca.len() > MAX_BYTES || ca.contains('\0') {
                return Err(invalid("invalid CA certificate"));
            }
        }
        let target = Target::new(&self.endpoint, self.ca_certificate_pem.clone())
            .map_err(|_| invalid("invalid endpoint or CA certificate association"))?;
        Ok(Selection {
            name: Some(self.name.clone()),
            target,
            expected_backend_id: self.expected_backend_id.clone(),
            credential: self.credential,
        })
    }
}

impl Store {
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
        Ok(self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new(".")))
    }

    pub fn load(&self) -> Result<Document, Error> {
        self.parent()?;
        match dokime_filesystem::input::read(&self.path, MAX_BYTES) {
            Ok(bytes) => Document::decode(&bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Document::default()),
            Err(error) => Err(error.into()),
        }
    }

    // Called only after bearer authentication and backend identity negotiation.
    // Protect the original profile association across the preceding network IO.
    // A file publication failure can leave the credential saved separately.
    pub fn remember(
        &self,
        selection: &Selection,
        token: &str,
        secrets: &dyn SecretStore,
    ) -> Result<(), Error> {
        let name = selection
            .name
            .as_deref()
            .ok_or_else(|| invalid("a named profile is required"))?;
        self.update(|document| {
            let current = document.get(name)?.selection()?;
            if current.target != selection.target
                || current.expected_backend_id != selection.expected_backend_id
            {
                return Err(invalid(
                    "profile target changed during login; retry explicitly",
                ));
            }
            let profile = document
                .profiles
                .iter_mut()
                .find(|profile| profile.name == name)
                .expect("validated profile exists");
            profile.credential = Credential::Keyring;
            // Refuse Schema And Capacity Failures Before Saving A Secret.
            document.encode()?;
            secrets.write(selection.target.endpoint(), token)?;
            Ok(())
        })
    }

    // Edit the latest snapshot while holding a stable sibling lock. Fail fast
    // on contention; never unlink the lock file or hold a lock during network IO.
    pub fn update<T>(
        &self,
        edit: impl FnOnce(&mut Document) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let parent = self.parent()?;
        dokime_filesystem::publication::create_dir_all(parent)?;
        let mut lock_path = self.path.as_os_str().to_os_string();
        lock_path.push(".lock");
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(PathBuf::from(lock_path))?;
        match lock.try_lock() {
            Ok(()) => (),
            Err(TryLockError::WouldBlock) => return Err(Error::Busy),
            Err(TryLockError::Error(error)) => return Err(error.into()),
        }
        let mut document = self.load()?;
        let result = edit(&mut document)?;
        let bytes = document.encode()?;
        dokime_filesystem::publication::write(&self.path, &bytes)?;
        Ok(result)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(message: &str) -> Error {
    Error::Invalid(message.to_owned())
}

fn validate_name(name: &str) -> Result<(), Error> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(invalid(
            "profile names require 1–64 ASCII letters, digits, dots, underscores or hyphens",
        ));
    }
    Ok(())
}

fn environment_path(name: &str) -> Result<PathBuf, Error> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            invalid("standard configuration root is unavailable; provide an explicit profile file")
        })
}

/* === Tests === */
