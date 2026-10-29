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
#[cfg(feature = "runtime")]
pub use observing::State;

#[cfg(all(feature = "runtime", unix))]
mod unix;
#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

/* === Imports === */

use crate::HostTelemetryStream;
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::telemetry::{TelemetryContract, TelemetryStream};
use std::time::Duration;

/* === Constants === */

const BUDGET: Duration = Duration::from_millis(12000);
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: BUDGET,
    attempt_timeout: BUDGET,
    retries: RetryPolicy::Never,
};

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
    pub row: dokime::host::WifiScan,
    pub timestamp_micros: i64,
}

/* === Implementations === */

impl dokime::StreamSpec for WifiScan {
    type Row = dokime::host::WifiScan;
    const ID: &'static str = "host.wifi.scan";
    const VERSION: u32 = 1;
    const WINDOW: usize = <dokime::host::WifiScan as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for WifiScan {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::WifiScan).typed::<dokime::host::WifiScan>()
    }
}

impl dokime::telemetry::OwnedStream for WifiScan {}

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
        assert_eq!(WifiScan::contract().storage_schema, <dokime::host::WifiScan as TelemetryStream>::contract().storage_schema);
    }
}
