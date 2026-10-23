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
use super::{Backend, Error, Limits, Result, transport};
use dokime_domain::operations::Endpoint;
use dokime_domain::operations::backend;
use dokime_domain::operations::runtime;

/* === Constants === */

const DISCOVERY_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn discover(channel: Channel, authorization: Option<&MetadataValue<Ascii>>, expected_id: Option<&str>) -> Result<Backend> {
    let identity = transport::unary::<dokime_domain::model::backend::wire::BackendIdentity, _>(
        channel.clone(),
        authorization,
        backend::GetIdentity::PATH,
        backend::GetIdentity {},
        DISCOVERY_BYTES,
        DISCOVERY_BYTES,
    )
    .await?;
    let identity = transport::value(identity)?;
    let api = identity.api.as_ref().ok_or(Error::IncompatibleApi)?;
    if !dokime_domain::model::backend::wire::compatible_api(&api.package, &api.version) || api.descriptor_sha256.len() != 32 {
        return Err(Error::IncompatibleApi);
    }
    if identity.backend_id.is_empty() || identity.epoch.is_empty() || expected_id.is_some_and(|expected| expected != identity.backend_id) {
        return Err(Error::IdentityChanged);
    }
    let health = transport::unary::<dokime_domain::model::backend::wire::BackendHealth, _>(
        channel.clone(),
        authorization,
        backend::GetHealth::PATH,
        backend::GetHealth {},
        DISCOVERY_BYTES,
        DISCOVERY_BYTES,
    )
    .await?;
    let health = transport::value(health)?;
    let availability = transport::unary::<dokime_domain::model::backend::wire::BackendAvailability, _>(
        channel,
        authorization,
        backend::GetAvailability::PATH,
        backend::GetAvailability {},
        DISCOVERY_BYTES,
        DISCOVERY_BYTES,
    )
    .await?;
    let availability = transport::value(availability)?;
    if health.backend_id != identity.backend_id
        || health.epoch != identity.epoch
        || availability.backend_id != identity.backend_id
        || availability.epoch != identity.epoch
    {
        return Err(Error::IdentityChanged);
    }
    if !matches!(
        health.readiness,
        dokime_domain::model::backend::wire::Readiness::Ready | dokime_domain::model::backend::wire::Readiness::Degraded
    ) {
        return Err(Error::BackendUnavailable);
    }
    let required = [
        backend::GetIdentity::PATH,
        backend::GetHealth::PATH,
        backend::GetAvailability::PATH,
        runtime::GetIdentity::PATH,
        runtime::GetHealth::PATH,
    ];
    if required.iter().any(|required| !availability.methods.iter().any(|method| method == required)) {
        return Err(Error::IncompatibleApi);
    }
    let limits = availability.limits.as_ref().ok_or(Error::IncompatibleApi)?;
    if limits.max_request_bytes == 0 || limits.max_response_bytes == 0 {
        return Err(Error::IncompatibleApi);
    }
    let api_version = api.version.clone();
    let descriptor_sha256 = api.descriptor_sha256.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(Backend {
        backend_id: identity.backend_id,
        epoch: identity.epoch,
        display_name: identity.display_name,
        product_version: identity.product_version,
        target: identity.target,
        api_version,
        descriptor_sha256,
        methods: availability.methods,
        limits: Limits {
            max_request_bytes: limits.max_request_bytes,
            max_response_bytes: limits.max_response_bytes,
            max_content_bytes: limits.max_content_bytes,
            max_evidence_bytes: limits.max_evidence_bytes,
            max_chunk_bytes: limits.max_chunk_bytes,
            max_page_items: limits.max_page_items,
            max_stream_buffer_items: limits.max_stream_buffer_items,
        },
    })
}

/* === Private Functions === */

/* === Tests === */
