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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;

/* === Constants === */

pub const ID: &str = "dokime.content.list_artifacts";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ContentListArtifactsRequest,v1::ListArtifactsReply>(ID,settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ContentListArtifactsRequest) -> Result<v1::ListArtifactsReply> {
    super::complete(ID,context.settings.read_timeout,async {
        let store=context.host.store();
        let cursor=v1::ArtifactCursor {epoch:context.host.identity().epoch,metadata_sha256:store.content_fingerprint(None)?};
        if request.cursor.as_ref().is_some_and(|given|given!=&cursor) || !request.after_artifact_id.is_empty() && request.cursor.is_none() {
            return Err(Error::Invalid("List Artifacts: Restart Pagination After Its Epoch or Inventory Changed".into()));
        }
        let limit=if request.limit==0 {context.settings.page_items} else {(request.limit as usize).min(context.settings.page_items)};
        let mut artifacts=store.artifacts_page(&request.after_artifact_id,limit+1)?;
        let more=artifacts.len()>limit; artifacts.truncate(limit);
        let next_after_artifact_id=if more {artifacts.last().map(|item|item.artifact_id.clone())} else {None};
        let reply=v1::ListArtifactsReply {artifacts,cursor:Some(cursor),next_after_artifact_id};
        if reply.encoded_len()>context.settings.message_bytes {return Err(Error::Invalid("List Artifacts: Reduce the Page Size".into()));}
        Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
