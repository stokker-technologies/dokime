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
// Local Imports
use super::{BackendLimits, BackendSnapshot, ConnectionError, ConnectionTarget};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Target {
    pub endpoint: String,
    pub remote: bool,
    pub inner: dokime_client::Target,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Target {
    pub fn parse(target: ConnectionTarget) -> Result<Self, ConnectionError> {
        let inner = match target {
            ConnectionTarget::Local { endpoint } => dokime_client::Target::local(endpoint),
            ConnectionTarget::Remote {
                endpoint,
                ca_certificate_pem,
            } => dokime_client::Target::remote(endpoint, ca_certificate_pem),
        }
        .map_err(ConnectionError::from)?;
        Ok(Self {
            endpoint: inner.endpoint().to_owned(),
            remote: inner.is_remote(),
            inner,
        })
    }
}

/* === Public Functions === */
pub(super) fn backend(value: &dokime_client::Backend) -> BackendSnapshot {
    BackendSnapshot {
        backend_id: value.backend_id.clone(),
        epoch: value.epoch.clone(),
        display_name: value.display_name.clone(),
        product_version: value.product_version.clone(),
        api_version: value.api_version.clone(),
        descriptor_sha256: value.descriptor_sha256.clone(),
        methods: value.methods.clone(),
        limits: BackendLimits {
            max_request_bytes: value.limits.max_request_bytes.to_string(),
            max_response_bytes: value.limits.max_response_bytes.to_string(),
            max_content_bytes: value
                .limits
                .max_content_bytes
                .map(|value| value.to_string()),
            max_chunk_bytes: value.limits.max_chunk_bytes,
            max_page_items: value.limits.max_page_items,
            max_stream_buffer_items: value.limits.max_stream_buffer_items,
        },
    }
}

/* === Private Functions === */

/* === Tests === */
