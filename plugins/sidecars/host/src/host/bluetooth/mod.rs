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

pub const POLICY: OperationPolicy = crate::settings::collection_policy(crate::settings::BLUETOOTH_BUDGET_MS);

/* === Macros === */

/* === Type Aliases === */

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
    pub rows: Vec<Collected>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Collected {
    pub row: crate::Bluetooth,
    pub timestamp_micros: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::StreamSpec for Bluetooth {
    type Row = crate::Bluetooth;
    const ID: &'static str = "host.bluetooth";
    const VERSION: u32 = 1;
    const WINDOW: usize = <crate::Bluetooth as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for Bluetooth {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::Bluetooth).typed::<crate::Bluetooth>()
    }
}

impl dokime::telemetry::OwnedStream for Bluetooth {}

/* === Public Functions === */

/* === Private Functions === */

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
        assert_eq!(Bluetooth::contract().storage_schema, <crate::Bluetooth as TelemetryStream>::contract().storage_schema);
    }
}
