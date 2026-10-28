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

const BUDGET: Duration = Duration::from_millis(6000);
pub const POLICY: OperationPolicy = OperationPolicy {
    timeout: BUDGET,
    attempt_timeout: BUDGET,
    retries: RetryPolicy::Never,
};

/* === Structs === */

#[dokime::operation(
    id = "host.bluetooth.collect",
    response = Samples,
    policy = POLICY,
    resources = [tokio::sync::Mutex<State>, crate::Workers]
)]
#[derive(Clone, Default, PartialEq)]
pub struct Bluetooth {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<dokime::host::Bluetooth>,
}

/* === Implementations === */

impl dokime::StreamSpec for Bluetooth {
    type Row = dokime::host::Bluetooth;
    const ID: &'static str = "host.bluetooth";
    const VERSION: u32 = 1;
    const WINDOW: usize = <dokime::host::Bluetooth as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for Bluetooth {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::Bluetooth).typed::<dokime::host::Bluetooth>()
    }
}

impl dokime::telemetry::OwnedStream for Bluetooth {}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{Message, operations::OperationDeclaration};

    #[test]
    fn contract() {
        let request = Bluetooth {};
        assert_eq!(<Bluetooth as OperationDeclaration>::ID, "host.bluetooth.collect");
        assert!(<Bluetooth as OperationDeclaration>::POLICY.timeout >= <Bluetooth as OperationDeclaration>::POLICY.attempt_timeout);
        assert!(!<Bluetooth as OperationDeclaration>::POLICY.timeout.is_zero());
        assert!(request == Bluetooth::decode(&request.encode()).expect("request round trip"));
        assert_eq!(Bluetooth::contract().storage_schema, <dokime::host::Bluetooth as TelemetryStream>::contract().storage_schema);
    }
}
