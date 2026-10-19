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
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

// Third Party Imports
use tracing_subscriber::Layer;

// Local Imports
use super::logging::BoxedLayer;
use crate::host::{Error, Result};

/* === Constants === */

const DEV_PROFILE_ENV: &str = "DOKIME_DEV_PROFILE";
const KEEP_PROFILE_DIRS: usize = 3;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct DevProfileGuards {
    _chrome: tracing_chrome::FlushGuard,
    _flame: tracing_flame::FlushGuard<BufWriter<File>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn dev_profile_layers(root: &Path) -> Result<Option<(Vec<BoxedLayer>, DevProfileGuards)>> {
    if !dev_profile_enabled() {
        return Ok(None);
    }

    let profiles_root = profiles_root(root);
    let profile_dir = profiles_root.join(format!("dev-{}", current_epoch_millis()));
    std::fs::create_dir_all(&profile_dir)?;
    prune_stale_profile_dirs(&profiles_root);

    let (chrome_layer, chrome_guard) =
        tracing_chrome::ChromeLayerBuilder::new().file(profile_dir.join("server.trace.json")).include_args(true).build();
    let (flame_layer, flame_guard) = tracing_flame::FlameLayer::with_file(profile_dir.join("server.folded"))
        .map_err(|error| Error::Setup(format!("failed to start flame profiling: {error}")))?;

    let layers: Vec<BoxedLayer> = vec![chrome_layer.boxed(), flame_layer.boxed()];
    let guards = DevProfileGuards { _chrome: chrome_guard, _flame: flame_guard };
    Ok(Some((layers, guards)))
}

/* === Private Functions === */

fn dev_profile_enabled() -> bool {
    profile_enabled(std::env::var(DEV_PROFILE_ENV).ok().as_deref())
}

fn profile_enabled(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        let value = value.trim();
        !(value.is_empty() || value == "0" || value.eq_ignore_ascii_case("false"))
    })
}

fn profiles_root(root: &Path) -> PathBuf {
    root.join("diagnostics").join("profiles")
}

fn current_epoch_millis() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|value| value.as_millis()).unwrap_or_default()
}

fn prune_stale_profile_dirs(profiles_root: &Path) {
    let Ok(entries) = std::fs::read_dir(profiles_root) else {
        return;
    };

    let mut dirs: Vec<(u128, PathBuf)> = entries
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
        .filter_map(|entry| {
            let stamp = entry.file_name().to_str()?.strip_prefix("dev-")?.to_owned();
            let is_digits = !stamp.is_empty() && stamp.bytes().all(|byte| byte.is_ascii_digit());
            is_digits.then(|| stamp.parse::<u128>().ok()).flatten().map(|stamp| (stamp, entry.path()))
        })
        .collect();

    dirs.sort_unstable_by_key(|(stamp, _)| std::cmp::Reverse(*stamp));

    for (_, stale_dir) in dirs.into_iter().skip(KEEP_PROFILE_DIRS) {
        let _ = std::fs::remove_dir_all(stale_dir);
    }
}

/* === Tests === */
