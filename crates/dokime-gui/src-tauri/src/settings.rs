/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Desktop Settings: Everything This Desktop Remembers, in One File Under Its Application Root

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    io::Read,
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::app::updates::policy::Mode;
use dokime_runtime::{INTERVAL_LAG, INTERVAL_WINDOW, PlacementSettings};

/* === Constants === */

/// The Largest Window the Desktop Can Represent Exactly as a Number.
const MAX_WINDOW_MS: u64 = (1 << 53) - 1;

const MAX_PARAMETER_BYTES: usize = 1024 * 1024;

const MAX_UPDATE_BYTES: usize = 16_384;

/// Every Section at Its Own Bound, Plus the Interval Keys.
const MAX_BYTES: usize = MAX_PARAMETER_BYTES + MAX_UPDATE_BYTES + 4096;

/* === Macros === */

/* === Type Aliases === */

/// Stable Template ID, Then Stable Parameter ID, Then the Edited Raw Input.
pub type Values = BTreeMap<String, BTreeMap<String, Value>>;

/* === Structs === */

/// The settings file; an absent key is its default and an unknown key is ignored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesktopSettings {
    /// Named as the standalone host's configuration names it.
    pub interval_window_ms: u64,

    /// Named as the standalone host's configuration names it.
    pub interval_lag_windows: NonZeroU32,

    /// Absent until the operator first edits a parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Values>,

    /// Absent until the operator first chooses an update source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates: Option<UpdateSelection>,
}

/// One remembered parameter input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Value {
    pub raw: String,

    #[serde(rename = "valueType")]
    pub value_type: String,

    pub unit: Option<String>,
}

/// The update mode and source the operator chose among the organization's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateSelection {
    pub mode: Mode,
    pub source: Option<String>,
}

/// The one writer: each change reads, edits and publishes the whole file under one lock.
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    serial: Mutex<()>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            interval_window_ms: INTERVAL_WINDOW.as_millis() as u64,
            interval_lag_windows: INTERVAL_LAG,
            parameters: None,
            updates: None,
        }
    }
}

impl DesktopSettings {
    /// Sets the placement's interval window and lag bound from these values.
    pub fn apply(&self, placement: &mut PlacementSettings) {
        placement.interval_window = Duration::from_millis(self.interval_window_ms);
        placement.interval_lag = self.interval_lag_windows;
    }
}

impl Store {
    pub fn new(path: PathBuf) -> Self {
        Self { path, serial: Mutex::new(()) }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads the file, applies the change and publishes the result; a file that cannot be read is left as it is.
    pub fn update<T>(&self, change: impl FnOnce(&mut DesktopSettings) -> std::io::Result<T>) -> std::io::Result<T> {
        let _serial = self.serial.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut settings = load(&self.path)?;
        let result = change(&mut settings)?;
        validate(&settings)?;
        let bytes = serde_json::to_vec(&settings).map_err(std::io::Error::other)?;
        harmos_runtime::filesystem::publication::write(&self.path, &bytes)?;
        Ok(result)
    }
}

/* === Public Functions === */

/// The desktop's one settings file.
pub fn path(root: &Path) -> PathBuf {
    root.join("settings.json")
}

/// The saved settings; an absent file is the defaults.
pub fn load(path: &Path) -> std::io::Result<DesktopSettings> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(DesktopSettings::default()),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid(format!("Read Desktop Settings: Correct or Remove {}", path.display())));
    }
    let settings: DesktopSettings = serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("Read Desktop Settings: Correct or Remove {}: {error}", path.display())))?;
    validate(&settings)?;
    Ok(settings)
}

/// The saved settings for starting the host; an unreadable file starts it with the defaults.
pub fn resolve(path: &Path) -> DesktopSettings {
    load(path).unwrap_or_else(|error| {
        tracing::warn!(%error, "Read Desktop Settings");
        DesktopSettings::default()
    })
}

/* === Private Functions === */

fn validate(settings: &DesktopSettings) -> std::io::Result<()> {
    if !(1..=MAX_WINDOW_MS).contains(&settings.interval_window_ms) {
        return Err(invalid(format!("Save Interval Settings: Enter an Interval Window from 1 to {MAX_WINDOW_MS} ms")));
    }
    if let Some(values) = &settings.parameters {
        validate_parameters(values)?;
    }
    if let Some(updates) = &settings.updates
        && serde_json::to_vec(updates).map_err(std::io::Error::other)?.len() > MAX_UPDATE_BYTES
    {
        return Err(invalid("Save Update Selection: Keep It Within Its Byte Budget".into()));
    }
    Ok(())
}

fn validate_parameters(values: &Values) -> std::io::Result<()> {
    let valid_id = |id: &str| !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control);
    let count = values.values().map(BTreeMap::len).sum::<usize>();
    if values.len() > 4096
        || count > 4096
        || values.iter().any(|(template, parameters)| {
            !valid_id(template)
                || parameters.iter().any(|(id, value)| {
                    !valid_id(id)
                        || value.raw.len() > 8192
                        || !valid_id(&value.value_type)
                        || value.unit.as_ref().is_some_and(|unit| unit.len() > 256)
                })
        })
    {
        return Err(invalid("Save Remembered Parameters: Correct the Invalid Values".into()));
    }
    if serde_json::to_vec(values).map_err(std::io::Error::other)?.len() > MAX_PARAMETER_BYTES {
        return Err(invalid("Save Remembered Parameters: Keep Them Under 1 MiB".into()));
    }
    Ok(())
}

fn invalid(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{DesktopSettings, Store, load, path};
    use std::num::NonZeroU32;

    #[test]
    fn settings_default_persist_ignore_unknown_keys_and_refuse_invalid_values() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::new(path(root.path()));
        assert_eq!(load(store.path()).unwrap(), DesktopSettings::default());
        store
            .update(|settings| {
                settings.interval_window_ms = 250;
                settings.interval_lag_windows = NonZeroU32::new(4).unwrap();
                Ok(())
            })
            .unwrap();
        let edited = load(store.path()).unwrap();
        assert_eq!((edited.interval_window_ms, edited.interval_lag_windows.get()), (250, 4));
        assert!(
            store
                .update(|settings| {
                    settings.interval_window_ms = 0;
                    Ok(())
                })
                .is_err()
        );
        assert_eq!(load(store.path()).unwrap(), edited);
        std::fs::write(store.path(), br#"{"interval_window_ms":250,"capture_folders":["D:/device"]}"#).unwrap();
        assert_eq!(load(store.path()).unwrap().interval_lag_windows, DesktopSettings::default().interval_lag_windows);
        std::fs::write(store.path(), br#"{"interval_lag_windows":0}"#).unwrap();
        assert!(load(store.path()).is_err());
        assert!(store.update(|_| Ok(())).is_err());
        assert_eq!(std::fs::read(store.path()).unwrap(), br#"{"interval_lag_windows":0}"#);
    }
}
