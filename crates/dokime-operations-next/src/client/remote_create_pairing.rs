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

pub const ID: &str = "dokime.remote.create_pairing";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::RemoteCreatePairingRequest, v1::CreatedPairing>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::RemoteCreatePairingRequest) -> Result<v1::CreatedPairing> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let host = context.host.clone();
        tokio::spawn(async move { host.create_pairing(request.label).await })
            .await
            .map_err(|_| crate::Error::Invalid("Configure Remote Access: Refresh Listener Status Before Retrying".into()))?
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
