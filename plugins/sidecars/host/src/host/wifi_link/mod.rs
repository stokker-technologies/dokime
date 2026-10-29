/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[cfg(feature = "runtime")]
mod observing;

#[cfg(all(feature = "runtime", unix))]
mod unix;

#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

#[cfg(feature = "runtime")]
pub use observing::State;

use std::time::Duration;

use crate::HostTelemetryStream;
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::telemetry::{TelemetryContract, TelemetryStream};

const BUDGET: Duration = Duration::from_millis(2000);
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: BUDGET,
    attempt_timeout: BUDGET,
    retries: RetryPolicy::Never,
};

#[dokime::operation(
    id = "host.wifi.link.collect",
    response = Samples,
    policy = POLICY,
    resources = [tokio::sync::Mutex<State>, crate::Workers]
)]
#[derive(Clone, Default, PartialEq)]
pub struct WifiLink {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<Collected>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Collected {
    pub row: crate::WifiLink,
    pub timestamp_micros: i64,
}

impl dokime::StreamSpec for WifiLink {
    type Row = crate::WifiLink;
    const ID: &'static str = "host.wifi.link";
    const VERSION: u32 = 1;
    const WINDOW: usize = <crate::WifiLink as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for WifiLink {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::WifiLink).typed::<crate::WifiLink>()
    }
}

impl dokime::telemetry::OwnedStream for WifiLink {}

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{Message, operations::OperationDeclaration};

    #[test]
    fn contract() {
        let request = WifiLink {};
        assert_eq!(<WifiLink as OperationDeclaration>::ID, "host.wifi.link.collect");
        assert!(<WifiLink as OperationDeclaration>::POLICY.timeout >= <WifiLink as OperationDeclaration>::POLICY.attempt_timeout);
        assert!(!<WifiLink as OperationDeclaration>::POLICY.timeout.is_zero());
        assert!(request == WifiLink::decode(&request.encode()).expect("request round trip"));
        assert_eq!(WifiLink::contract().storage_schema, <crate::WifiLink as TelemetryStream>::contract().storage_schema);
    }
}
