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

pub const POLICY: OperationPolicy = crate::settings::collection_policy(crate::settings::SYSTEM_BUDGET_MS);

/* === Macros === */

/* === Type Aliases === */

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
    pub row: crate::System,
    pub timestamp_micros: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::StreamSpec for System {
    type Row = crate::System;
    const ID: &'static str = "host.system";
    const VERSION: u32 = 1;
    const WINDOW: usize = <crate::System as dokime::StreamSpec>::WINDOW;
}

impl TelemetryStream for System {
    fn contract() -> TelemetryContract {
        crate::host_telemetry_contract(HostTelemetryStream::System).typed::<crate::System>()
    }
}

impl dokime::telemetry::OwnedStream for System {}

/* === Public Functions === */

/* === Private Functions === */

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
        assert_eq!(System::contract().storage_schema, <crate::System as TelemetryStream>::contract().storage_schema);
    }
}
