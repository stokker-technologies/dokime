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

pub const ID: &str = "dokime.remote.regenerate_certificate";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::RemoteRegenerateCertificateRequest, v1::RemoteListenerStatus>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::RemoteRegenerateCertificateRequest) -> Result<v1::RemoteListenerStatus> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let host = context.host.clone();
        tokio::spawn(async move { host.regenerate_certificate(request.certificate_names).await })
            .await
            .map_err(|_| crate::Error::Invalid("Configure Remote Access: Refresh Listener Status Before Retrying".into()))?
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
