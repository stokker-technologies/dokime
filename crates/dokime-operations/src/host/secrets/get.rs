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

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::SecretsGetRequest;
use dokime_protocol::host::SecretsGetResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SecretsGetRequest) -> Result<SecretsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        if let Some(caller) = &context.caller {
            // A Declared Identifier Names the Host-Wide Secret Stored Under Exactly That Identifier.
            crate::host::plugins::secrets::require_declared(context, &caller.plugin.name, &request.name, ID).await?;
            if !crate::host::secrets::store::availability(&context.host.store(), &request.name)? {
                return Err(Error::NotFound(format!("{ID}: Add the Secret {:?} in Secret Storage", request.name)));
            }
            let crate::host::secrets::store::Reply::Lookup(value, generation) =
                crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Lookup(request.name.clone())).await?
            else {
                return Err(Error::Unavailable(format!("{ID}: Read Its Protected Revision")));
            };
            let secret = value
                .value
                .ok_or_else(|| Error::NotFound(format!("{ID}: Add the Secret {:?} in Secret Storage", request.name)))?
                .into_bytes();
            let cut = crate::host::secrets::list::cut(context, &request.name, generation, request.at.as_ref())?;
            return Ok(SecretsGetResponse {
                secret,
                value: None,
                revision: Vec::new(),
                resolved_at: Some(cut),
            });
        }
        let crate::host::secrets::store::Reply::Lookup(value, generation) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Lookup(request.name.clone())).await?
        else {
            return Err(Error::Invalid("Get Secret: Restart the Secret Store".into()));
        };
        let cut = crate::host::secrets::list::cut(context, &request.name, generation, request.at.as_ref())?;
        let secret =
            value.value.as_ref().ok_or_else(|| Error::Invalid("Get Secret: Restore Its Protected Value".into()))?.as_bytes().to_vec();
        Ok(SecretsGetResponse {
            secret,
            value: Some(value),
            revision: generation.to_be_bytes().to_vec(),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SecretsGetRequest, SecretsGetResponse, context, "dokime.host.v1", "Secrets", "Get");
