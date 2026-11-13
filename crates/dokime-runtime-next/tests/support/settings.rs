/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host Test Settings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports

// Local Imports
use dokime_runtime_next::{ControlledClock, HostSettings, PlacementSettings};

/* === Constants === */

pub const SYSTEM_HANG_GUARD: Duration = Duration::from_secs(600);

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

/* === Private Functions === */

/* === Tests === */
