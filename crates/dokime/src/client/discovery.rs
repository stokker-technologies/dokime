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

// Third Party Imports
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::Channel,
};

// Local Imports
use super::{Backend, Error, Limits, Result, request};
use dokime_protocol::v1::{self, backend_client::BackendClient};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) async fn discover(
    channel: Channel,
    authorization: Option<&MetadataValue<Ascii>>,
    expected_id: Option<&str>,
) -> Result<Backend> {
    let mut client = BackendClient::new(channel).max_decoding_message_size(1024 * 1024);
    let identity = client
        .get_identity(request(v1::GetIdentityRequest {}, authorization))
        .await
        .map_err(Error::Rpc)?
        .into_inner();
    let api = identity.api.ok_or(Error::IncompatibleApi)?;
    // Compatibility is negotiated from the stable package/version and the
    // methods available to this authenticated caller. The advertised
    // descriptor digest is retained below for diagnostics, but is not an
    // equality gate: compatible protobuf additions can change it. Exact
    // descriptor validation belongs to the adapter accepting encoded requests.
    if !dokime_protocol::compatible_api(&api.package, &api.version) {
        return Err(Error::IncompatibleApi);
    }
    if identity.backend_id.is_empty()
        || identity.epoch.is_empty()
        || expected_id.is_some_and(|expected| expected != identity.backend_id)
    {
        return Err(Error::IdentityChanged);
    }
    let health = client
        .get_health(request(v1::GetHealthRequest {}, authorization))
        .await
        .map_err(Error::Rpc)?
        .into_inner();
    let capabilities = client
        .get_capabilities(request(v1::GetCapabilitiesRequest {}, authorization))
        .await
        .map_err(Error::Rpc)?
        .into_inner();
    if health.backend_id != identity.backend_id
        || health.epoch != identity.epoch
        || capabilities.backend_id != identity.backend_id
        || capabilities.epoch != identity.epoch
    {
        return Err(Error::IdentityChanged);
    }
    if !matches!(
        health.readiness(),
        v1::Readiness::Ready | v1::Readiness::Degraded
    ) {
        return Err(Error::BackendUnavailable);
    }
    if dokime_protocol::DISCOVERY_METHODS
        .iter()
        .any(|required| !capabilities.methods.iter().any(|method| method == required))
    {
        return Err(Error::IncompatibleApi);
    }
    let limits = capabilities.limits.ok_or(Error::IncompatibleApi)?;
    if limits.max_request_bytes == 0 || limits.max_response_bytes == 0 {
        return Err(Error::IncompatibleApi);
    }
    Ok(Backend {
        backend_id: identity.backend_id,
        epoch: identity.epoch,
        display_name: identity.display_name,
        product_version: identity.product_version,
        api_version: api.version,
        descriptor_sha256: api
            .descriptor_sha256
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        methods: capabilities.methods,
        limits: Limits {
            max_request_bytes: limits.max_request_bytes,
            max_response_bytes: limits.max_response_bytes,
            max_content_bytes: limits.max_content_bytes,
            max_chunk_bytes: limits.max_chunk_bytes,
            max_page_items: limits.max_page_items,
            max_stream_buffer_items: limits.max_stream_buffer_items,
        },
    })
}

/* === Private Functions === */

/* === Tests === */
