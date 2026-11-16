/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Desktop Interval Settings: The Window Length and Lag Bound the Built-In Host Places Streams With

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::Read,
    num::NonZeroU32,
    path::{Path, PathBuf},
    time::Duration,
};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use dokime_runtime::{INTERVAL_LAG, INTERVAL_WINDOW, PlacementSettings};

/* === Constants === */

const MAX_BYTES: u64 = 4096;

/// The Largest Window the Desktop Can Represent Exactly as a Number.
const MAX_WINDOW_MS: u64 = (1 << 53) - 1;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalSettings {
    pub interval_window_ms: u64,
    pub interval_lag_windows: NonZeroU32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for IntervalSettings {
    fn default() -> Self {
        Self {
            interval_window_ms: INTERVAL_WINDOW.as_millis() as u64,
            interval_lag_windows: INTERVAL_LAG,
        }
    }
}

impl IntervalSettings {
    /// Sets the placement's interval window and lag bound from these values.
    pub fn apply(&self, placement: &mut PlacementSettings) {
        placement.interval_window = Duration::from_millis(self.interval_window_ms);
        placement.interval_lag = self.interval_lag_windows;
    }
}

/* === Public Functions === */

/// The desktop-owned file beside the application's other remembered settings.
pub fn path(root: &Path) -> PathBuf {
    root.join("interval-settings.json")
}

/// The saved values; an absent file is the host defaults.
pub fn load(path: &Path) -> std::io::Result<IntervalSettings> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(IntervalSettings::default()),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(invalid(format!("Read Interval Settings: Correct or Remove {}", path.display())));
    }
    let settings: IntervalSettings = serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("Read Interval Settings: Correct or Remove {}: {error}", path.display())))?;
    validate(&settings)?;
    Ok(settings)
}

pub fn save(path: &Path, settings: &IntervalSettings) -> std::io::Result<()> {
    validate(settings)?;
    let bytes = serde_json::to_vec(settings).map_err(std::io::Error::other)?;
    harmos_runtime::filesystem::publication::write(path, &bytes).map_err(Into::into)
}

/// The saved values for starting the host; an unreadable file starts it with the defaults.
pub fn resolve(path: &Path) -> IntervalSettings {
    load(path).unwrap_or_else(|error| {
        tracing::warn!(%error, "Read Interval Settings");
        IntervalSettings::default()
    })
}

/* === Private Functions === */

fn validate(settings: &IntervalSettings) -> std::io::Result<()> {
    if !(1..=MAX_WINDOW_MS).contains(&settings.interval_window_ms) {
        return Err(invalid(format!("Save Interval Settings: Enter an Interval Window from 1 to {MAX_WINDOW_MS} ms")));
    }
    Ok(())
}

fn invalid(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{IntervalSettings, load, path, save};
    use std::num::NonZeroU32;

    #[test]
    fn settings_default_persist_and_refuse_zero_values() {
        let root = tempfile::tempdir().unwrap();
        let file = path(root.path());
        assert_eq!(load(&file).unwrap(), IntervalSettings::default());
        let edited = IntervalSettings {
            interval_window_ms: 250,
            interval_lag_windows: NonZeroU32::new(4).unwrap(),
        };
        save(&file, &edited).unwrap();
        assert_eq!(load(&file).unwrap(), edited);
        assert!(save(&file, &IntervalSettings { interval_window_ms: 0, ..edited }).is_err());
        assert_eq!(load(&file).unwrap(), edited);
        std::fs::write(&file, br#"{"interval_window_ms":250,"interval_lag_windows":0}"#).unwrap();
        assert!(load(&file).is_err());
    }
}
