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

const BUDGET: Duration = Duration::from_millis(2000);
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: BUDGET,
    attempt_timeout: BUDGET,
    retries: RetryPolicy::Never,
};

/* === Structs === */

#[dokime::operation(
    id = "host.network.collect",
    response = Samples,
    policy = POLICY,
    resources = [tokio::sync::Mutex<State>, crate::Workers]
)]
#[derive(Clone, Default, PartialEq)]
pub struct Network {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<dokime::host::Network>,
}

/* === Implementations === */

impl dokime::StreamSpec for Network {
    type Row = dokime::host::Network;
    const ID: &'static str = "host.network";
    const VERSION: u32 = 1;
    const WINDOW: usize = <dokime::host::Network as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for Network {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::Network).typed::<dokime::host::Network>()
    }
}

impl dokime::telemetry::OwnedStream for Network {}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{Message, operations::OperationDeclaration};

    #[test]
    fn contract() {
        let request = Network {};
        assert_eq!(<Network as OperationDeclaration>::ID, "host.network.collect");
        assert!(<Network as OperationDeclaration>::POLICY.timeout >= <Network as OperationDeclaration>::POLICY.attempt_timeout);
        assert!(!<Network as OperationDeclaration>::POLICY.timeout.is_zero());
        assert!(request == Network::decode(&request.encode()).expect("request round trip"));
        assert_eq!(Network::contract().storage_schema, <dokime::host::Network as TelemetryStream>::contract().storage_schema);
    }
}
