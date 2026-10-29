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
    id = "host.wifi.channel_occupancy.collect",
    response = Samples,
    policy = POLICY,
    resources = [tokio::sync::Mutex<State>, crate::Workers]
)]
#[derive(Clone, Default, PartialEq)]
pub struct WifiChannelOccupancy {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<Collected>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Collected {
    pub row: dokime::host::WifiChannelOccupancy,
    pub timestamp_micros: i64,
}

/* === Implementations === */

impl dokime::StreamSpec for WifiChannelOccupancy {
    type Row = dokime::host::WifiChannelOccupancy;
    const ID: &'static str = "host.wifi.channel_occupancy";
    const VERSION: u32 = 1;
    const WINDOW: usize = <dokime::host::WifiChannelOccupancy as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for WifiChannelOccupancy {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::WifiChannelOccupancy).typed::<dokime::host::WifiChannelOccupancy>()
    }
}

impl dokime::telemetry::OwnedStream for WifiChannelOccupancy {}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{Message, operations::OperationDeclaration};

    #[test]
    fn contract() {
        let request = WifiChannelOccupancy {};
        assert_eq!(<WifiChannelOccupancy as OperationDeclaration>::ID, "host.wifi.channel_occupancy.collect");
        assert!(
            <WifiChannelOccupancy as OperationDeclaration>::POLICY.timeout
                >= <WifiChannelOccupancy as OperationDeclaration>::POLICY.attempt_timeout
        );
        assert!(!<WifiChannelOccupancy as OperationDeclaration>::POLICY.timeout.is_zero());
        assert!(request == WifiChannelOccupancy::decode(&request.encode()).expect("request round trip"));
        assert_eq!(
            WifiChannelOccupancy::contract().storage_schema,
            <dokime::host::WifiChannelOccupancy as TelemetryStream>::contract().storage_schema
        );
    }
}
