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

pub const ID: &str = "dokime.backend.get_availability";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::GetBackendAvailability, v1::BackendAvailability>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::GetBackendAvailability) -> Result<v1::BackendAvailability> {
    super::complete(ID, context.settings.read_timeout, async {
        let _ = request;
        let identity = context.host.identity();
        Ok(v1::BackendAvailability {
            backend_id: identity.backend_id,
            epoch: identity.epoch,
            methods: super::METHODS.iter().map(|method| (*method).into()).collect(),
            limits: Some(v1::BackendLimits {
                max_request_bytes: context.settings.message_bytes as u64,
                max_response_bytes: context.settings.message_bytes as u64,
                max_page_items: Some(context.settings.page_items as u32),
                max_stream_buffer_items: Some(context.settings.stream_buffer as u32),
                ..Default::default()
            }),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
