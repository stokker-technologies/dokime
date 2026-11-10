/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Pairing File Folder and Public Backend Listings

/* === Modules === */

/* === Re-Exports === */

pub use dokime_domain::model::backends::{Document, Profile};

/* === Imports === */

// Standard Library Imports
use std::{
    env, io,
    path::{Path, PathBuf},
};

// Third Party Imports
use zeroize::Zeroizing;

// Local Imports
use super::Target;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Store {
    path: PathBuf,
}

pub struct Selection {
    pub identifier: Option<String>,
    pub target: Target,
    pub expected_backend_id: Option<String>,
    pub access_key: Option<Zeroizing<String>>,
}

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid pairing file: {0}")]
    Invalid(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Filesystem(#[from] harmos::filesystem::Error),
}

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn discover(override_path: Option<&Path>) -> Result<Self, Error> {
        if let Some(path) = override_path {
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
        Ok(Self::new(base.join("dokime/remotes")))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Document, Error> {
        let entries = match std::fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Document::default()),
            Err(error) => return Err(error.into()),
        };
        let mut profiles = Vec::new();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("pair") {
                continue;
            }
            let pairing = super::pairing::read(&path)?;
            let identifier = path.file_stem().and_then(|value| value.to_str()).ok_or_else(|| invalid("invalid pairing filename"))?;
            profiles.push(profile(&pairing, identifier, &path)?);
        }
        profiles.sort_by(|left, right| left.identifier.cmp(&right.identifier));
        Ok(Document { profiles })
    }

    pub fn import_pairing(&self, pairing: &super::pairing::PairingFile, identifier: &str) -> Result<Profile, Error> {
        validate_identifier(identifier)?;
        pairing.validate().map_err(Error::Invalid)?;
        harmos::filesystem::publication::create_dir_all(&self.path)?;
        let path = self.path.join(format!("{identifier}.pair"));
        let bytes = Zeroizing::new(serde_json::to_vec_pretty(pairing).map_err(|_| invalid("cannot encode pairing file"))?);
        // Publication Owns Private File Permissions And Refuses Replacing Another Pairing.
        let mut staged = tempfile::NamedTempFile::new_in(&self.path)?;
        use std::io::Write;
        staged.write_all(&bytes)?;
        staged.as_file().sync_all()?;
        harmos::filesystem::publication::install(staged.path(), &path)?;
        profile(pairing, identifier, &path)
    }

    pub fn remove(&self, identifier: &str) -> Result<(), Error> {
        validate_identifier(identifier)?;
        std::fs::remove_file(self.path.join(format!("{identifier}.pair")))?;
        Ok(())
    }
}

/* === Public Functions === */

pub fn get<'a>(document: &'a Document, identifier: &str) -> Result<&'a Profile, Error> {
    validate_identifier(identifier)?;
    document.profiles.iter().find(|profile| profile.identifier == identifier).ok_or_else(|| invalid("pairing file does not exist"))
}

pub fn selection(profile: &Profile) -> Result<Selection, Error> {
    let mut selection = super::pairing::selection(&super::pairing::read(Path::new(&profile.pairing_file))?)?;
    selection.identifier = Some(profile.identifier.clone());
    Ok(selection)
}

/* === Private Functions === */

fn profile(pairing: &super::pairing::PairingFile, identifier: &str, path: &Path) -> Result<Profile, Error> {
    Ok(Profile {
        identifier: identifier.to_owned(),
        label: pairing.label.clone(),
        endpoint: pairing.endpoints[0].clone(),
        endpoints: pairing.endpoints.clone(),
        expected_backend_id: Some(pairing.backend_identifier.clone()),
        certificate_pem: pairing.certificate_pem.clone(),
        pairing_file: std::path::absolute(path)?.to_str().ok_or_else(|| invalid("pairing path must be UTF-8"))?.to_owned(),
    })
}

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
        .ok_or_else(|| invalid("standard configuration root is unavailable"))
}

/* === Tests === */
