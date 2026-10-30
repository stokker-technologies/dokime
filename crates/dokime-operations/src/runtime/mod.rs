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
use harmos::operations::OperationPolicy;
use harmos::operations::RetryPolicy;
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use dokime_domain::model::runtime::wire::*;

/* === Constants === */

const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    cancellation_timeout: std::time::Duration::ZERO,
    retries: RetryPolicy::Never,
};
pub const PACKAGE: &str = "dokime.v1";

/* === Macros === */

#[cfg(feature = "runtime")]
crate::hosted!(GetHealth, GetIdentity);

/* === Type Aliases === */

/* === Structs === */

#[harmos::operation(resources = [crate::hosting::Host<GetHealth>], response = Health, id = "dokime.runtime.health",
    description = "Read the current runtime identity and admission readiness",
    policy = POLICY, visibility = Host, grpc(package = "dokime.v1", service = "Runtime", method = "GetHealth"))]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetHealth {}

#[harmos::operation(resources = [crate::hosting::Host<GetIdentity>], response = Identity, id = "dokime.runtime.identity",
    description = "Read the application identity and process epoch",
    policy = POLICY, visibility = Host, grpc(package = "dokime.v1", service = "Runtime", method = "GetIdentity"))]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetIdentity {}

/* === Enums === */

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
}

impl super::Endpoint for GetIdentity {
    const PATH: &'static str = GetIdentity::PATH;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
