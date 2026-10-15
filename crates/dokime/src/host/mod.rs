/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod streams;

/* === Re-Exports === */
pub use streams::{Bluetooth, Network, System, Wifi, WifiChannelOccupancy, WifiLatency};

// The host's own vocabulary for its streams, which its collector and its
// contracts are written against. A template names the types above instead.
pub use dokime_contract::telemetry::{HOST_STREAMS, HostTelemetryStream, host_stream_identifiers};

// The records the host itself files, beside the streams it samples.
pub use crate::records::host::{
    ProcessExited, StepHandleCancelled, TargetAcquired, TargetLost, TestCycleCompleted,
    ToolConnectionAcquired, ToolConnectionLost,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
