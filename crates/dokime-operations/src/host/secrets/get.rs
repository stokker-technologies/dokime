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

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, host::Context};

use dokime_protocol::host::SecretsGetRequest;
use dokime_protocol::host::SecretsGetResponse;

use harmos_protocol::model::{OperationContract, ResourceRevision};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SecretsGetRequest, SecretsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SecretsGetRequest) -> Result<SecretsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        if let Some(caller) = &context.caller {
            let (name, binding_revision) = crate::host::plugins::secrets::resolve(context, &caller.plugin.name, &request.name).await?;
            let crate::host::secrets::store::Reply::Lookup(value, generation) =
                crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Lookup(name))
                    .await
                    .map_err(|_| Error::Permission(format!("{ID}: The Calling Plugin Cannot Read This Name")))?
            else {
                return Err(Error::Unavailable(format!("{ID}: Read Its Protected Revision")));
            };
            let secret =
                value.value.ok_or_else(|| Error::Permission(format!("{ID}: The Calling Plugin Cannot Read This Name")))?.into_bytes();
            let mut cut = crate::host::secrets::list::cut(context, &request.name, generation, None)?;
            cut.revisions.push(ResourceRevision {
                name: format!("{}/plugin-secret/{}/{}", context.host.identity().backend_id, caller.plugin.name, request.name),
                revision: binding_revision,
            });
            if request.at.as_ref().is_some_and(|at| at != &cut) {
                return Err(Error::Precondition(format!("{ID}: Its Protected Binding Is Unavailable; Resynchronize")));
            }
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
