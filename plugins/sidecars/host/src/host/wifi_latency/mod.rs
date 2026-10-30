/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "runtime")]
mod observing;

#[cfg(all(feature = "runtime", unix))]
mod unix;

#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

/* === Re-Exports === */

#[cfg(feature = "runtime")]
pub use observing::State;

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::HostTelemetryStream;
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::telemetry::{TelemetryContract, TelemetryStream};

/* === Constants === */

const BUDGET: Duration = Duration::from_millis(4000);
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: BUDGET,
    attempt_timeout: BUDGET,
    cancellation_timeout: std::time::Duration::ZERO,
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(
    id = "host.wifi.latency.collect",
    response = Samples,
    policy = POLICY,
    resources = [tokio::sync::Mutex<State>, crate::Workers]
)]
#[derive(Clone, Default, PartialEq)]
pub struct WifiLatency {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<Collected>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Collected {
    pub row: crate::WifiLatency,
    pub timestamp_micros: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::StreamSpec for WifiLatency {
    type Row = crate::WifiLatency;
    const ID: &'static str = "host.wifi.latency";
    const VERSION: u32 = 1;
    const WINDOW: usize = <crate::WifiLatency as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for WifiLatency {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::WifiLatency).typed::<crate::WifiLatency>()
    }
}

impl dokime::telemetry::OwnedStream for WifiLatency {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{Message, operations::OperationDeclaration};

    #[test]
    fn contract() {
        let request = WifiLatency {};
        assert_eq!(<WifiLatency as OperationDeclaration>::ID, "host.wifi.latency.collect");
        assert!(<WifiLatency as OperationDeclaration>::POLICY.timeout >= <WifiLatency as OperationDeclaration>::POLICY.attempt_timeout);
        assert!(!<WifiLatency as OperationDeclaration>::POLICY.timeout.is_zero());
        assert!(request == WifiLatency::decode(&request.encode()).expect("request round trip"));
        assert_eq!(WifiLatency::contract().storage_schema, <crate::WifiLatency as TelemetryStream>::contract().storage_schema);
    }
}
