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

const BUDGET: Duration = Duration::from_millis(10000);
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: BUDGET,
    attempt_timeout: BUDGET,
    retries: RetryPolicy::Never,
};

/* === Structs === */

#[dokime::operation(
    id = "host.wifi.collect",
    response = Samples,
    policy = POLICY,
    resources = [tokio::sync::Mutex<State>, crate::Workers]
)]
#[derive(Clone, Default, PartialEq)]
pub struct Wifi {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<Collected>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Collected {
    pub row: dokime::host::Wifi,
    pub timestamp_micros: i64,
}

/* === Implementations === */

impl dokime::StreamSpec for Wifi {
    type Row = dokime::host::Wifi;
    const ID: &'static str = "host.wifi";
    const VERSION: u32 = 1;
    const WINDOW: usize = <dokime::host::Wifi as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for Wifi {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::Wifi).typed::<dokime::host::Wifi>()
    }
}

impl dokime::telemetry::OwnedStream for Wifi {}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{Message, operations::OperationDeclaration};

    #[test]
    fn contract() {
        let request = Wifi {};
        assert_eq!(<Wifi as OperationDeclaration>::ID, "host.wifi.collect");
        assert!(<Wifi as OperationDeclaration>::POLICY.timeout >= <Wifi as OperationDeclaration>::POLICY.attempt_timeout);
        assert!(!<Wifi as OperationDeclaration>::POLICY.timeout.is_zero());
        assert!(request == Wifi::decode(&request.encode()).expect("request round trip"));
        assert_eq!(Wifi::contract().storage_schema, <dokime::host::Wifi as TelemetryStream>::contract().storage_schema);
    }
}
