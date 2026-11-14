/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SecretsGetRequest, dokime_protocol::host::SecretsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SecretsGetRequest,
) -> Result<dokime_protocol::host::SecretsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let catalog = crate::host::secrets::list::handle(context, dokime_protocol::host::SecretsListRequest::default()).await?;
        let cut = crate::host::secrets::list::cut(context, &request.name, catalog.generation, request.at.as_ref())?;
        let crate::host::secrets::store::Reply::Lookup(value) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Lookup(request.name)).await?
        else {
            return Err(Error::Invalid("Get Secret: Restart the Secret Store".into()));
        };
        let secret =
            value.value.as_ref().ok_or_else(|| Error::Invalid("Get Secret: Restore Its Protected Value".into()))?.as_bytes().to_vec();
        Ok(dokime_protocol::host::SecretsGetResponse {
            secret,
            value: Some(value),
            revision: catalog.generation.to_be_bytes().to_vec(),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
