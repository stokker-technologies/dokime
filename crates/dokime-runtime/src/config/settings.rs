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
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::SystemTime,
};

// Third Party Imports
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Local Imports
use super::{ApplicationPaths, StoragePolicy};
use crate::plugins::PluginSettings;
use dokime_domain::model::identity::DokimeNodeIdentity;
use dokime_domain::model::identity::NodeId;

/* === Constants === */

const MAX_BYTES: u64 = 16 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct Settings {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    paths: ApplicationPaths,
    current: Mutex<Current>,
}

#[derive(Debug)]
struct Current {
    document: SettingsDocument,
    stamp: Option<(SystemTime, u64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsDocument {
    pub identity: DokimeNodeIdentity,

    #[serde(default)]
    pub data_root: Option<PathBuf>,

    #[serde(default)]
    pub storage: Option<StoragePolicy>,

    #[serde(default)]
    pub appearance: Appearance,

    #[serde(default)]
    pub report: ReportDefaults,

    #[serde(default)]
    pub plugins: BTreeMap<Uuid, PluginSettings>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Appearance {
    pub theme_mode: ThemeMode,
    pub theme_family: Option<String>,
    pub density: Density,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReportDefaults {
    pub plot_points: u32,
    pub plot_downsample: crate::document::DownsampleTechnique,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    #[default]
    System,

    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    #[default]
    Comfortable,

    Compact,
    Touch,
}

/* === Traits === */

/* === Implementations === */

impl Default for ReportDefaults {
    fn default() -> Self {
        Self { plot_points: crate::document::DEFAULT_PLOT_MAX_POINTS as u32, plot_downsample: crate::document::DownsampleTechnique::Auto }
    }
}

impl ReportDefaults {
    pub fn apply(&self, document: &mut crate::document::DocumentDefinition) {
        let bodies = document
            .cards
            .iter_mut()
            .map(|card| &mut card.body)
            .chain(document.sections.iter_mut().flat_map(|section| section.cards.iter_mut().map(|card| &mut card.body)));
        for body in bodies {
            if let crate::document::DocumentCardBody::Plot { plot } = body {
                plot.display_points.get_or_insert(self.plot_points);
                if matches!(plot.downsample, crate::document::DownsampleTechnique::Auto | crate::document::DownsampleTechnique::Unspecified)
                {
                    plot.downsample = self.plot_downsample;
                }
            }
        }
    }
}

impl Settings {
    pub fn load(application_root: impl Into<PathBuf>) -> std::io::Result<Self> {
        let root = application_root.into();
        let root = if root.is_absolute() { root } else { std::env::current_dir()?.join(root) };
        let paths = ApplicationPaths::new(root);
        let path = paths.settings();
        let document = match read(&path)? {
            Some(document) => document,
            None => create(&path)?,
        };
        Ok(Self { inner: Arc::new(Inner { paths, current: Mutex::new(Current { document, stamp: stamp(&path) }) }) })
    }

    pub fn paths(&self) -> &ApplicationPaths {
        &self.inner.paths
    }

    pub fn read<T>(&self, view: impl FnOnce(&SettingsDocument) -> T) -> T {
        let mut current = self.lock();
        if let Err(error) = self.refresh(&mut current) {
            tracing::warn!(%error, "keeping the last readable settings");
        }
        view(&current.document)
    }

    pub fn data_root(&self) -> PathBuf {
        self.read(|document| document.data_root.clone()).unwrap_or_else(|| self.paths().default_data())
    }

    pub fn edit<T, E>(&self, change: impl FnOnce(&mut SettingsDocument) -> Result<T, E>) -> Result<T, E>
    where
        E: From<std::io::Error>,
    {
        let mut current = self.lock();
        self.refresh(&mut current)?;
        let mut next = current.document.clone();
        let result = change(&mut next)?;
        if next == current.document {
            return Ok(result);
        }
        if next.identity != current.document.identity {
            return Err(invalid("the node identity is never rewritten").into());
        }
        let path = self.inner.paths.settings();
        write(&path, &next)?;
        current.document = next;
        current.stamp = stamp(&path);
        Ok(result)
    }

    fn lock(&self) -> MutexGuard<'_, Current> {
        // The document is replaced only after a successful write, so a
        // panicking reader cannot leave it half-changed.
        self.inner.current.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn refresh(&self, current: &mut Current) -> std::io::Result<()> {
        let path = self.inner.paths.settings();
        let observed = stamp(&path);
        if observed.is_some() && observed == current.stamp {
            return Ok(());
        }
        let document = read(&path)?.ok_or_else(|| invalid("settings file disappeared while in use"))?;
        if document.identity != current.document.identity {
            return Err(invalid("the node identity changed while in use"));
        }
        current.document = document;
        current.stamp = observed;
        Ok(())
    }
}

impl SettingsDocument {
    fn validate(&self) -> std::io::Result<()> {
        if self.data_root.as_ref().is_some_and(|root| !root.is_absolute()) {
            return Err(invalid("data root must be an absolute path"));
        }
        if let Some(storage) = &self.storage {
            storage.validate().map_err(|error| invalid(error.to_string()))?;
        }
        for (id, plugin) in &self.plugins {
            plugin.validate().map_err(|error| invalid(format!("plugin `{id}`: {error}")))?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn read(path: &Path) -> std::io::Result<Option<SettingsDocument>> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(invalid("settings file exceeds its size limit"));
    }
    let document: SettingsDocument = serde_json::from_slice(&bytes).map_err(invalid)?;
    document.validate()?;
    Ok(Some(document))
}

fn create(path: &Path) -> std::io::Result<SettingsDocument> {
    let document = SettingsDocument {
        identity: DokimeNodeIdentity::new(crate::model::identity::new_id::<NodeId>(), hostname_label(), crate::model::time::now_utc()),
        data_root: None,
        storage: None,
        appearance: Appearance::default(),
        report: ReportDefaults::default(),
        plugins: BTreeMap::new(),
    };
    let root = path.parent().expect("settings file has a parent");
    harmos::filesystem::publication::create_dir_all(root)?;
    let mut temporary = tempfile::NamedTempFile::new_in(root)?;
    std::io::Write::write_all(&mut temporary, &encode(&document)?)?;
    temporary.as_file().sync_all()?;
    match harmos::filesystem::publication::install(temporary.path(), path) {
        Ok(()) => Ok(document),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            harmos::filesystem::publication::confirm(path)?;
            read(path)?.ok_or_else(|| invalid("settings file vanished during creation"))
        }
        Err(error) => Err(error.into()),
    }
}

fn write(path: &Path, document: &SettingsDocument) -> std::io::Result<()> {
    document.validate()?;
    harmos::filesystem::publication::write(path, &encode(document)?).map_err(Into::into)
}

fn encode(document: &SettingsDocument) -> std::io::Result<Vec<u8>> {
    let bytes = serde_json::to_vec_pretty(document).map_err(invalid)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(invalid("settings file exceeds its size limit"));
    }
    Ok(bytes)
}

fn stamp(path: &Path) -> Option<(SystemTime, u64)> {
    let metadata = std::fs::metadata(path).ok()?;
    Some((metadata.modified().ok()?, metadata.len()))
}

fn hostname_label() -> Option<String> {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn invalid(error: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, format!("settings.json: {error}"))
}

/* === Tests === */
