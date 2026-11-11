/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared Fixture Cadences and Boundaries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports

/* === Constants === */

pub const CADENCE_MS: u64 = 1;
pub const CADENCE: Duration = Duration::from_millis(CADENCE_MS);
// File Capture Admission Retains Its Product Minimum.
pub const FILE_POLL: Duration = Duration::from_millis(50);
pub const STATE_TAIL: usize = 4;
pub const PAGE_ROWS: usize = 2;
pub const EXPORT_CHUNK_BYTES: usize = 64 * 1024 + 1;
// Bounds Fixture Operations And In-Story Waits; Sized For Hosted-Runner Disk Stalls.
pub const STORY_BOUND: Duration = Duration::from_secs(240);
pub const FIXTURE_POLICY: super::OperationPolicy = super::OperationPolicy {
    timeout: STORY_BOUND,
    attempt_timeout: STORY_BOUND,
    cancellation_timeout: Duration::ZERO,
    retries: super::RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn configure(config: &mut super::RuntimeConfig) {
    config.local_storage.telemetry_interval = CADENCE;
    config.storage.filesystem.poll_interval = FILE_POLL;
    config.managed_storage.journal_tail = STATE_TAIL;
}

/* === Private Functions === */

/* === Tests === */
