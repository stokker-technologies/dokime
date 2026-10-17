/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use harmos::operations::{OperationPolicy, RestartPolicy, RetryPolicy};
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5), attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never, restart: RestartPolicy::Never,
};
pub const PACKAGE: &str = "dokime.v1";
pub const SERVICE: &str = "Runtime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[harmos::operation(response = Health, id = "dokime.runtime.health", version = 1,
    description = "Read the current runtime identity and admission readiness",
    policy = POLICY, grpc(package = "dokime.v1", service = "Runtime", method = "GetHealth"))]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetHealth {}

#[harmos::message]
#[harmos::operation(response = Identity, id = "dokime.runtime.identity", version = 1,
    description = "Read the application identity and process epoch",
    policy = POLICY, grpc(package = "dokime.v1", service = "Runtime", method = "GetIdentity"))]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetIdentity {}

#[harmos::message]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub backend_id: String,
    pub epoch: String,
    pub display_name: String,
    pub product_version: String,
}

#[harmos::message]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Health {
    pub backend_id: String,
    pub epoch: String,
    pub readiness: Readiness,
    pub observed_at_seconds: i64,
    pub observed_at_nanoseconds: u32,
    pub message: String,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Readiness {
    Starting,
    Ready,
    Degraded,
    Stopping,
}

/* === Traits === */

/* === Implementations === */
impl GetHealth {
    pub const PATH: &str = "/dokime.v1.Runtime/GetHealth";
}
impl GetIdentity {
    pub const PATH: &str = "/dokime.v1.Runtime/GetIdentity";
}
impl super::Endpoint for GetHealth {
    const PATH: &'static str = GetHealth::PATH;
    const ACCESS: super::Access = super::Access::Read;
}
impl super::Endpoint for GetIdentity {
    const PATH: &'static str = GetIdentity::PATH;
    const ACCESS: super::Access = super::Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
