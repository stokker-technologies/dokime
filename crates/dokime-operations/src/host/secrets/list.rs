/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SecretsListRequest, dokime_protocol::host::SecretsListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SecretsListRequest,
) -> Result<dokime_protocol::host::SecretsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let crate::host::secrets::store::Reply::Snapshot(snapshot) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::List).await?
        else {
            return Err(Error::Invalid("List Secrets: Restart the Secret Store".into()));
        };
        let cut = cut(context, "", snapshot.generation, request.at.as_ref())?;
        Ok(dokime_protocol::host::SecretsListResponse {
            entries: snapshot.entries,
            generation: snapshot.generation,
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

pub(crate) fn cut(context: &Context, name: &str, generation: u64, at: Option<&h::ReadCut>) -> Result<h::ReadCut> {
    let cut = h::ReadCut {
        positions: Vec::new(),
        revisions: vec![h::ResourceRevision {
            name: format!("{}/secrets/{name}", context.host.identity().backend_id),
            revision: generation.to_be_bytes().to_vec(),
        }],
    };
    if at.is_some_and(|at| at != &cut) {
        return Err(Error::Precondition("Read Secret: Its Protected Revision Is Unavailable; Resynchronize".into()));
    }
    Ok(cut)
}

/* === Tests === */
