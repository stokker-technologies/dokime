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

pub const ID: &str = "dokime.remote.read_listener";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::RemoteReadListenerRequest, v1::RemoteListenerStatus>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::RemoteReadListenerRequest) -> Result<v1::RemoteListenerStatus> {
    super::complete(ID, context.settings.read_timeout, async {
        let _ = request;
        context.host.read_remote_listener().await
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
