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
use super::secrets_store::{Action, Reply};
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.secrets.preview";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::SecretsPreviewRequest, v1::SecretImportPreview>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::SecretsPreviewRequest) -> Result<v1::SecretImportPreview> {
    super::complete(ID, context.settings.read_timeout, async {
        let result = super::secrets_store::perform(context, Action::Preview(request.data)).await?;
        match result {
            Reply::Preview(reply) => Ok(reply),
            _ => Err(Error::Invalid("Secret Storage Returned the Wrong Reply; Restart the Host".into())),
        }
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
