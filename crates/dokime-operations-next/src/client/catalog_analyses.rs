/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.Catalog/Analyses";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CatalogAnalysesRequest, v1::CatalogAnalysesResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CatalogAnalysesRequest) -> Result<v1::CatalogAnalysesResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (mut reply,cut)=super::views::read(context,ID,request.at.as_ref(),async { 
        let mut definitions = std::collections::BTreeMap::new();
        for plugin in context.host.plugins()? {
            for section in plugin.description.and_then(|description| description.catalog).into_iter().flat_map(|catalog| catalog.sections) {
                if section.r#type != "dokime.v1.Analyses" {
                    continue;
                }
                                for definition in v1::Analyses::decode(section.payload.as_slice())?.analyses {
                    if let Some(metadata) = &definition.metadata {
                        definitions.insert(metadata.identifier.clone(), definition);
                    }
                }
            }
        }
Ok(v1::CatalogAnalysesResponse {definitions:definitions.into_values().collect(),..Default::default()}) }).await?;
reply.resolved_at=Some(cut);Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
