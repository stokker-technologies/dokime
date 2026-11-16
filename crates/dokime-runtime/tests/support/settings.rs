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

// Third Party Imports

// Local Imports
use dokime_runtime::ControlledClock;
use dokime_runtime::HostSettings;
use dokime_runtime::PlacementSettings;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

/* === Constants === */

pub const SYSTEM_HANG_GUARD: Duration = Duration::from_secs(600);
/// The Story Client's Read and Mutation Deadline and the Policy Timeout of Story-Declared Operations.
pub const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn settings() -> HostSettings {
    HostSettings {
        storage: dokime_storage::storage_defaults(),
        placement: PlacementSettings {
            reconnect_delay: Duration::from_secs(1),
            stream_window: NonZeroUsize::new(8).unwrap(),
            interval_window: dokime_runtime::INTERVAL_WINDOW,
            interval_lag: dokime_runtime::INTERVAL_LAG,
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        clock: Arc::new(ControlledClock::new(0)),
        capture: dokime_runtime::CaptureSettings::default(),
    }
}

/* === Private Functions === */

/* === Tests === */
