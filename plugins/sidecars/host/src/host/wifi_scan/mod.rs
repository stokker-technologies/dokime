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

// Third Party Imports

// Local Imports
use crate::HostTelemetryStream;
use dokime::harmos::operations::OperationPolicy;
use dokime::telemetry::{TelemetryContract, TelemetryStream};

/* === Constants === */

pub const POLICY: OperationPolicy = crate::settings::collection_policy(crate::settings::WIFI_SCAN_BUDGET_MS);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(
    id = "host.wifi.scan.collect",
    response = Samples,
    policy = POLICY,
    resources = [tokio::sync::Mutex<State>, crate::Workers]
)]
#[derive(Clone, Default, PartialEq)]
pub struct WifiScan {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<Collected>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Collected {
    pub row: crate::WifiScan,
    pub timestamp_micros: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::StreamSpec for WifiScan {
    type Row = crate::WifiScan;
    const ID: &'static str = "host.wifi.scan";
    const VERSION: u32 = 1;
    const WINDOW: usize = <crate::WifiScan as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for WifiScan {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::WifiScan).typed::<crate::WifiScan>()
    }
}

impl dokime::telemetry::OwnedStream for WifiScan {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{Message, operations::OperationDeclaration};

    #[test]
    fn contract() {
        let request = WifiScan {};
        assert_eq!(<WifiScan as OperationDeclaration>::ID, "host.wifi.scan.collect");
        assert!(<WifiScan as OperationDeclaration>::POLICY.timeout >= <WifiScan as OperationDeclaration>::POLICY.attempt_timeout);
        assert!(!<WifiScan as OperationDeclaration>::POLICY.timeout.is_zero());
        assert!(request == WifiScan::decode(&request.encode()).expect("request round trip"));
        assert_eq!(WifiScan::contract().storage_schema, <crate::WifiScan as TelemetryStream>::contract().storage_schema);
    }
}
