/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// List Plugin Secret Bindings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::{Bindings, key, selected, values};
use crate::Error;
use crate::{
    Result,
    host::{ClientSettings, Context, storage::administration},
};
use dokime_protocol::host::plugins::{SecretsListRequest, SecretsListResponse};
use harmos_protocol::model::{OperationContract, ResourceRevision};
use prost::Message;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plugins.Secrets/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SecretsListRequest, SecretsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SecretsListRequest) -> Result<SecretsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let name = selected(request.name.as_deref(), ID)?;
        let mut storage_at = request.at.clone();
        if let Some(at) = &mut storage_at {
            at.revisions.clear();
        }
        let snapshot = administration::read(context, &key(name), &Bindings::default().encode_to_vec(), storage_at).await?;
        let catalog = Bindings::decode(snapshot.payload.as_slice())?;
        let bindings = values(context, name, &catalog).await?;
        let mut response = SecretsListResponse {
            bindings,
            revision: snapshot.revision,
            resolved_at: None,
        };
        let mut cut = snapshot.resolved_at.ok_or_else(|| Error::Precondition(format!("{ID}: Return Its Storage Cut")))?;
        cut.revisions.push(ResourceRevision {
            name: format!("{}/plugin-secrets-view/{name}", context.host.identity().backend_id),
            revision: crate::host::paging::bytes(&response),
        });
        if request.at.as_ref().is_some_and(|at| at != &cut) {
            return Err(Error::Precondition(format!("{ID}: Its Declaration or Availability Has Changed; Resynchronize")));
        }
        response.resolved_at = Some(cut);
        Ok(response)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
