/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.runtime.health";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::RuntimeGetHealthRequest, v1::BackendHealth>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::RuntimeGetHealthRequest) -> Result<v1::BackendHealth> {
    super::complete(ID, context.settings.read_timeout, async {
        let _ = request;
        let identity = context.host.identity();
        Ok(v1::BackendHealth {
            backend_id: identity.backend_id,
            epoch: identity.epoch,
            readiness: v1::BackendReadiness::Ready as i32,
            observed_at: Some(super::projection::timestamp(context.host.now_ns())),
            message: "Ready".into(),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
