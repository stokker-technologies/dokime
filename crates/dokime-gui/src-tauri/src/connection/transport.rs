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
    pub target: ConnectionTarget,
    pub endpoint: String,
    pub remote: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Target {
    pub fn parse(target: ConnectionTarget) -> Result<Self, ConnectionError> {
        let (endpoint, remote) = match &target {
            ConnectionTarget::Local { endpoint } => {
                super::host::validate_local(endpoint).map_err(ConnectionError::from)?;
                (endpoint.clone(), false)
            }
            ConnectionTarget::Remote { pairing_file } => {
                let pairing = super::host::Pairing::read(std::path::Path::new(pairing_file)).map_err(ConnectionError::from)?;
                (pairing.endpoints[0].clone(), true)
            }
        };
        Ok(Self { target, endpoint, remote })
    }
}

/* === Public Functions === */

pub(crate) fn backend(session: &super::host::Session) -> BackendSnapshot {
    let ready = session.identity();
    let status = ready.status.as_ref().expect("ready host status");
    let identity = status.identity.as_ref().expect("ready host identity");
    BackendSnapshot {
        backend_id: identity.host_id.clone(),
        epoch: session.product_epoch().to_owned(),
        display_name: identity.application_name.clone(),
        product_version: identity.application_version.clone(),
        source_revision: None,
        channel: None,
        target: String::new(),
        api_version: "v1".into(),
        descriptor_sha256: String::new(),
        methods: Vec::new(),
        limits: BackendLimits {
            max_request_bytes: crate::rpc::protocol::FRAME_BYTES.to_string(),
            max_response_bytes: crate::rpc::protocol::FRAME_BYTES.to_string(),
            max_content_bytes: None,
            max_evidence_bytes: None,
            max_chunk_bytes: Some(64 * 1024),
            max_page_items: Some(1000),
            max_stream_buffer_items: Some(8),
        },
    }
}

/* === Private Functions === */

/* === Tests === */
