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
use harmos::operations::{OperationDeclaration, OperationPolicy, RetryPolicy, WireOperation};
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */
pub const PACKAGE: &str = "dokime.v1";
pub const SERVICE: &str = "Runtime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Request {}

pub struct GetHealth;

pub struct GetIdentity;

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

impl OperationDeclaration for GetHealth {
    type Request = Request;
    type Response = Health;

    const ID: &'static str = "dokime.runtime.health";
    const DESCRIPTION: &'static str = "Read the current runtime identity and admission readiness";
    const POLICY: OperationPolicy = OperationPolicy {
        timeout: Duration::from_secs(5),
        attempt_timeout: Duration::from_secs(5),
        retries: RetryPolicy::Never,
    };
}

impl WireOperation for GetHealth {
    const VERSION: u32 = 1;
    const GRPC: Option<harmos::operations::Grpc> =
        Some(harmos::operations::Grpc::new(PACKAGE, SERVICE, "GetHealth"));

    harmos::__private::with_protobuf! {
        fn visit_grpc<V: harmos::operations::GrpcVisitor>(visitor: V) -> Option<V::Entry> {
            Some(visitor.unary::<Self>())
        }
    }
}

impl super::Endpoint for GetHealth {
    const PATH: &'static str = GetHealth::PATH;
    const ACCESS: super::Access = super::Access::Read;
}

impl GetIdentity {
    pub const PATH: &str = "/dokime.v1.Runtime/GetIdentity";
}

impl OperationDeclaration for GetIdentity {
    type Request = Request;
    type Response = Identity;

    const ID: &'static str = "dokime.runtime.identity";
    const DESCRIPTION: &'static str = "Read the application identity and process epoch";
    const POLICY: OperationPolicy = OperationPolicy {
        timeout: Duration::from_secs(5),
        attempt_timeout: Duration::from_secs(5),
        retries: RetryPolicy::Never,
    };
}

impl WireOperation for GetIdentity {
    const VERSION: u32 = 1;
    const GRPC: Option<harmos::operations::Grpc> = Some(harmos::operations::Grpc::new(
        PACKAGE,
        SERVICE,
        "GetIdentity",
    ));

    harmos::__private::with_protobuf! {
        fn visit_grpc<V: harmos::operations::GrpcVisitor>(visitor: V) -> Option<V::Entry> {
            Some(visitor.unary::<Self>())
        }
    }
}

impl super::Endpoint for GetIdentity {
    const PATH: &'static str = GetIdentity::PATH;
    const ACCESS: super::Access = super::Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
