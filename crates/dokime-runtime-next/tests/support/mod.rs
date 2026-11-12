/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, path::PathBuf, sync::Arc, time::Duration};

// Third Party Imports

// Local Imports
use dokime_runtime_next::{ControlledClock, HostSettings, PlacementSettings};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn settings() -> HostSettings {
    HostSettings {
        placement: PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_secs(1),
            stop_grace: Duration::from_secs(30),
            stream_window: NonZeroUsize::new(8).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        clock: Arc::new(ControlledClock::new(0)),
    }
}
pub fn thermal_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::var_os("DOKIME_THERMAL_PLUGIN").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug")
            .join(format!("dokime-rust-thermal{}", std::env::consts::EXE_SUFFIX))
    });
    if !path.is_file() {
        return Err("Build the Shared Thermal Example Before Running the Host Stories".into());
    }
    Ok(path)
}

/* === Private Functions === */

/* === Tests === */
