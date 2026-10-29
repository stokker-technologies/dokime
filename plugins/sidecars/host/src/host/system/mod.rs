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
    id = "host.system.collect",
    response = Samples,
    policy = POLICY,
    resources = [tokio::sync::Mutex<State>, crate::Workers]
)]
#[derive(Clone, Default, PartialEq)]
pub struct System {}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<Collected>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Collected {
    pub row: dokime::host::System,
    pub timestamp_micros: i64,
}

/* === Implementations === */

impl dokime::StreamSpec for System {
    type Row = dokime::host::System;
    const ID: &'static str = "host.system";
    const VERSION: u32 = 1;
    const WINDOW: usize = <dokime::host::System as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for System {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::System).typed::<dokime::host::System>()
    }
}

impl dokime::telemetry::OwnedStream for System {}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::harmos::{Message, operations::OperationDeclaration};

    #[test]
    fn contract() {
        let request = System {};
        assert_eq!(<System as OperationDeclaration>::ID, "host.system.collect");
        assert!(<System as OperationDeclaration>::POLICY.timeout >= <System as OperationDeclaration>::POLICY.attempt_timeout);
        assert!(!<System as OperationDeclaration>::POLICY.timeout.is_zero());
        assert!(request == System::decode(&request.encode()).expect("request round trip"));
        assert_eq!(System::contract().storage_schema, <dokime::host::System as TelemetryStream>::contract().storage_schema);
    }
}
