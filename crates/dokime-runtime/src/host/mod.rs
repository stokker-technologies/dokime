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
pub use dokime_domain::model::telemetry::host::contracts::HostTextList;
pub use dokime_domain::model::telemetry::host::contracts::HostTimestamp;
pub use streams::{Bluetooth, Network, System, Wifi, WifiChannelOccupancy, WifiLatency};

// The host's own vocabulary for its streams, which its collector and its
// contracts are written against. A template names the types above instead.
pub use dokime_domain::model::telemetry::HOST_STREAMS;
pub use dokime_domain::model::telemetry::HostTelemetryStream;
pub use dokime_domain::model::telemetry::host_stream_identifiers;

// The records the host itself files, beside the streams it samples.
pub use crate::records::host::{
    ProcessExited, StepHandleCancelled, TargetAcquired, TargetLost, TestCycleCompleted, ToolConnectionAcquired, ToolConnectionLost,
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
