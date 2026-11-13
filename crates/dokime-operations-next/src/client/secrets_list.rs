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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Secrets/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::SecretsListRequest, v1::SecretsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::SecretsListRequest) -> Result<v1::SecretsListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let super::secrets_store::Reply::Snapshot(snapshot) =
            super::secrets_store::perform(context, super::secrets_store::Action::List).await?
        else {
            return Err(Error::Invalid("List Secrets: Restart the Secret Store".into()));
        };
        let cut = cut(context, "", snapshot.generation, request.at.as_ref())?;
        Ok(v1::SecretsListResponse {
            entries: snapshot.entries,
            generation: snapshot.generation,
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

pub(super) fn cut(context: &Context, name: &str, generation: u64, at: Option<&h::ReadCut>) -> Result<h::ReadCut> {
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
