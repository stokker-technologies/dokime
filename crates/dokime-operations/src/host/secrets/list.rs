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
use harmos_protocol::model::ReadCut;
use harmos_protocol::model::ResourceRevision;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::SecretsListRequest;
use dokime_protocol::host::SecretsListResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SecretsListRequest) -> Result<SecretsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let crate::host::secrets::store::Reply::Snapshot(snapshot) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::List).await?
        else {
            return Err(Error::Invalid("List Secrets: Restart the Secret Store".into()));
        };
        let cut = cut(context, "", snapshot.generation, request.at.as_ref())?;
        Ok(SecretsListResponse {
            entries: snapshot.entries,
            generation: snapshot.generation,
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

pub(crate) fn cut(context: &Context, name: &str, generation: u64, at: Option<&ReadCut>) -> Result<ReadCut> {
    let cut = ReadCut {
        positions: Vec::new(),
        revisions: vec![ResourceRevision {
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

crate::native::unary!(SecretsListRequest, SecretsListResponse, context, "dokime.host.v1", "Secrets", "List");
