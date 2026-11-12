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
use sha2::Digest;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.runtime.identity";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::RuntimeGetIdentityRequest, v1::BackendIdentity>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::RuntimeGetIdentityRequest) -> Result<v1::BackendIdentity> {
    super::complete(ID, context.settings.read_timeout, async {
        let _ = request;
        let mut identity = context.host.identity();
        if let Some(api) = &mut identity.api {
            api.descriptor_sha256 = sha2::Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET).to_vec();
        }
        Ok(identity)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
